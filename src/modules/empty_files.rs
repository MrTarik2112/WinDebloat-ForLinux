use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::utils::error::Result;
use std::path::Path;
use walkdir::WalkDir;

pub struct EmptyFilesModule;

impl CleanModule for EmptyFilesModule {
    fn id(&self) -> &'static str { "empty_files" }
    fn name(&self) -> &'static str { "Empty Files" }
    fn category(&self) -> Category { Category::EmptyFiles }
    fn description(&self) -> &'static str {
        "Find and remove empty files and directories"
    }

    fn scan(&self, _os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();
        
        if let Some(home) = dirs::home_dir() {
            let (empty_files, empty_dirs) = scan_empty_items(&home);
            
            if !empty_files.is_empty() {
                items.push(CleanItem::new(
                    "empty-files",
                    &home.to_string_lossy(),
                    0,
                    true,
                    &format!("{} empty files", empty_files.len()),
                    &format!("find {} -type f -empty -delete 2>/dev/null", home.display()),
                ).with_file_count(empty_files.len() as u64));
            }

            if !empty_dirs.is_empty() {
                items.push(CleanItem::new(
                    "empty-dirs",
                    &home.to_string_lossy(),
                    0,
                    true,
                    &format!("{} empty directories", empty_dirs.len()),
                    &format!("find {} -type d -empty -delete 2>/dev/null", home.display()),
                ).with_file_count(empty_dirs.len() as u64));
            }
        }

        // System tmp directories
        let tmp_path = Path::new("/tmp");
        if tmp_path.exists() {
            let (empty_files, empty_dirs) = scan_empty_items(tmp_path);
            let total = empty_files.len() + empty_dirs.len();
            if total > 10 {
                items.push(CleanItem::new(
                    "empty-tmp",
                    "/tmp",
                    0,
                    true,
                    &format!("{} empty items in /tmp", total),
                    "find /tmp -empty -delete 2>/dev/null",
                ).with_file_count(total as u64));
            }
        }

        Ok(items)
    }
}

fn scan_empty_items(base: &Path) -> (Vec<u64>, Vec<u64>) {
    let mut empty_files = Vec::new();
    let mut empty_dirs = Vec::new();

    for entry in WalkDir::new(base)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path == base {
            continue;
        }

        if entry.file_type().is_file() {
            if let Ok(meta) = path.metadata() {
                if meta.len() == 0 {
                    empty_files.push(0);
                }
            }
        } else if entry.file_type().is_dir() {
            if let Ok(meta) = path.metadata() {
                if let Ok(entries) = std::fs::read_dir(path) {
                    if entries.count() == 0 {
                        empty_dirs.push(0);
                    }
                }
            }
        }
    }

    (empty_files, empty_dirs)
}