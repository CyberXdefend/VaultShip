use base64::Engine as _;
use vaultship_encrypt::encrypt::EncryptedLayer;

pub async fn run(input: &str, output: Option<&str>, key_file: &str) -> anyhow::Result<()> {
    let encrypted: EncryptedLayer = serde_json::from_str(&std::fs::read_to_string(input)?)?;
    let key = read_key(key_file)?;
    let plaintext = vaultship_encrypt::decrypt::decrypt_layer(&encrypted, &key)?;

    let out = output
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("{input}.dec"));
    std::fs::write(&out, plaintext)?;
    println!("Decrypted artifact written to {out}");
    Ok(())
}

fn read_key(path: &str) -> anyhow::Result<[u8; 32]> {
    let raw = base64::engine::general_purpose::STANDARD.decode(std::fs::read_to_string(path)?.trim())?;
    raw.try_into()
        .map_err(|_| anyhow::anyhow!("Encryption key must be 32 bytes in base64"))
}
