use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::utils::error::Result;
use std::path::Path;

pub struct SSHKeysModule;

impl CleanModule for SSHKeysModule {
    fn id(&self) -> &'static str { "ssh_keys" }
    fn name(&self) -> &'static str { "SSH/GPG" }
    fn category(&self) -> Category { Category::Privacy }
    fn description(&self) -> &'static str {
        "Find old SSH keys, GPG keys, and authorized_keys cleanup"
    }

    fn scan(&self, _os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();
        if let Some(home) = dirs::home_dir() {
            let ssh_dir = home.join(".ssh");
            if ssh_dir.exists() {
                let known_hosts = ssh_dir.join("known_hosts");
                if known_hosts.exists() {
                    if let Ok(content) = std::fs::read_to_string(&known_hosts) {
                        let line_count = content.lines().count();
                        if line_count > 100 {
                            items.push(CleanItem::new(
                                "ssh-known-hosts-large",
                                &known_hosts.to_string_lossy(),
                                content.len() as u64,
                                false,
                                &format!("SSH known_hosts too large ({} hosts)", line_count),
                                &format!("# Review before cleaning: {}", known_hosts.display()),
                            ).with_category("ssh"));
                        }
                    }
                }

                let old_keys = vec![
                    ssh_dir.join("id_rsa.old"),
                    ssh_dir.join("id_dsa.old"),
                    ssh_dir.join("id_ecdsa.old"),
                    ssh_dir.join("id_ed25519.old"),
                    ssh_dir.join("authorized_keys.old"),
                ];
                for key in &old_keys {
                    if key.exists() {
                        if let Ok(meta) = key.metadata() {
                            items.push(CleanItem::new(
                                &format!("ssh-old-{}", key.file_name().unwrap_or_default().to_string_lossy()),
                                &key.to_string_lossy(),
                                meta.len(),
                                false,
                                "Old SSH key backup",
                                &format!("rm -f '{}'", key.display()),
                            ).with_category("ssh"));
                        }
                    }
                }
            }

            let gnupg_dir = home.join(".gnupg");
            if gnupg_dir.exists() {
                let secring = gnupg_dir.join("secring.gpg");
                if secring.exists() {
                    if let Ok(meta) = secring.metadata() {
                        items.push(CleanItem::new(
                            "gpg-secring-old",
                            &secring.to_string_lossy(),
                            meta.len(),
                            false,
                            "Old GPG secret ring (if using gpg2)",
                            &format!("rm -f '{}'", secring.display()),
                        ).with_category("ssh"));
                    }
                }
            }
        }
        Ok(items)
    }
}
