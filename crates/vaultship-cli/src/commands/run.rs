use base64::Engine as _;
use std::{path::Path, process::Command};

pub async fn run(
    image: &str,
    bind_file: Option<&str>,
    public_key_path: &str,
    dry_run: bool,
    extra_args: &[String],
    license: Option<&str>,
    engine: &str,
) -> anyhow::Result<()> {
    if let Some(path) = bind_file {
        let _key = crate::commands::bind::validate_bound_file(path, public_key_path)?;
        println!("Hardware-bound key validation passed.");
    }

    // Backward-compatible mode for existing license command flow.
    if let Some(path) = license {
        if !Path::new(public_key_path).exists() {
            anyhow::bail!(
                "License provided but public key missing at `{}`",
                public_key_path
            );
        }

        let license: vaultship_license::License =
            serde_json::from_str(&std::fs::read_to_string(path)?)?;
        let key_bytes = base64::engine::general_purpose::STANDARD
            .decode(std::fs::read_to_string(public_key_path)?.trim())?;
        let key: [u8; 32] = key_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Public key must be 32 bytes"))?;
        let verifying = ed25519_dalek::VerifyingKey::from_bytes(&key)?;
        vaultship_license::validate::validate_license(&license, &verifying)?;
        println!("License validation passed.");
    }

    let mut docker_args = vec![
        "run".to_string(),
        "--rm".to_string(),
        "--read-only".to_string(),
        "--security-opt".to_string(),
        "no-new-privileges:true".to_string(),
        "--security-opt".to_string(),
        "seccomp=./seccomp-profile.json".to_string(),
        "--cap-drop".to_string(),
        "ALL".to_string(),
        "--cap-add".to_string(),
        "NET_BIND_SERVICE".to_string(),
        "--tmpfs".to_string(),
        "/tmp:noexec,nosuid".to_string(),
    ];
    docker_args.extend(extra_args.iter().cloned());
    docker_args.push(image.to_string());

    if dry_run {
        println!("Dry-run {} command: {} {}", engine, engine, docker_args.join(" "));
        return Ok(());
    }

    let status = Command::new(engine).args(&docker_args).status()?;
    if !status.success() {
        anyhow::bail!("{engine} run failed with status {status}");
    }

    println!("Protected run completed for image `{image}`");
    Ok(())
}
