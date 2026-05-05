use base64::Engine as _;
use vaultship_encrypt::{decrypt::decrypt_layer, encrypt::encrypt_layer, keys::generate_layer_key};
use vaultship_license::{License, create::keygen as license_keygen, validate::validate_license};
use vaultship_sign::{sign::sign_image_reference, verify::verify_signature};

/// Writes a temporary key file with a unique label and returns its path.
fn write_key_file(content: &str, label: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = format!("/tmp/vaultship-pipeline-{label}-{nanos}.key");
    std::fs::write(&path, content).unwrap();
    path
}

#[test]
fn encrypt_decrypt_sign_verify_pipeline() {
    // 1. Generate keys
    let secret: [u8; 32] = {
        let mut b = [0u8; 32];
        for (i, x) in b.iter_mut().enumerate() {
            *x = (i as u8).wrapping_add(42);
        }
        b
    };
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret);
    let verifying_key = signing_key.verifying_key();
    let priv_b64 = base64::engine::general_purpose::STANDARD.encode(signing_key.to_bytes());
    let pub_b64 = base64::engine::general_purpose::STANDARD.encode(verifying_key.to_bytes());
    let priv_path = write_key_file(&priv_b64, "priv-a");
    let pub_path = write_key_file(&pub_b64, "pub-a");

    // 2. Encrypt a payload
    let layer_key = generate_layer_key();
    let payload = b"docker-compose.yml contents for myapp:v2";
    let encrypted = encrypt_layer(payload, &layer_key).expect("encrypt");

    // 3. Decrypt it back
    let decrypted = decrypt_layer(&encrypted, &layer_key).expect("decrypt");
    assert_eq!(decrypted, payload);

    // 4. Sign the artifact name
    let image = "myapp:v2";
    let signed_ref = sign_image_reference(image, &priv_path).expect("sign");
    assert!(signed_ref.starts_with("myapp:v2@sig:"));

    // 5. Verify the signature
    verify_signature(&signed_ref, &pub_path).expect("verify must pass");

    // Cleanup
    let _ = std::fs::remove_file(&priv_path);
    let _ = std::fs::remove_file(&pub_path);
}

#[test]
fn license_and_encryption_together() {
    // Generate license
    let (signing, verifying) = license_keygen();
    let license = License::create(
        &signing,
        "Pipeline Test Corp",
        "vaultship-full",
        None,
        None,
        10,
        vec!["encrypt".to_string(), "sign".to_string()],
    )
    .expect("create license");
    validate_license(&license, &verifying).expect("license valid");

    // Encrypt something
    let key = generate_layer_key();
    let plaintext = b"protected layer bytes";
    let enc = encrypt_layer(plaintext, &key).expect("encrypt");
    let dec = decrypt_layer(&enc, &key).expect("decrypt");
    assert_eq!(dec, plaintext);
}

#[test]
fn tampered_signed_ref_is_rejected() {
    let secret: [u8; 32] = [99u8; 32];
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret);
    let verifying_key = signing_key.verifying_key();
    let priv_b64 = base64::engine::general_purpose::STANDARD.encode(signing_key.to_bytes());
    let pub_b64 = base64::engine::general_purpose::STANDARD.encode(verifying_key.to_bytes());
    let priv_path = write_key_file(&priv_b64, "priv-b");
    let pub_path = write_key_file(&pub_b64, "pub-b");

    let signed = sign_image_reference("service:v1", &priv_path).expect("sign");

    // Tamper the image name in the signed ref
    let tampered = signed.replace("service:v1", "evil:v1");
    assert!(verify_signature(&tampered, &pub_path).is_err());

    // Tamper the signature itself
    let parts: Vec<&str> = signed.splitn(2, "@sig:").collect();
    let bad_ref = format!(
        "{}@sig:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",
        parts[0]
    );
    assert!(verify_signature(&bad_ref, &pub_path).is_err());

    let _ = std::fs::remove_file(&priv_path);
    let _ = std::fs::remove_file(&pub_path);
}
