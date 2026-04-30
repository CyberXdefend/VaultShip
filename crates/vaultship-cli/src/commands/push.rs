use std::path::PathBuf;
use vaultship_encrypt::encrypt::EncryptedLayer;

pub async fn run(registry: &str) -> anyhow::Result<()> {
    let src = PathBuf::from(".vaultship/artifacts");
    if !src.exists() {
        anyhow::bail!("No local artifacts found. Run `vaultship build` first.");
    }

    if PathBuf::from(registry).exists() {
        let dest = PathBuf::from(registry);
        std::fs::create_dir_all(&dest)?;

        let mut copied = 0usize;
        for entry in std::fs::read_dir(&src)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let to = dest.join(entry.file_name());
                std::fs::copy(entry.path(), to)?;
                copied += 1;
            }
        }
        println!("Pushed {copied} artifact file(s) to {}", dest.display());
        return Ok(());
    }

    // Real OCI push path.
    // Example registry arg: http://localhost:5000/vaultship
    let mut pushed = 0usize;
    for entry in std::fs::read_dir(&src)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        if !file_name.ends_with(".layer.enc.json") {
            continue;
        }
        let service = file_name.trim_end_matches(".layer.enc.json");
        let reference = format!("{}/{}:latest", registry.trim_end_matches('/'), service);
        let encrypted: EncryptedLayer = serde_json::from_str(&std::fs::read_to_string(entry.path())?)?;
        vaultship_encrypt::registry::push_encrypted_layer(&reference, &encrypted).await?;
        println!("Pushed encrypted OCI artifact: {reference}");
        pushed += 1;
    }
    if pushed == 0 {
        anyhow::bail!("No encrypted layer artifacts found under .vaultship/artifacts");
    }

    Ok(())
}
