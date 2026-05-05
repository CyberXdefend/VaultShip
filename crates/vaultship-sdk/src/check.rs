use anyhow::{Result, bail};
use base64::Engine as _;
use ed25519_dalek::VerifyingKey;

pub fn validate(license_path: &str, public_key_base64: &[u8]) -> Result<()> {
    let license_data = std::fs::read_to_string(license_path)?;
    let license: vaultship_license::License = serde_json::from_str(&license_data)?;

    let key_bytes = base64::engine::general_purpose::STANDARD
        .decode(std::str::from_utf8(public_key_base64)?)?;
    if key_bytes.len() != 32 {
        bail!("Public key must be 32 bytes (base64) for Ed25519");
    }
    let mut raw = [0_u8; 32];
    raw.copy_from_slice(&key_bytes);
    let public_key = VerifyingKey::from_bytes(&raw)?;

    vaultship_license::validate::validate_license(&license, &public_key)?;
    Ok(())
}
