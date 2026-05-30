use crate::utils::formatting::format_bytes;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CleanItem {
    pub id: String,
    pub path: String,
    pub size: u64,
    pub safe: bool,
    pub description: String,
    pub clean_command: String,
    pub selected: bool,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub age_days: Option<u64>,
    #[serde(default)]
    pub file_count: Option<u64>,
}

impl CleanItem {
    pub fn new(
        id: &str,
        path: &str,
        size: u64,
        safe: bool,
        description: &str,
        clean_command: &str,
    ) -> Self {
        CleanItem {
            id: id.to_string(),
            path: path.to_string(),
            size,
            safe,
            description: description.to_string(),
            clean_command: clean_command.to_string(),
            selected: false,
            category: String::new(),
            age_days: None,
            file_count: None,
        }
    }

    pub fn with_category(mut self, category: &str) -> Self {
        self.category = category.to_string();
        self
    }

    pub fn with_age(mut self, days: u64) -> Self {
        self.age_days = Some(days);
        self
    }

    pub fn with_file_count(mut self, count: u64) -> Self {
        self.file_count = Some(count);
        self
    }

    pub fn size_formatted(&self) -> String {
        format_bytes(self.size)
    }

    pub fn display_path(&self) -> String {
        if self.path.len() > 50 {
            let home = dirs::home_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            self.path.replace(&home, "~")
        } else {
            self.path.clone()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    Packages,
    System,
    Apps,
    Privacy,
    Services,
    Duplicates,
    Containers,
    Disk,
    Wizard,
    BrokenLinks,
    EmptyFiles,
    OldDownloads,
    OldLogs,
    Fonts,
    Kernel,
    DevTools,
    AutoClean,
}

impl Category {
    pub fn name(&self) -> &str {
        match self {
            Category::Packages => "Packages",
            Category::System => "System",
            Category::Apps => "Applications",
            Category::Privacy => "Privacy",
            Category::Services => "Services",
            Category::Duplicates => "Duplicates",
            Category::Containers => "Containers",
            Category::Disk => "Disk Usage",
            Category::Wizard => "Disk Wizard",
            Category::BrokenLinks => "Broken Links",
            Category::EmptyFiles => "Empty Files",
            Category::OldDownloads => "Old Downloads",
            Category::OldLogs => "Old Logs",
            Category::Fonts => "Fonts",
            Category::Kernel => "Kernel",
            Category::DevTools => "Dev Tools",
            Category::AutoClean => "Auto Clean",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Category::Packages => "\u{f487}",
            Category::System => "\u{f0e4}",
            Category::Apps => "\u{f121}",
            Category::Privacy => "\u{f023}",
            Category::Services => "\u{f013}",
            Category::Duplicates => "\u{f0c5}",
            Category::Containers => "\u{f20a}",
            Category::Disk => "\u{f0a1}",
            Category::Wizard => "\u{f0a1}",
            Category::BrokenLinks => "\u{f128}",
            Category::EmptyFiles => "\u{f0c5}",
            Category::OldDownloads => "\u{f019}",
            Category::OldLogs => "\u{f0ce}",
            Category::Fonts => "\u{f031}",
            Category::Kernel => "\u{f0e2}",
            Category::DevTools => "\u{f121}",
            Category::AutoClean => "\u{1f680}",
        }
    }

    pub fn all() -> &'static [Category] {
        &[
            Category::Packages,
            Category::System,
            Category::Apps,
            Category::Privacy,
            Category::Services,
            Category::Duplicates,
            Category::Containers,
            Category::Disk,
            Category::Wizard,
            Category::BrokenLinks,
            Category::EmptyFiles,
            Category::OldDownloads,
            Category::OldLogs,
            Category::Fonts,
            Category::Kernel,
            Category::DevTools,
            Category::AutoClean,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub category: Category,
    pub items: Vec<CleanItem>,
    pub total_size: u64,
    pub total_items: usize,
}

impl ScanResult {
    pub fn new(category: Category, items: Vec<CleanItem>) -> Self {
        let total_size = items.iter().map(|i| i.size).sum();
        let total_items = items.len();
        ScanResult {
            category,
            items,
            total_size,
            total_items,
        }
    }

    pub fn total_size_formatted(&self) -> String {
        format_bytes(self.total_size)
    }
}

pub struct Scanner;

impl Scanner {
    pub fn new() -> Self {
        Scanner
    }

    pub fn scan_path(
        &self,
        path: &str,
        min_size: u64,
        pattern: &str,
        safe: bool,
        description: &str,
        id_prefix: &str,
    ) -> Vec<CleanItem> {
        let path_obj = std::path::Path::new(path);
        if !path_obj.exists() {
            return vec![];
        }

        let items = if pattern.is_empty() {
            let size = crate::utils::disk::dir_size(path_obj);
            if size > min_size {
                vec![CleanItem::new(
                    &format!("{}-{}", id_prefix, path.replace('/', "-")),
                    path,
                    size,
                    safe,
                    description,
                    &format!("rm -rf {}", path),
                )]
            } else {
                vec![]
            }
        } else {
            self.scan_with_pattern(path, pattern, id_prefix)
        };

        items
    }

    fn scan_with_pattern(&self, _path: &str, _pattern: &str, _id_prefix: &str) -> Vec<CleanItem> {
        let mut items = Vec::new();
        if let Ok(entries) = glob::glob(_pattern) {
            for entry in entries.flatten() {
                if entry.is_file() || entry.is_dir() {
                    let meta = entry.metadata().ok();
                    let size = meta.map(|m| m.len()).unwrap_or(0);
                    items.push(CleanItem::new(
                        &format!("{}-{}", _id_prefix, entry.to_string_lossy().replace('/', "-")),
                        &entry.to_string_lossy(),
                        size,
                        true,
                        &format!("Matched pattern: {}", _pattern),
                        &format!("rm -rf '{}'", entry.display()),
                    ));
                }
            }
        }
        items
    }
}
