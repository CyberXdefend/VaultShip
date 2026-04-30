use anyhow::Result;

pub fn generate_activation_challenge(fingerprint_json: &str) -> Result<String> {
    let digest = blake3::hash(fingerprint_json.as_bytes());
    Ok(format!("activation:{}", digest.to_hex()))
}
