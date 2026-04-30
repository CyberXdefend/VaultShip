use base64::Engine as _;

pub async fn run(input: &str, output: Option<&str>, key_file: &str) -> anyhow::Result<()> {
    let plaintext = std::fs::read(input)?;
    let key = read_key(key_file)?;
    let encrypted = vaultship_encrypt::encrypt::encrypt_layer(&plaintext, &key)?;

    let out = output
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("{input}.enc.json"));
    std::fs::write(&out, serde_json::to_string_pretty(&encrypted)?)?;
    println!("Encrypted artifact written to {out}");
    Ok(())
}

fn read_key(path: &str) -> anyhow::Result<[u8; 32]> {
    let raw = base64::engine::general_purpose::STANDARD.decode(std::fs::read_to_string(path)?.trim())?;
    raw.try_into()
        .map_err(|_| anyhow::anyhow!("Encryption key must be 32 bytes in base64"))
}
