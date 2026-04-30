use anyhow::{Result, bail};
use base64::Engine as _;
use chrono::Utc;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use crate::create::{License, LicensePayload};

pub fn validate_license(license: &License, public_key: &VerifyingKey) -> Result<()> {
    let payload = serde_json::to_string(&LicensePayload::from(license))?;
    let sig_bytes = base64::engine::general_purpose::STANDARD.decode(&license.signature)?;
    if sig_bytes.len() != 64 {
        bail!("Invalid signature length");
    }
    let mut sig = [0_u8; 64];
    sig.copy_from_slice(&sig_bytes);
    let signature = Signature::from_bytes(&sig);

    public_key
        .verify(payload.as_bytes(), &signature)
        .map_err(|_| anyhow::anyhow!("Invalid license signature"))?;

    if let Some(expires) = license.expires_at && Utc::now() > expires {
        bail!("License expired on {expires}");
    }

    if let Some(ref expected_fp) = license.hardware_fingerprint && !expected_fp.verify_current()? {
        bail!("License not valid for this hardware");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_license;
    use crate::create::License;

    #[test]
    fn accepts_valid_signed_license() {
        let (signing, verifying) = crate::create::keygen();
        let license = License::create(
            &signing,
            "LawFirm-ABC",
            "vaultship-demo",
            None,
            None,
            5,
            vec!["scan".to_string()],
        )
        .expect("create");
        validate_license(&license, &verifying).expect("validate");
    }
}
