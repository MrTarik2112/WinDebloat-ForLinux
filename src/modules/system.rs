use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::utils::error::Result;
use std::fs;
use std::path::Path;

pub struct SystemModule;

fn journal_size() -> u64 {
    let path = Path::new("/var/log/journal");
    if !path.exists() {
        let path2 = Path::new("/run/systemd/journal");
        if path2.exists() {
            return 50 * 1024 * 1024;
        }
        return 0;
    }
    crate::utils::disk::dir_size(path)
}

impl CleanModule for SystemModule {
    fn id(&self) -> &'static str { "system" }
    fn name(&self) -> &'static str { "System" }
    fn category(&self) -> Category { Category::System }
    fn description(&self) -> &'static str {
        "Clean system logs, journal, temporary files, trash, and caches"
    }

    fn scan(&self, _os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();

        // Journal logs
        let j_size = journal_size();
        if j_size > 10 * 1024 * 1024 {
            items.push(CleanItem::new(
                "journal-logs",
                "/var/log/journal",
                j_size,
                true,
                "Systemd journal logs (rotated)",
                "journalctl --vacuum-time=7d",
            ));
        }

        // /var/log old files
        if let Ok(meta) = fs::metadata("/var/log") {
            if meta.is_dir() {
                let log_size = crate::utils::disk::dir_size(Path::new("/var/log"));
                if log_size > 50 * 1024 * 1024 {
                    items.push(CleanItem::new(
                        "var-log",
                        "/var/log",
                        log_size,
                        true,
                        "System log files",
                        "sudo find /var/log -name '*.log' -mtime +7 -delete 2>/dev/null; sudo journalctl --vacuum-time=7d",
                    ));
                }
            }
        }

        // /tmp cleanup
        let tmp_size = crate::utils::disk::dir_size(Path::new("/tmp"));
        if tmp_size > 10 * 1024 * 1024 {
            items.push(CleanItem::new(
                "tmp-files",
                "/tmp",
                tmp_size,
                true,
                "Temporary files",
                "rm -rf /tmp/*",
            ));
        }

        // User cache
        if let Some(home) = dirs::home_dir() {
            let cache_dir = home.join(".cache");
            if cache_dir.exists() {
                let cache_size = crate::utils::disk::dir_size(&cache_dir);
                if cache_size > 50 * 1024 * 1024 {
                    items.push(CleanItem::new(
                        "user-cache",
                        &cache_dir.to_string_lossy(),
                        cache_size,
                        true,
                        "User cache (~/.cache)",
                        &format!("rm -rf {}/*", cache_dir.display()),
                    ));
                }
            }

            // Thumbnails
            let thumb_dir = home.join(".cache/thumbnails");
            if thumb_dir.exists() {
                let thumb_size = crate::utils::disk::dir_size(&thumb_dir);
                if thumb_size > 1024 * 1024 {
                    items.push(CleanItem::new(
                        "thumbnails",
                        &thumb_dir.to_string_lossy(),
                        thumb_size,
                        true,
                        "Thumbnail cache",
                        &format!("rm -rf {}/*", thumb_dir.display()),
                    ));
                }
            }

            // Trash
            let trash_dir = home.join(".local/share/Trash");
            if trash_dir.exists() {
                let trash_size = crate::utils::disk::dir_size(&trash_dir);
                if trash_size > 1024 * 1024 {
                    items.push(CleanItem::new(
                        "user-trash",
                        &trash_dir.to_string_lossy(),
                        trash_size,
                        true,
                        "Trash directory",
                        &format!("rm -rf {}/*", trash_dir.display()),
                    ));
                }
            }
        }

        Ok(items)
    }
}