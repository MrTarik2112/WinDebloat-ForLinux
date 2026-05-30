use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::utils::error::Result;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

pub struct OldDownloadsModule {
    days_threshold: u64,
}

impl OldDownloadsModule {
    pub fn new(days: u64) -> Self {
        Self { days_threshold: days }
    }
}

impl Default for OldDownloadsModule {
    fn default() -> Self {
        Self::new(30)
    }
}

impl CleanModule for OldDownloadsModule {
    fn id(&self) -> &'static str { "old_downloads" }
    fn name(&self) -> &'static str { "Old Downloads" }
    fn category(&self) -> Category { Category::OldDownloads }
    fn description(&self) -> &'static str {
        "Find files older than 30 days in Downloads folder"
    }

    fn scan(&self, _os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let threshold_seconds = self.days_threshold * 24 * 60 * 60;
        let cutoff = now - threshold_seconds;

        if let Some(home) = dirs::home_dir() {
            let downloads = home.join("Downloads");
            if downloads.exists() {
                let mut old_files_count = 0u64;
                let mut old_files_size = 0u64;
                let mut recent_files: Vec<(u64, u64, String)> = Vec::new();

                for entry in WalkDir::new(&downloads)
                    .max_depth(2)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    if entry.file_type().is_file() {
                        if let Ok(meta) = entry.metadata() {
                            if let Ok(modified) = meta.modified() {
                                let modified_secs = modified
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();
                                
                                if modified_secs < cutoff {
                                    old_files_count += 1;
                                    old_files_size += meta.len();
                                } else {
                                    recent_files.push((meta.len(), modified_secs, entry.path().to_string_lossy().to_string()));
                                }
                            }
                        }
                    }
                }

                if old_files_count > 0 {
                    let description = format!(
                        "{} files older than {} days ({} total)",
                        old_files_count,
                        self.days_threshold,
                        crate::utils::formatting::format_bytes(old_files_size)
                    );
                    
                    items.push(CleanItem::new(
                        "old-downloads",
                        &downloads.to_string_lossy(),
                        old_files_size,
                        true,
                        &description,
                        &format!(
                            "find {} -type f -mtime +{} -delete 2>/dev/null",
                            downloads.display(),
                            self.days_threshold
                        ),
                    )
                    .with_file_count(old_files_count)
                    .with_age(self.days_threshold));
                }

                // Also check for large recent files (> 100MB)
                let large_recent: Vec<_> = recent_files
                    .iter()
                    .filter(|(size, _, _)| *size > 100 * 1024 * 1024)
                    .collect();

                if !large_recent.is_empty() {
                    let total_size: u64 = large_recent.iter().map(|(s, _, _)| *s).sum();
                    items.push(CleanItem::new(
                        "large-downloads",
                        &downloads.to_string_lossy(),
                        total_size,
                        false,
                        &format!("{} large recent files (>100MB)", large_recent.len()),
                        "Manual review recommended",
                    )
                    .with_file_count(large_recent.len() as u64));
                }
            }
        }

        // Also check other common download locations
        let other_downloads = vec![
            "/home/Downloads",
            "/home/Загрузки",
            "/home/Download",
        ];

        for path_str in other_downloads {
            let path = Path::new(path_str);
            if path.exists() && path != dirs::home_dir().map(|h| h.join("Downloads")).unwrap_or_default() {
                if let Ok(meta) = path.metadata() {
                    if meta.is_dir() {
                        let size = crate::utils::disk::dir_size(path);
                        if size > 50 * 1024 * 1024 {
                            items.push(CleanItem::new(
                                "other-downloads",
                                path_str,
                                size,
                                false,
                                "Alternative Downloads folder",
                                &format!("rm -rf {}/*", path_str),
                            ));
                        }
                    }
                }
            }
        }

        Ok(items)
    }
}