use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultShipConfig {
    pub profile: Option<Profile>,
    pub project: Project,
    pub harden: Harden,
    pub bind: Option<Bind>,
    pub license: Option<License>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Profile { pub name: String }
#[derive(Debug, Serialize, Deserialize)]
pub struct Project { pub name: String }
#[derive(Debug, Serialize, Deserialize)]
pub struct Harden {
    pub read_only: bool,
    pub drop_capabilities: bool,
    pub no_new_privileges: bool,
    pub seccomp_profile: String,
    pub anti_debug: bool,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct License { pub product: String, pub public_key_path: String }
#[derive(Debug, Serialize, Deserialize)]
pub struct Bind {
    pub enabled: bool,
    pub hardware_required: bool,
    pub public_key_path: String,
}

pub fn load(path: &str) -> Result<VaultShipConfig> {
    let content = std::fs::read_to_string(path)?;
    Ok(toml::from_str::<VaultShipConfig>(&content)?)
}
