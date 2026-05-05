use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HardwareFingerprint {
    pub mac_addresses: Vec<String>,
    pub disk_serial: Option<String>,
    pub hostname: String,
    pub cpu_id: String,
    pub composite_hash: String,
}

impl HardwareFingerprint {
    pub fn collect() -> Result<Self> {
        let mac_addresses = collect_mac_addresses();
        let disk_serial = collect_disk_serial();
        let hostname = hostname::get()?.to_string_lossy().to_string();
        let cpu_id = collect_cpu_id();

        let mut hasher = Sha256::new();
        for mac in &mac_addresses {
            hasher.update(mac.as_bytes());
        }
        if let Some(ref serial) = disk_serial {
            hasher.update(serial.as_bytes());
        }
        hasher.update(hostname.as_bytes());
        hasher.update(cpu_id.as_bytes());

        Ok(Self {
            mac_addresses,
            disk_serial,
            hostname,
            cpu_id,
            composite_hash: format!("{:x}", hasher.finalize()),
        })
    }

    pub fn verify_current(&self) -> Result<bool> {
        let current = Self::collect()?;
        let mut matches = 0;
        if self
            .mac_addresses
            .iter()
            .any(|m| current.mac_addresses.contains(m))
        {
            matches += 1;
        }
        if self.disk_serial.is_some() && self.disk_serial == current.disk_serial {
            matches += 1;
        }
        if self.hostname == current.hostname {
            matches += 1;
        }
        if self.cpu_id == current.cpu_id {
            matches += 1;
        }
        Ok(matches >= 2)
    }
}

fn collect_mac_addresses() -> Vec<String> {
    let mut addrs = Vec::new();
    if let Ok(Some(ma)) = mac_address::get_mac_address() {
        addrs.push(ma.to_string());
    }
    if addrs.is_empty() {
        addrs.push("unknown-mac".to_string());
    }
    addrs
}

/// Attempts to read a real disk serial number via OS tools.
/// Returns `None` rather than a device path when the serial cannot be determined.
fn collect_disk_serial() -> Option<String> {
    #[cfg(target_os = "linux")]
    return collect_disk_serial_linux();

    #[cfg(target_os = "macos")]
    return collect_disk_serial_macos();

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    None
}

#[cfg(target_os = "linux")]
fn collect_disk_serial_linux() -> Option<String> {
    let output = std::process::Command::new("lsblk")
        .args(["--nodeps", "--output", "SERIAL", "--noheadings"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    text.lines()
        .map(str::trim)
        .find(|s| !s.is_empty())
        .map(str::to_string)
}

#[cfg(target_os = "macos")]
fn collect_disk_serial_macos() -> Option<String> {
    let output = std::process::Command::new("system_profiler")
        .args(["SPStorageDataType", "-json"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    v["SPStorageDataType"].as_array()?.iter().find_map(|item| {
        let serial = item["volume_uuid"]
            .as_str()
            .or_else(|| item["_rowIdentifier"].as_str())?;
        if serial.is_empty() {
            None
        } else {
            Some(serial.to_string())
        }
    })
}

fn collect_cpu_id() -> String {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_cpu_all();
    if let Some(cpu) = sys.cpus().first() {
        return format!("{}-{}", cpu.brand(), cpu.vendor_id());
    }
    "unknown-cpu".to_string()
}
