use anyhow::Result;
use sha2::{Digest, Sha256};
use std::process::Command;

pub fn sign_image_reference(image: &str, signer: &str) -> Result<String> {
    if let Ok(key) = std::env::var("VAULTSHIP_COSIGN_KEY") {
        let output = Command::new("cosign")
            .arg("sign")
            .arg("--yes")
            .arg("--key")
            .arg(key)
            .arg(image)
            .output();
        if let Ok(result) = output
            && result.status.success()
        {
            return Ok(image.to_string());
        }
    }

    let mut hasher = Sha256::new();
    hasher.update(image.as_bytes());
    hasher.update(signer.as_bytes());
    let sig = format!("{:x}", hasher.finalize());
    Ok(format!("{image}@sig:{sig}"))
}
