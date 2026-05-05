use base64::Engine as _;
use std::path::PathBuf;
use vaultship_encrypt::decrypt::decrypt_layer;
use vaultship_encrypt::encrypt::EncryptedLayer;

pub async fn run(image: &str) -> anyhow::Result<()> {
    let pulled = PathBuf::from(".vaultship/pulled");
    std::fs::create_dir_all(&pulled)?;

    let encrypted: EncryptedLayer = if image.contains('/') {
        vaultship_encrypt::registry::pull_encrypted_layer(image).await?
    } else {
        let registry = PathBuf::from(".vaultship/registry");
        let artifact = registry.join(format!("{image}.layer.enc.json"));
        if !artifact.exists() {
            anyhow::bail!(
                "Registry artifact not found at {}. Use `vaultship push .vaultship/registry` first.",
                artifact.display()
            );
        }
        serde_json::from_str(&std::fs::read_to_string(&artifact)?)?
    };

    let image_name = image
        .split('/')
        .next_back()
        .unwrap_or(image)
        .split(':')
        .next()
        .unwrap_or(image);
    let key_file = PathBuf::from(".vaultship/keys").join(format!("{image_name}.key"));
    if !key_file.exists() {
        anyhow::bail!("Key not found at {}", key_file.display());
    }

    let key_raw = base64::engine::general_purpose::STANDARD
        .decode(std::fs::read_to_string(&key_file)?.trim())?;
    let key: [u8; 32] = key_raw
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid key length for {image_name}"))?;

    let plaintext = decrypt_layer(&encrypted, &key)?;
    let out = pulled.join(format!("{image_name}.layer.tar"));
    std::fs::write(&out, plaintext)?;
    println!("Pulled and validated `{image_name}` to {}", out.display());
    Ok(())
}
