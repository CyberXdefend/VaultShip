pub mod capabilities;
pub mod dockerfile;
pub mod readonly;
pub mod seccomp;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};

#[derive(Debug, Serialize, Deserialize)]
pub struct HardenConfig {
    pub read_only: bool,
    pub drop_capabilities: bool,
    pub no_new_privileges: bool,
    pub seccomp_profile: SeccompMode,
    pub anti_debug: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SeccompMode {
    Auto,
    Strict,
    Custom(String),
}

pub fn generate_hardened_compose(service_name: &str, config: &HardenConfig) -> Result<String> {
    let mut lines = vec![
        "services:".to_string(),
        format!("  {service_name}:"),
        format!("    read_only: {}", config.read_only),
    ];

    if config.drop_capabilities {
        lines.push("    cap_drop:".to_string());
        lines.push("      - ALL".to_string());
        lines.push("    cap_add:".to_string());
        lines.push("      - NET_BIND_SERVICE".to_string());
    }

    let seccomp = match &config.seccomp_profile {
        SeccompMode::Auto | SeccompMode::Strict => "./seccomp-profile.json".to_string(),
        SeccompMode::Custom(path) => path.clone(),
    };

    lines.push("    security_opt:".to_string());
    if config.no_new_privileges {
        lines.push("      - no-new-privileges:true".to_string());
    }
    lines.push(format!("      - seccomp:{seccomp}"));
    lines.push("    tmpfs:".to_string());
    lines.push("      - /tmp:noexec,nosuid".to_string());

    Ok(lines.join("\n") + "\n")
}

pub fn harden_compose_document(compose_content: &str, config: &HardenConfig) -> Result<String> {
    let mut doc: Value = serde_yaml::from_str(compose_content)?;
    let services_key = Value::String("services".to_string());
    let services = doc
        .as_mapping_mut()
        .and_then(|m| m.get_mut(&services_key))
        .and_then(Value::as_mapping_mut)
        .ok_or_else(|| anyhow::anyhow!("Compose file has no `services` section"))?;

    for (_name, service) in services.iter_mut() {
        let mut service_map = service.as_mapping().cloned().unwrap_or_else(Mapping::new);
        service_map.insert(
            Value::String("read_only".to_string()),
            Value::Bool(config.read_only),
        );

        if config.drop_capabilities {
            service_map.insert(
                Value::String("cap_drop".to_string()),
                Value::Sequence(vec![Value::String("ALL".to_string())]),
            );
            service_map.insert(
                Value::String("cap_add".to_string()),
                Value::Sequence(vec![Value::String("NET_BIND_SERVICE".to_string())]),
            );
        }

        let seccomp_path = match &config.seccomp_profile {
            SeccompMode::Auto | SeccompMode::Strict => "./seccomp-profile.json".to_string(),
            SeccompMode::Custom(path) => path.clone(),
        };
        let mut security = vec![Value::String(format!("seccomp:{seccomp_path}"))];
        if config.no_new_privileges {
            security.push(Value::String("no-new-privileges:true".to_string()));
        }
        service_map.insert(
            Value::String("security_opt".to_string()),
            Value::Sequence(security),
        );
        service_map.insert(
            Value::String("tmpfs".to_string()),
            Value::Sequence(vec![Value::String("/tmp:noexec,nosuid".to_string())]),
        );
        *service = Value::Mapping(service_map);
    }

    Ok(serde_yaml::to_string(&doc)?)
}

#[cfg(test)]
mod tests {
    use super::{HardenConfig, SeccompMode, generate_hardened_compose, harden_compose_document};

    #[test]
    fn generates_expected_hardening_fields() {
        let cfg = HardenConfig {
            read_only: true,
            drop_capabilities: true,
            no_new_privileges: true,
            seccomp_profile: SeccompMode::Auto,
            anti_debug: true,
        };
        let out = generate_hardened_compose("api", &cfg).expect("compose");
        assert!(out.contains("read_only: true"));
        assert!(out.contains("cap_drop:"));
        assert!(out.contains("no-new-privileges:true"));
        assert!(out.contains("seccomp:./seccomp-profile.json"));
    }

    #[test]
    fn hardens_full_compose_document() {
        let source = "services:\n  api:\n    image: demo:latest\n";
        let cfg = HardenConfig {
            read_only: true,
            drop_capabilities: true,
            no_new_privileges: true,
            seccomp_profile: SeccompMode::Auto,
            anti_debug: true,
        };
        let out = harden_compose_document(source, &cfg).expect("harden compose");
        assert!(out.contains("read_only: true"));
        assert!(out.contains("security_opt"));
    }
}
