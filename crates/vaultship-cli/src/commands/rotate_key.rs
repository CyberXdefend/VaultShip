use base64::Engine as _;
use vaultship_encrypt::encrypt::EncryptedLayer;

pub async fn run(
    input: &str,
    old_key: &str,
    new_key: &str,
    output: Option<&str>,
) -> anyhow::Result<()> {
    let encrypted: EncryptedLayer = serde_json::from_str(&std::fs::read_to_string(input)?)?;
    let old = read_key(old_key)?;
    let new = read_key(new_key)?;

    let plaintext = vaultship_encrypt::decrypt::decrypt_layer(&encrypted, &old)?;
    let rotated = vaultship_encrypt::encrypt::encrypt_layer(&plaintext, &new)?;

    let out = output
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("{input}.rotated.json"));
    std::fs::write(&out, serde_json::to_string_pretty(&rotated)?)?;
    println!("Rotated encryption key; output written to {out}");
    Ok(())
}

fn read_key(path: &str) -> anyhow::Result<[u8; 32]> {
    let raw =
        base64::engine::general_purpose::STANDARD.decode(std::fs::read_to_string(path)?.trim())?;
    raw.try_into()
        .map_err(|_| anyhow::anyhow!("Key must be 32 bytes in base64"))
}
