use anyhow::{Result, bail};
use base64::Engine as _;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use std::process::Command;

/// Verifies a signed image reference produced by `sign_image_reference`.
/// Format: `{image}@sig:{base64_ed25519_signature}`
///
/// When `VAULTSHIP_ENFORCE_COSIGN=1` is set, delegates to the `cosign` CLI instead.
pub fn verify_signature(image_ref: &str, public_key_path: &str) -> Result<()> {
    if std::env::var("VAULTSHIP_ENFORCE_COSIGN")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        let output = Command::new("cosign")
            .arg("verify")
            .arg(image_ref)
            .output()?;
        if !output.status.success() {
            bail!(
                "Cosign verification failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        return Ok(());
    }

    let (image, sig_b64) = image_ref
        .split_once("@sig:")
        .ok_or_else(|| anyhow::anyhow!("Missing @sig: marker in ref: {image_ref}"))?;

    let key_b64 = std::fs::read_to_string(public_key_path).map_err(|_| {
        anyhow::anyhow!(
            "Public key not found at `{public_key_path}`. Run `vaultship keygen` first."
        )
    })?;
    let key_bytes = base64::engine::general_purpose::STANDARD.decode(key_b64.trim())?;
    let key_arr: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Public key must be 32 bytes (Ed25519)"))?;
    let verifying_key = VerifyingKey::from_bytes(&key_arr)?;

    let sig_bytes = base64::engine::general_purpose::STANDARD.decode(sig_b64.trim())?;
    let sig_arr: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid signature length in ref: expected 64 bytes"))?;
    let signature = Signature::from_bytes(&sig_arr);

    verifying_key
        .verify(image.as_bytes(), &signature)
        .map_err(|_| anyhow::anyhow!("Signature verification failed for `{image}`"))?;

    Ok(())
}
