use anyhow::Result;
use base64::Engine as _;
use ed25519_dalek::Signer;
use std::process::Command;

/// Signs `image` with the Ed25519 private key at `private_key_path`.
/// Returns a signed reference of the form `{image}@sig:{base64_signature}`.
/// Falls back to cosign if `VAULTSHIP_COSIGN_KEY` is set and cosign is available.
pub fn sign_image_reference(image: &str, private_key_path: &str) -> Result<String> {
    if let Ok(cosign_key) = std::env::var("VAULTSHIP_COSIGN_KEY") {
        let output = Command::new("cosign")
            .arg("sign")
            .arg("--yes")
            .arg("--key")
            .arg(cosign_key)
            .arg(image)
            .output();
        if let Ok(result) = output
            && result.status.success()
        {
            return Ok(image.to_string());
        }
    }

    let key_b64 = std::fs::read_to_string(private_key_path).map_err(|_| {
        anyhow::anyhow!(
            "Private key not found at `{private_key_path}`. Run `vaultship keygen` first."
        )
    })?;
    let key_bytes = base64::engine::general_purpose::STANDARD.decode(key_b64.trim())?;
    let key_arr: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Private key must be 32 bytes (Ed25519)"))?;
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&key_arr);
    let signature = signing_key.sign(image.as_bytes());
    let sig_b64 = base64::engine::general_purpose::STANDARD.encode(signature.to_bytes());
    Ok(format!("{image}@sig:{sig_b64}"))
}

#[cfg(test)]
mod tests {
    use super::sign_image_reference;
    use crate::verify::verify_signature;

    #[test]
    fn roundtrip_sign_verify() {
        let (signing, verifying) = vaultship_keys_for_test();
        let priv_path = write_temp_key(&signing);
        let pub_path = write_temp_key(&verifying);

        let signed = sign_image_reference("myapp:v1.0", &priv_path).expect("sign");
        assert!(signed.contains("@sig:"), "must contain @sig: marker");
        verify_signature(&signed, &pub_path).expect("verify must pass");

        // Tampered image name must fail
        let tampered = signed.replace("myapp:v1.0", "evil:latest");
        assert!(verify_signature(&tampered, &pub_path).is_err());
    }

    #[test]
    fn wrong_key_fails_verification() {
        let (signing_b64, _) = make_keypair(1);
        let (_, wrong_pub_b64) = make_keypair(2); // different seed → different key
        let priv_path = write_temp_key(&signing_b64);
        let wrong_pub_path = write_temp_key(&wrong_pub_b64);

        let signed = sign_image_reference("myapp:v1.0", &priv_path).expect("sign");
        assert!(verify_signature(&signed, &wrong_pub_path).is_err());
    }

    fn make_keypair(seed: u8) -> (String, String) {
        use base64::Engine as _;
        let mut secret = [0u8; 32];
        for (i, x) in secret.iter_mut().enumerate() {
            *x = (i as u8).wrapping_mul(37).wrapping_add(seed);
        }
        let signing = ed25519_dalek::SigningKey::from_bytes(&secret);
        let verifying = signing.verifying_key();
        let priv_b64 = base64::engine::general_purpose::STANDARD.encode(signing.to_bytes());
        let pub_b64 = base64::engine::general_purpose::STANDARD.encode(verifying.to_bytes());
        (priv_b64, pub_b64)
    }

    fn vaultship_keys_for_test() -> (String, String) {
        make_keypair(1)
    }

    fn write_temp_key(key_b64: &str) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        // Hash content to avoid collisions between priv/pub written in the same nanosecond.
        let discriminator: u64 = key_b64
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_add(b as u64));
        let path = format!("/tmp/vaultship-sign-test-{nanos}-{discriminator}.key");
        std::fs::write(&path, key_b64).unwrap();
        path
    }
}
