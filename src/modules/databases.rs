use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::utils::error::Result;
use crate::utils::disk::dir_size;
use std::path::Path;

pub struct DatabasesModule;

impl CleanModule for DatabasesModule {
    fn id(&self) -> &'static str { "databases" }
    fn name(&self) -> &'static str { "Databases" }
    fn category(&self) -> Category { Category::System }
    fn description(&self) -> &'static str {
        "Clean database logs, slow queries, and temporary files"
    }

    fn scan(&self, _os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();

        let paths = vec![
            ("/var/log/mysql", "MySQL log files"),
            ("/var/log/mariadb", "MariaDB log files"),
            ("/var/log/postgresql", "PostgreSQL log files"),
            ("/var/log/mongodb", "MongoDB log files"),
            ("/var/log/redis", "Redis log files"),
            ("/var/cache/mysql", "MySQL cache"),
        ];

        for (path, desc) in &paths {
            let p = Path::new(path);
            if p.exists() {
                let size = dir_size(p);
                if size > 5 * 1024 * 1024 {
                    items.push(CleanItem::new(
                        &format!("db-{}", path.replace('/', "-")),
                        path,
                        size,
                        true,
                        desc,
                        &format!("sudo find {} -name '*.log' -mtime +7 -delete 2>/dev/null", path),
                    ).with_category("databases"));
                }
            }
        }

        if let Some(home) = dirs::home_dir() {
            let sqlite_wals = vec![
                home.join(".local/share"),
                home.join(".config"),
            ];
            for dir in &sqlite_wals {
                if dir.exists() {
                    if let Ok(entries) = std::fs::read_dir(dir) {
                        for entry in entries.filter_map(|e| e.ok()) {
                            let path = entry.path();
                            if let Some(ext) = path.extension() {
                                if ext == "wal" || ext == "shm" {
                                    if let Ok(meta) = path.metadata() {
                                        if meta.len() > 1024 * 1024 {
                                            items.push(CleanItem::new(
                                                &format!("sqlite-{}", path.file_name().unwrap_or_default().to_string_lossy()),
                                                &path.to_string_lossy(),
                                                meta.len(),
                                                true,
                                                "SQLite WAL/SHM file",
                                                &format!("rm -f '{}'", path.display()),
                                            ).with_category("databases"));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(items)
    }
}
