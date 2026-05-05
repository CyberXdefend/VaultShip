use chrono::Utc;
use vaultship_license::{License, create::keygen, validate::validate_license};

#[test]
fn create_and_validate_basic_license() {
    let (signing, verifying) = keygen();
    let license = License::create(
        &signing,
        "Acme Corp",
        "vaultship-pro",
        None,
        None,
        5,
        vec!["scan".to_string(), "harden".to_string()],
    )
    .expect("create license");

    assert_eq!(license.customer, "Acme Corp");
    assert_eq!(license.seats, 5);
    validate_license(&license, &verifying).expect("valid license must pass");
}

#[test]
fn expired_license_is_rejected() {
    let (signing, verifying) = keygen();
    let past = Utc::now() - chrono::Duration::days(1);
    let license = License::create(
        &signing,
        "Acme Corp",
        "vaultship-pro",
        None,
        Some(past),
        1,
        vec![],
    )
    .expect("create");
    assert!(
        validate_license(&license, &verifying).is_err(),
        "expired must fail"
    );
}

#[test]
fn wrong_public_key_is_rejected() {
    let (signing, _) = keygen();
    let (_, other_verifying) = keygen();
    let license = License::create(
        &signing,
        "Acme Corp",
        "vaultship-pro",
        None,
        None,
        1,
        vec![],
    )
    .expect("create");
    assert!(validate_license(&license, &other_verifying).is_err());
}

#[test]
fn tampered_customer_is_rejected() {
    let (signing, verifying) = keygen();
    let mut license = License::create(
        &signing,
        "Acme Corp",
        "vaultship-pro",
        None,
        None,
        1,
        vec![],
    )
    .expect("create");
    license.customer = "Evil Corp".to_string();
    assert!(validate_license(&license, &verifying).is_err());
}

#[test]
fn future_expiry_passes() {
    let (signing, verifying) = keygen();
    let future = Utc::now() + chrono::Duration::days(365);
    let license = License::create(
        &signing,
        "Acme Corp",
        "vaultship-pro",
        None,
        Some(future),
        3,
        vec!["scan".to_string()],
    )
    .expect("create");
    validate_license(&license, &verifying).expect("future expiry must pass");
}
