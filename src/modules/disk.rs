use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::utils::error::Result;
use crate::utils::disk::{dir_size, count_files};
use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use sysinfo::Disks;
use rayon::prelude::*;

const KB: u64 = 1024;
const MB: u64 = 1024 * KB;
const GB: u64 = 1024 * MB;

pub struct DiskModule {
    min_size_large: u64,
}

impl DiskModule {
    pub fn new(min_size: u64) -> Self {
        DiskModule {
            min_size_large: min_size,
        }
    }

    pub fn scan_disk_overview(&self) -> Vec<CleanItem> {
        let mut items = Vec::new();
        let disks = Disks::new_with_refreshed_list();

        for disk in disks.list() {
            let mount = disk.mount_point().to_string_lossy().to_string();
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);

            if total > 0 {
                let usage_pct = (used as f64 / total as f64 * 100.0) as u32;
                items.push(CleanItem::new(
                    &format!("disk-{}", mount.replace('/', "-")),
                    &mount,
                    used,
                    false,
                    &format!("Disk: {} ({:.1}% used, {} free)", mount, usage_pct, format_size(available)),
                    &format!("# Read-only info for {}", mount),
                ));
            }
        }

        items
    }

    pub fn analyze_directory_tree(&self, home: &Path) -> Vec<CleanItem> {
        let mut items = Vec::new();
        let analysis_dirs = vec![
            (home.join("Downloads"), "Downloads", 50 * MB),
            (home.join("Documents"), "Documents", 20 * MB),
            (home.join("Videos"), "Videos", 100 * MB),
            (home.join("Music"), "Music", 50 * MB),
            (home.join("Pictures"), "Pictures", 20 * MB),
            (home.join(".local/share"), "AppData (.local/share)", 10 * MB),
            (home.join(".config"), "Config (.config)", 5 * MB),
            (home.join(".cache"), "Cache (.cache)", 20 * MB),
        ];

        for (dir, name, min_size) in analysis_dirs {
            if dir.exists() {
                let size = dir_size(&dir);
                if size >= min_size {
                    let file_count = count_files(&dir);
                    items.push(CleanItem::new(
                        &format!("tree-{}", name.to_lowercase().replace(' ', "-")),
                        &dir.to_string_lossy(),
                        size,
                        false,
                        &format!("{} ({} files)", name, file_count),
                        &format!("# Directory analysis: {}", name),
                    ).with_file_count(file_count));
                }
            }
        }

        items.sort_by(|a, b| b.size.cmp(&a.size));
        items
    }

    pub fn scan_by_file_type(&self, home: &Path) -> Vec<CleanItem> {
        let mut items = Vec::new();
        let type_configs = vec![
            FileTypeConfig {
                name: "Videos",
                category: "video",
                extensions: vec!["mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "mpeg", "mpg", "vob"],
                min_size: 50 * MB,
                search_dirs: vec![home.join("Downloads"), home.join("Videos"), home.join("Movies")],
            },
            FileTypeConfig {
                name: "Archives",
                category: "archive",
                extensions: vec!["zip", "tar", "gz", "bz2", "xz", "rar", "7z", "tgz"],
                min_size: 10 * MB,
                search_dirs: vec![home.join("Downloads"), home.join("Documents")],
            },
            FileTypeConfig {
                name: "ISO Images",
                category: "iso",
                extensions: vec!["iso", "img", "dmg"],
                min_size: 1 * MB,
                search_dirs: vec![home.join("Downloads"), home.join("Documents"), home.join("VMs")],
            },
            FileTypeConfig {
                name: "Development Builds",
                category: "build",
                extensions: vec!["o", "obj", "so", "a", "dylib"],
                min_size: 5 * MB,
                search_dirs: vec![home.join("target"), home.join(".cargo")],
            },
            FileTypeConfig {
                name: "Log Files",
                category: "log",
                extensions: vec!["log", "logs"],
                min_size: 1 * MB,
                search_dirs: vec![home.join(".cache"), std::path::Path::new("/var/log").to_path_buf()],
            },
            FileTypeConfig {
                name: "Backup Files",
                category: "backup",
                extensions: vec!["bak", "backup", "old", "orig", "swp", "swo"],
                min_size: 1 * MB,
                search_dirs: vec![home.join("Downloads"), home.join("Documents")],
            },
            FileTypeConfig {
                name: "Torrent Data",
                category: "torrent",
                extensions: vec!["torrent", "part"],
                min_size: 1 * MB,
                search_dirs: vec![home.join("Downloads")],
            },
            FileTypeConfig {
                name: "Audio",
                category: "audio",
                extensions: vec!["mp3", "flac", "wav", "aac", "ogg", "m4a", "wma"],
                min_size: 20 * MB,
                search_dirs: vec![home.join("Downloads"), home.join("Music")],
            },
        ];

        for config in type_configs {
            let files = self.scan_files_by_type(&config, home);
            items.extend(files);
        }

        items
    }

    fn scan_files_by_type(&self, config: &FileTypeConfig, _home: &Path) -> Vec<CleanItem> {
        let mut total_size: u64 = 0;
        let mut file_count: u64 = 0;
        let mut largest_path = String::new();
        let mut largest_size: u64 = 0;

        for dir in &config.search_dirs {
            if !dir.exists() {
                continue;
            }

            for entry in WalkDir::new(dir)
                .follow_links(false)
                .max_depth(6)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if !entry.file_type().is_file() {
                    continue;
                }

                if let Some(ext) = entry.path().extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if config.extensions.contains(&ext_str.as_str()) {
                        if let Ok(meta) = entry.metadata() {
                            let size = meta.len();
                            file_count += 1;
                            total_size += size;

                            if size > largest_size {
                                largest_size = size;
                                largest_path = entry.path().to_string_lossy().to_string();
                            }
                        }
                    }
                }
            }
        }

        if total_size >= config.min_size && file_count > 0 {
            vec![CleanItem::new(
                &format!("{}-{}", config.category, "bulk"),
                &largest_path,
                total_size,
                true,
                &format!("{} ({} files, largest: {})", config.name, file_count, format_size(largest_size)),
                &format!("# {} files grouped by type", config.name),
            ).with_category(&config.category).with_file_count(file_count)]
        } else {
            vec![]
        }
    }

    pub fn scan_by_age(&self, home: &Path) -> Vec<CleanItem> {
        let mut items = Vec::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let age_thresholds = vec![
            (7, "1 Week Old"),
            (30, "1 Month Old"),
            (90, "3 Months Old"),
            (365, "1 Year Old"),
        ];

        for (days, name) in age_thresholds {
            let threshold = now - (days * 86400);
            let mut old_files: Vec<(u64, String, u64)> = Vec::new();

            let scan_dirs = vec![
                home.join("Downloads"),
                home.join("Documents"),
            ];

            for dir in scan_dirs {
                if !dir.exists() {
                    continue;
                }

                for entry in WalkDir::new(&dir)
                    .follow_links(false)
                    .max_depth(3)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    if !entry.file_type().is_file() {
                        continue;
                    }

                    if let Ok(meta) = entry.metadata() {
                        if let Ok(modified) = meta.modified() {
                            let modified_secs = modified
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();

                            if modified_secs < threshold {
                                let size = meta.len();
                                if size > 10 * MB {
                                    let age = (now - modified_secs) / 86400;
                                    old_files.push((size, entry.path().to_string_lossy().to_string(), age));
                                }
                            }
                        }
                    }

                    if old_files.len() >= 50 {
                        break;
                    }
                }
            }

            if !old_files.is_empty() {
                let total_size: u64 = old_files.iter().map(|(s, _, _)| s).sum();
                old_files.sort_by(|a, b| b.0.cmp(&a.0));
                let sample = old_files.first().map(|(_, p, _)| p.clone()).unwrap_or_default();

                items.push(CleanItem::new(
                    &format!("age-{}-days", days),
                    &sample,
                    total_size,
                    true,
                    &format!("{} files ({} days old)", name, days),
                    &format!("# Old files from {} days ago", days),
                ).with_category("old").with_file_count(old_files.len() as u64));
            }
        }

        items
    }

    pub fn scan_empty_dirs(&self, home: &Path) -> Vec<CleanItem> {
        let mut items = Vec::new();

        for entry in WalkDir::new(home)
            .follow_links(false)
            .max_depth(4)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_dir() {
                if let Ok(mut entries) = fs::read_dir(entry.path()) {
                    if entries.next().is_none() {
                        items.push(CleanItem::new(
                            &format!("empty-{}", entry.file_name().to_string_lossy()),
                            &entry.path().to_string_lossy(),
                            0,
                            true,
                            "Empty directory",
                            &format!("rmdir '{}'", entry.path().display()),
                        ));
                    }
                }
            }
        }

        items.truncate(20);
        items
    }

    pub fn scan_broken_symlinks(&self, home: &Path) -> Vec<CleanItem> {
        let mut items = Vec::new();

        for entry in WalkDir::new(home)
            .follow_links(false)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_symlink() {
                if let Ok(meta) = entry.metadata() {
                    if meta.file_type().is_symlink() {
                        let target = fs::read_link(entry.path());
                        if target.is_err() || !target.as_ref().unwrap().exists() {
                            items.push(CleanItem::new(
                                &format!("broken-link-{}", entry.file_name().to_string_lossy()),
                                &entry.path().to_string_lossy(),
                                0,
                                true,
                                "Broken symlink",
                                &format!("rm '{}'", entry.path().display()),
                            ));
                        }
                    }
                }
            }
        }

        items
    }

    pub fn scan_system_cleanable(&self) -> Vec<CleanItem> {
        let mut items = Vec::new();

        let paths = vec![
            ("/var/cache/apt/archives", "APT Package Cache", 50 * MB, "sudo apt-get clean"),
            ("/var/cache/yum", "YUM/DNF Cache", 50 * MB, "sudo yum clean all"),
            ("/var/cache/pacman/pkg", "Pacman Package Cache", 50 * MB, "sudo pacman -Scc"),
            ("/var/tmp", "System Temp Files", 10 * MB, "sudo rm -rf /var/tmp/*"),
            ("/tmp", "User Temp Files", 50 * MB, "rm -rf /tmp/*"),
        ];

        for (path, desc, min_size, cmd) in paths {
            let p = Path::new(path);
            if p.exists() {
                let size = dir_size(p);
                if size >= min_size {
                    items.push(CleanItem::new(
                        &format!("sys-{}", path.replace('/', "-")),
                        path,
                        size,
                        true,
                        desc,
                        cmd,
                    ));
                }
            }
        }

        items
    }

    pub fn scan_trash(&self, home: &Path) -> Vec<CleanItem> {
        let mut items = Vec::new();

        let trash_paths = vec![
            (home.join(".local/share/Trash"), "User Trash"),
            (home.join(".local/share/Trash/files"), "User Trash Files"),
        ];

        for (trash, name) in trash_paths {
            if trash.exists() {
                let size = dir_size(&trash);
                let file_count = count_files(&trash);
                if size > 1024 * 1024 || file_count > 10 {
                    items.push(CleanItem::new(
                        &name.to_lowercase().replace(' ', "-"),
                        &trash.to_string_lossy(),
                        size,
                        true,
                        &format!("{} ({} files)", name, file_count),
                        &format!("rm -rf {}/*", trash.display()),
                    ));
                }
            }
        }

        items
    }

    pub fn scan_thumbnails(&self, home: &Path) -> Vec<CleanItem> {
        let mut items = Vec::new();
        let thumb_dirs = vec![
            home.join(".cache/thumbnails"),
            home.join(".thumbnails"),
        ];

        for thumb_dir in thumb_dirs {
            if thumb_dir.exists() {
                let size = dir_size(&thumb_dir);
                let file_count = count_files(&thumb_dir);
                if size > 512 * KB {
                    items.push(CleanItem::new(
                        "thumbnails",
                        &thumb_dir.to_string_lossy(),
                        size,
                        true,
                        &format!("Thumbnail cache ({} files)", file_count),
                        &format!("rm -rf {}/*", thumb_dir.display()),
                    ));
                }
            }
        }

        items
    }

    pub fn scan_expanded_packages(&self, home: &Path) -> Vec<CleanItem> {
        let mut items = Vec::new();

        let caches = vec![
            (home.join(".npm"), "npm Cache", 10 * MB),
            (home.join(".yarn"), "Yarn Cache", 10 * MB),
            (home.join(".cargo/registry/cache"), "Cargo Registry Cache", 10 * MB),
            (home.join(".cache/pip"), "pip Cache", 10 * MB),
            (home.join(".cache/go-build"), "Go Build Cache", 10 * MB),
            (home.join(".gradle/caches"), "Gradle Cache", 10 * MB),
            (home.join(".ivy2/cache"), "Ivy Cache", 10 * MB),
        ];

        for (path, name, min_size) in caches {
            if path.exists() {
                let size = dir_size(&path);
                if size >= min_size {
                    items.push(CleanItem::new(
                        &name.to_lowercase().replace(' ', "-"),
                        &path.to_string_lossy(),
                        size,
                        true,
                        name,
                        &format!("rm -rf {}/*", path.display()),
                    ));
                }
            }
        }

        items
    }

    pub fn scan_core_dumps(&self) -> Vec<CleanItem> {
        let mut items = Vec::new();
        let dump_dirs = vec!["/var/lib/systemd/coredump", "/tmp", "/var/crash"];

        for dir in dump_dirs {
            let path = Path::new(dir);
            if !path.exists() {
                continue;
            }

            for entry in WalkDir::new(path)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let name = entry.file_name().to_string_lossy();
                if name.starts_with("core.") || name.ends_with(".core") || name.ends_with(".crash") {
                    if let Ok(meta) = entry.metadata() {
                        items.push(CleanItem::new(
                            &format!("coredump-{}", name.replace('.', "-")),
                            &entry.path().to_string_lossy(),
                            meta.len(),
                            true,
                            "Core dump file",
                            &format!("rm -f '{}'", entry.path().display()),
                        ));
                    }
                }
            }
        }

        items
    }
}

