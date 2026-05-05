use base64::Engine as _;
use serde_json::json;
use std::path::{Path, PathBuf};

pub async fn run(path: &str) -> anyhow::Result<()> {
    let cfg = if Path::new("vaultship.toml").exists() {
        Some(crate::config::load("vaultship.toml")?)
    } else {
        None
    };

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

    // Build harden config from vaultship.toml or use defaults.
    let harden_config = if let Some(ref c) = cfg {
        vaultship_harden::HardenConfig {
            read_only: c.harden.read_only,
            drop_capabilities: c.harden.drop_capabilities,
            no_new_privileges: c.harden.no_new_privileges,
            seccomp_profile: match c.harden.seccomp_profile.as_str() {
                "strict" => vaultship_harden::SeccompMode::Strict,
                "auto" => vaultship_harden::SeccompMode::Auto,
                custom => vaultship_harden::SeccompMode::Custom(custom.to_string()),
            },
            anti_debug: c.harden.anti_debug,
        }
    } else {
        vaultship_harden::HardenConfig {
            read_only: true,
            drop_capabilities: true,
            no_new_privileges: true,
            seccomp_profile: vaultship_harden::SeccompMode::Auto,
            anti_debug: true,
        }
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

    // Try to export the actual Docker image; fall back to encrypting compose config.
    let image_ref = infer_image_ref(&compose_content);
    let (plaintext, content_type) = match image_ref {
        Some(ref img) => match export_docker_image(img).await {
            Ok(bytes) => {
                println!("Exported Docker image `{img}` ({} bytes)", bytes.len());
                (bytes, "docker-image-tar")
            }
            Err(e) => {
                eprintln!(
                    "WARNING: Could not export Docker image `{img}` ({e}). \
                     Encrypting compose configuration instead. \
                     For full image encryption, ensure Docker is running and the image exists locally."
                );
                (compose_content.as_bytes().to_vec(), "compose-config")
            }
        },
        None => (compose_content.as_bytes().to_vec(), "compose-config"),
    };

    let key = vaultship_encrypt::keys::generate_layer_key();
    let encrypted = vaultship_encrypt::encrypt::encrypt_layer(&plaintext, &key)?;
    let enc_path = artifacts_dir.join(format!("{artifact_name}.layer.enc.json"));
    std::fs::write(&enc_path, serde_json::to_string_pretty(&encrypted)?)?;

    // Always write the per-artifact key AND update the root convenience key.
    let key_b64 = base64::engine::general_purpose::STANDARD.encode(key);
    let key_path = keys_dir.join(format!("{artifact_name}.key"));
    std::fs::write(&key_path, &key_b64)?;
    std::fs::write("vaultship.layer.key", &key_b64)?;

    let private_key_path = cfg
        .as_ref()
        .and_then(|c| c.bind.as_ref())
        .map(|b| b.public_key_path.replace("public", "private"))
        .unwrap_or_else(|| "vaultship.private.key".to_string());

    let signed_ref = vaultship_sign::sign::sign_image_reference(&artifact_name, &private_key_path)?;
    let manifest = json!({
        "service": service_name,
        "content_type": content_type,
        "source_compose": compose_file.display().to_string(),
        "hardened_compose": "docker-compose.hardened.yml",
        "seccomp_profile": "seccomp-profile.json",
        "artifact": enc_path.display().to_string(),
        "key": key_path.display().to_string(),
        "signed_ref": signed_ref,
    });
    let manifest_path = artifacts_dir.join(format!("{artifact_name}.manifest.json"));
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

    println!("Build complete: {content_type} encrypted artifact for service `{artifact_name}`");
    Ok(())
}

/// Export a Docker image to a tar byte stream using the local Docker daemon.
async fn export_docker_image(image: &str) -> anyhow::Result<Vec<u8>> {
    use bollard::Docker;
    use bollard::image::CreateImageOptions;
    use futures_util::TryStreamExt;

    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| anyhow::anyhow!("Docker daemon not available: {e}"))?;

    // Pull if not present locally.
    if docker.inspect_image(image).await.is_err() {
        docker
            .create_image(
                Some(CreateImageOptions {
                    from_image: image,
                    ..Default::default()
                }),
                None,
                None,
            )
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to pull image `{image}`: {e}"))?;
    }

    let bytes: Vec<u8> = docker
        .export_image(image)
        .map_ok(|chunk| chunk.to_vec())
        .try_collect::<Vec<_>>()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to export image `{image}`: {e}"))?
        .into_iter()
        .flatten()
        .collect();

    if bytes.is_empty() {
        anyhow::bail!("Docker exported empty image for `{image}`");
    }
    Ok(bytes)
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

fn infer_image_ref(compose: &str) -> Option<String> {
    for line in compose.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("image:") {
            let img = rest.trim().trim_matches('"').trim_matches('\'').to_string();
            if !img.is_empty() {
                return Some(img);
            }
        }
    }
    None
}
