use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SeccompProfile {
    #[serde(rename = "defaultAction")]
    pub default_action: String,
    pub architectures: Vec<String>,
    pub syscalls: Vec<SyscallRule>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyscallRule {
    pub names: Vec<String>,
    pub action: String,
}

pub fn generate_anti_extraction_profile() -> SeccompProfile {
    SeccompProfile {
        default_action: "SCMP_ACT_ERRNO".to_string(),
        architectures: vec![
            "SCMP_ARCH_X86_64".to_string(),
            "SCMP_ARCH_AARCH64".to_string(),
        ],
        syscalls: vec![
            SyscallRule {
                names: vec![
                    "read",
                    "write",
                    "close",
                    "openat",
                    "newfstatat",
                    "mmap",
                    "munmap",
                    "brk",
                    "socket",
                    "bind",
                    "listen",
                    "accept",
                    "connect",
                    "sendto",
                    "recvfrom",
                    "epoll_wait",
                    "epoll_ctl",
                    "epoll_create1",
                    "clock_gettime",
                    "futex",
                    "exit",
                    "exit_group",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                action: "SCMP_ACT_ALLOW".to_string(),
            },
            SyscallRule {
                names: vec![
                    "ptrace",
                    "process_vm_readv",
                    "process_vm_writev",
                    "mount",
                    "umount2",
                    "pivot_root",
                    "chroot",
                    "kexec_load",
                    "init_module",
                    "finit_module",
                    "delete_module",
                    "reboot",
                    "sethostname",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                action: "SCMP_ACT_ERRNO".to_string(),
            },
        ],
    }
}

pub fn write_profile(profile: &SeccompProfile, path: &str) -> Result<()> {
    let json = serde_json::to_string_pretty(profile)?;
    std::fs::write(path, json)?;
    Ok(())
}
