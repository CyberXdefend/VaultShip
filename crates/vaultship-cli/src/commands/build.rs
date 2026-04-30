use base64::Engine as _;
use serde_json::json;
use std::path::{Path, PathBuf};

pub async fn run(path: &str) -> anyhow::Result<()> {
    if Path::new("vaultship.toml").exists() {
        let _ = crate::config::load("vaultship.toml")?;
    }

    let input = Path::new(path);
    let compose_file = if input.is_dir() {
        input.join("docker-compose.yml")
    } else {
        input.to_path_buf()
    };
    if !compose_file.exists() {
        anyhow::bail!("Compose file not found at {}", compose_file.display());
    }

    let compose_content = std::fs::read_to_string(&compose_file)?;
    let service_name = infer_service_name(&compose_content).unwrap_or_else(|| "app".to_string());

    let harden_config = vaultship_harden::HardenConfig {
        read_only: true,
        drop_capabilities: true,
        no_new_privileges: true,
        seccomp_profile: vaultship_harden::SeccompMode::Auto,
        anti_debug: true,
    };

    let hardened = vaultship_harden::generate_hardened_compose(&service_name, &harden_config)?;
    std::fs::write("docker-compose.hardened.yml", hardened)?;
    let profile = vaultship_harden::seccomp::generate_anti_extraction_profile();
    vaultship_harden::seccomp::write_profile(&profile, "seccomp-profile.json")?;

    let artifact_name = service_name.replace(['/', ':'], "-");
    let artifacts_dir = PathBuf::from(".vaultship/artifacts");
    let keys_dir = PathBuf::from(".vaultship/keys");
    std::fs::create_dir_all(&artifacts_dir)?;
    std::fs::create_dir_all(&keys_dir)?;

    let key = vaultship_encrypt::keys::generate_layer_key();
    let encrypted = vaultship_encrypt::encrypt::encrypt_layer(compose_content.as_bytes(), &key)?;
    let enc_path = artifacts_dir.join(format!("{artifact_name}.layer.enc.json"));
    std::fs::write(&enc_path, serde_json::to_string_pretty(&encrypted)?)?;

    let key_b64 = base64::engine::general_purpose::STANDARD.encode(key);
    let key_path = keys_dir.join(format!("{artifact_name}.key"));
    std::fs::write(&key_path, key_b64)?;
    let default_key = Path::new("vaultship.layer.key");
    if !default_key.exists() {
        std::fs::write(default_key, base64::engine::general_purpose::STANDARD.encode(key))?;
    }

    let signed_ref = vaultship_sign::sign::sign_image_reference(&artifact_name, "vaultship")?;
    let manifest = json!({
        "service": service_name,
        "source_compose": compose_file.display().to_string(),
        "hardened_compose": "docker-compose.hardened.yml",
        "seccomp_profile": "seccomp-profile.json",
        "artifact": enc_path.display().to_string(),
        "key": key_path.display().to_string(),
        "signed_ref": signed_ref,
    });
    let manifest_path = artifacts_dir.join(format!("{artifact_name}.manifest.json"));
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

    println!(
        "Build complete: hardened + encrypted artifact created for service `{}`",
        artifact_name
    );
    Ok(())
}

fn infer_service_name(compose: &str) -> Option<String> {
    let mut in_services = false;
    for line in compose.lines() {
        let trimmed = line.trim_end();
        if trimmed.trim() == "services:" {
            in_services = true;
            continue;
        }
        if in_services && line.starts_with("  ") && trimmed.ends_with(':') {
            let name = trimmed.trim().trim_end_matches(':').to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}
