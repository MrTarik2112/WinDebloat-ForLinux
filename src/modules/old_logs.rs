use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::utils::error::Result;
use std::path::Path;
use walkdir::WalkDir;

pub struct OldLogsModule {
    days_threshold: u64,
}

impl OldLogsModule {
    pub fn new(days: u64) -> Self {
        Self { days_threshold: days }
    }
}

impl Default for OldLogsModule {
    fn default() -> Self {
        Self::new(30)
    }
}

impl CleanModule for OldLogsModule {
    fn id(&self) -> &'static str { "old_logs" }
    fn name(&self) -> &'static str { "Old Logs" }
    fn category(&self) -> Category { Category::OldLogs }
    fn description(&self) -> &'static str {
        "Find and clean old log files from /var/log"
    }

    fn scan(&self, _os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();
        let log_paths = vec![
            "/var/log",
            "/var/cache",
        ];

        for log_base in log_paths {
            let base = Path::new(log_base);
            if !base.exists() {
                continue;
            }

            let mut old_log_count = 0u64;
            let mut old_log_size = 0u64;
            let mut rotated_count = 0u64;
            let mut rotated_size = 0u64;

            for entry in WalkDir::new(base)
                .max_depth(2)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(meta) = path.metadata() {
                        let name = path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();

                        // Rotated logs (.log.1, .log.2, etc.)
                        if name.ends_with(".1") || name.ends_with(".2") || 
                           name.ends_with(".gz") || name.contains(".old") {
                            rotated_count += 1;
                            rotated_size += meta.len();
                        }
                        // Check age for non-rotated logs
                        else if let Ok(modified) = meta.modified() {
                            let age = std::time::SystemTime::now()
                                .duration_since(modified)
                                .map(|d| d.as_secs() / 86400)
                                .unwrap_or(0);
                            
                            if age > self.days_threshold {
                                old_log_count += 1;
                                old_log_size += meta.len();
                            }
                        }
                    }
                }
            }

            if rotated_count > 0 {
                items.push(CleanItem::new(
                    &format!("rotated-logs-{}", log_base.trim_start_matches('/').replace('/', "-")),
                    log_base,
                    rotated_size,
                    true,
                    &format!("{} rotated log files", rotated_count),
                    &format!("sudo find {} -name '*.log.[0-9]*' -o -name '*.log.gz' -delete 2>/dev/null || true", log_base),
                ).with_file_count(rotated_count));
            }

            if old_log_count > 0 {
                items.push(CleanItem::new(
                    &format!("old-logs-{}", log_base.trim_start_matches('/').replace('/', "-")),
                    log_base,
                    old_log_size,
                    true,
                    &format!("{} old log files (>{} days)", old_log_count, self.days_threshold),
                    &format!("sudo find {} -type f -name '*.log' -mtime +{} -delete 2>/dev/null || true", log_base, self.days_threshold),
                ).with_file_count(old_log_count));
            }
        }

        Ok(items)
    }
}