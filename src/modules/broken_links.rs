use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::utils::error::Result;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub struct BrokenLinksModule;

impl CleanModule for BrokenLinksModule {
    fn id(&self) -> &'static str { "broken_links" }
    fn name(&self) -> &'static str { "Broken Links" }
    fn category(&self) -> Category { Category::BrokenLinks }
    fn description(&self) -> &'static str {
        "Find and remove broken symbolic links"
    }

    fn scan(&self, _os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();
        let search_paths = vec![
            dirs::home_dir().map(|p| p.to_path_buf()).unwrap_or_default(),
            Path::new("/tmp").to_path_buf(),
            Path::new("/var").to_path_buf(),
        ];

        let mut broken_count = 0u64;
        let mut total_size = 0u64;

        for base_path in search_paths.iter().filter(|p| p.exists()) {
            for entry in WalkDir::new(base_path)
                .follow_links(false)
                .max_depth(5)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_symlink() {
                    if let Ok(meta) = entry.metadata() {
                        if meta.file_type().is_symlink() {
                            let target = fs::read_link(entry.path());
                            if target.is_err() || !target.unwrap().exists() {
                                broken_count += 1;
                                if let Ok(lnk_meta) = entry.metadata() {
                                    total_size += lnk_meta.len();
                                }
                            }
                        }
                    }
                }
            }
        }

        if broken_count > 0 {
            items.push(CleanItem::new(
                "broken-symlinks-home",
                &dirs::home_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
                total_size,
                true,
                &format!("{} broken symbolic links found", broken_count),
                "find ~ -type l ! -exec test -e {} \\; -print -delete",
            ).with_file_count(broken_count));
        }

        Ok(items)
    }
}