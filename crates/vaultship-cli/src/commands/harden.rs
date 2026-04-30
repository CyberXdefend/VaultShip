use std::path::Path;
use vaultship_harden::{HardenConfig, SeccompMode, harden_compose_document};

pub async fn run(compose_file: &str) -> anyhow::Result<()> {
    if !Path::new(compose_file).exists() {
        anyhow::bail!("Compose file not found: {compose_file}");
    }

    let config = HardenConfig {
        read_only: true,
        drop_capabilities: true,
        no_new_privileges: true,
        seccomp_profile: SeccompMode::Auto,
        anti_debug: true,
    };

    let compose = std::fs::read_to_string(compose_file)?;
    let hardened = harden_compose_document(&compose, &config)?;
    std::fs::write("docker-compose.hardened.yml", hardened)?;

    let profile = vaultship_harden::seccomp::generate_anti_extraction_profile();
    vaultship_harden::seccomp::write_profile(&profile, "seccomp-profile.json")?;

    println!("Generated docker-compose.hardened.yml and seccomp-profile.json");
    Ok(())
}