struct FileTypeConfig {
    name: &'static str,
    category: &'static str,
    extensions: Vec<&'static str>,
    min_size: u64,
    search_dirs: Vec<std::path::PathBuf>,
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1} {}", size, UNITS[unit_idx])
}

impl CleanModule for DiskModule {
    fn id(&self) -> &'static str {
        "disk"
    }
    fn name(&self) -> &'static str {
        "Disk Usage"
    }
    fn category(&self) -> Category {
        Category::Disk
    }
    fn description(&self) -> &'static str {
        "Advanced disk analysis: directory tree, file types, age-based, containers"
    }

    fn scan(&self, _os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();

        items.extend(self.scan_disk_overview());

        if let Some(home) = dirs::home_dir() {
            items.extend(self.analyze_directory_tree(&home));
            items.extend(self.scan_by_file_type(&home));
            items.extend(self.scan_by_age(&home));
            items.extend(self.scan_empty_dirs(&home));
            items.extend(self.scan_broken_symlinks(&home));
            items.extend(self.scan_trash(&home));
            items.extend(self.scan_thumbnails(&home));
            items.extend(self.scan_expanded_packages(&home));
        }

        items.extend(self.scan_system_cleanable());
        items.extend(self.scan_core_dumps());

        Ok(items)
    }
}