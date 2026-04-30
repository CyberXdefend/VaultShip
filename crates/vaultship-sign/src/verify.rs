use anyhow::{Result, bail};
use std::process::Command;

pub fn verify_signature(image_ref: &str) -> Result<()> {
    if std::env::var("VAULTSHIP_ENFORCE_COSIGN")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        let output = Command::new("cosign").arg("verify").arg(image_ref).output()?;
        if !output.status.success() {
            bail!(
                "Cosign verification failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        return Ok(());
    }

    if !image_ref.contains("@sig:") {
        bail!("Missing signature marker");
    }
    Ok(())
}
