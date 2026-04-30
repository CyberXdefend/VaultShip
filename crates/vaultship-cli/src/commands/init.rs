pub async fn run(profile: &str) -> anyhow::Result<()> {
    let template = match profile {
        "baseline" => include_str!("../../../../vaultship.toml.example").to_string(),
        "strict" => strict_template(),
        "high-assurance" => high_assurance_template(),
        other => anyhow::bail!("Unknown profile `{other}`. Use baseline|strict|high-assurance"),
    };

    std::fs::write("vaultship.toml", template)?;
    println!("Created vaultship.toml using `{profile}` profile.");
    Ok(())
}

fn strict_template() -> String {
    r#"[profile]
name = "strict"

[project]
name = "vaultship-demo"

[harden]
read_only = true
drop_capabilities = true
no_new_privileges = true
seccomp_profile = "strict"
anti_debug = true

[bind]
enabled = true
hardware_required = true
public_key_path = "./vaultship.public.key"
"#
    .to_string()
}

fn high_assurance_template() -> String {
    r#"[profile]
name = "high-assurance"

[project]
name = "vaultship-demo"

[harden]
read_only = true
drop_capabilities = true
no_new_privileges = true
seccomp_profile = "strict"
anti_debug = true

[bind]
enabled = true
hardware_required = true
public_key_path = "./vaultship.public.key"

[attestation]
enabled = true
backend = "coco"
require_attested_key_release = true
"#
    .to_string()
}
