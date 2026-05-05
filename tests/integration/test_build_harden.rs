use vaultship_harden::{
    HardenConfig, SeccompMode, generate_hardened_compose, harden_compose_document,
};

fn default_config() -> HardenConfig {
    HardenConfig {
        read_only: true,
        drop_capabilities: true,
        no_new_privileges: true,
        seccomp_profile: SeccompMode::Auto,
        anti_debug: true,
    }
}

#[test]
fn generates_read_only_flag() {
    let out = generate_hardened_compose("api", &default_config()).expect("generate");
    assert!(out.contains("read_only: true"));
}

#[test]
fn generates_cap_drop_all() {
    let out = generate_hardened_compose("api", &default_config()).expect("generate");
    assert!(out.contains("cap_drop:"));
    assert!(out.contains("- ALL"));
}

#[test]
fn generates_no_new_privileges() {
    let out = generate_hardened_compose("api", &default_config()).expect("generate");
    assert!(out.contains("no-new-privileges:true"));
}

#[test]
fn generates_seccomp_path() {
    let out = generate_hardened_compose("api", &default_config()).expect("generate");
    assert!(out.contains("seccomp:./seccomp-profile.json"));
}

#[test]
fn harden_document_preserves_image() {
    let source = "services:\n  api:\n    image: myapp:v1\n";
    let out = harden_compose_document(source, &default_config()).expect("harden");
    assert!(out.contains("myapp:v1"));
    assert!(out.contains("read_only: true"));
    assert!(out.contains("security_opt"));
}

#[test]
fn custom_seccomp_path_is_used() {
    let cfg = HardenConfig {
        seccomp_profile: SeccompMode::Custom("/custom/profile.json".to_string()),
        ..default_config()
    };
    let out = generate_hardened_compose("web", &cfg).expect("generate");
    assert!(out.contains("seccomp:/custom/profile.json"));
}

#[test]
fn drop_capabilities_disabled() {
    let cfg = HardenConfig {
        drop_capabilities: false,
        ..default_config()
    };
    let out = generate_hardened_compose("api", &cfg).expect("generate");
    assert!(!out.contains("cap_drop:"));
    assert!(!out.contains("cap_add:"));
}

#[test]
fn seccomp_profile_anti_extraction_syscall_count() {
    let profile = vaultship_harden::seccomp::generate_anti_extraction_profile();
    // Must have allow and deny rules
    assert!(profile.syscalls.len() >= 2);
    let allow_rule = profile
        .syscalls
        .iter()
        .find(|r| r.action == "SCMP_ACT_ALLOW");
    let deny_rule = profile
        .syscalls
        .iter()
        .find(|r| r.action == "SCMP_ACT_ERRNO");
    assert!(allow_rule.is_some(), "must have an ALLOW rule");
    assert!(deny_rule.is_some(), "must have a DENY rule");
    // ptrace must be blocked
    let deny_names: Vec<&str> = deny_rule
        .unwrap()
        .names
        .iter()
        .map(String::as_str)
        .collect();
    assert!(
        deny_names.contains(&"ptrace"),
        "ptrace must be in deny list"
    );
}
