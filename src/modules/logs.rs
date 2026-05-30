use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::utils::error::Result;
use crate::utils::disk::dir_size;
use std::path::Path;
use std::process::Command;
use std::fs;

pub struct LogsModule;

impl CleanModule for LogsModule {
    fn id(&self) -> &'static str {
        "logs"
    }
    fn name(&self) -> &'static str {
        "System Logs"
    }
    fn category(&self) -> Category {
        Category::System
    }
    fn description(&self) -> &'static str {
        "Clean old system logs, journal logs, and application logs"
    }
    fn is_available(&self, _os: &OSInfo) -> bool {
        Path::new("/var/log").exists() || dirs::home_dir().map(|p| p.join(".local/share").exists()).unwrap_or(false)
    }

    fn scan(&self, os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();

        if self.has_systemd() {
            items.extend(self.scan_journal_logs());
        }

        items.extend(self.scan_var_log());
        items.extend(self.scan_user_logs());

        Ok(items)
    }
}

impl LogsModule {
    fn has_systemd(&self) -> bool {
        Command::new("systemctl").arg("--version").output().is_ok()
    }

    fn scan_journal_logs(&self) -> Vec<CleanItem> {
        let mut items = Vec::new();

        let journal_path = Path::new("/var/log/journal");
        if journal_path.exists() {
            let size = dir_size(journal_path);
            if size > 1024 * 1024 {
                items.push(CleanItem::new(
                    "/var/log/journal",
                    "/var/log/journal",
                    size,
                    true,
                    "Journal logs (systemd)",
                    "journalctl --vacuum-size=100M",
                ));
            }
        }

        items
    }

    fn scan_var_log(&self) -> Vec<CleanItem> {
        let mut items = Vec::new();

        let log_dir = Path::new("/var/log");
        if let Ok(entries) = fs::read_dir(log_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(metadata) = entry.metadata() {
                        let size = metadata.len();
                        if size > 50 * 1024 * 1024 {
                            let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                            items.push(CleanItem::new(
                                &path.to_string_lossy(),
                                &path.to_string_lossy(),
                                size,
                                false,
                                &format!("Large log file: {}", name),
                                &format!("rm -f {}", path.display()),
                            ));
                        }
                    }
                }
            }
        }

        items
    }

    fn scan_user_logs(&self) -> Vec<CleanItem> {
        let mut items = Vec::new();

        if let Some(home) = dirs::home_dir() {
            let logs_dir = home.join(".local/share");
            if logs_dir.exists() {
                let size = dir_size(&logs_dir);
                if size > 5 * 1024 * 1024 {
                    items.push(CleanItem::new(
                        &logs_dir.to_string_lossy(),
                        &logs_dir.to_string_lossy(),
                        size,
                        true,
                        "User application logs",
                        &format!("rm -rf {}/*", logs_dir.display()),
                    ));
                }
            }
        }

        items
    }
}