use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub safe: SafeConfig,
    #[serde(default)]
    pub whitelist: WhitelistConfig,
    #[serde(default)]
    pub blacklist: BlacklistConfig,
    #[serde(default)]
    pub modules: ModulesConfig,
    #[serde(default)]
    pub backup: BackupConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub auto_clean: AutoCleanConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AutoCleanConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_auto_clean_profile")]
    pub profile: String,
    #[serde(default = "default_true")]
    pub safe_only: bool,
    #[serde(default)]
    pub benchmark: bool,
    #[serde(default)]
    pub predict: bool,
    #[serde(default)]
    pub scheduler: bool,
    #[serde(default = "default_auto_clean_categories")]
    pub categories: Vec<String>,
    #[serde(default)]
    pub exclude_categories: Vec<String>,
    #[serde(default = "default_auto_clean_min_size")]
    pub min_size: u64,
    #[serde(default = "default_auto_clean_max_size")]
    pub max_size: u64,
    #[serde(default = "default_auto_clean_report_format")]
    pub report_format: String,
    #[serde(default)]
    pub report_output: Option<String>,
}

fn default_auto_clean_profile() -> String { "balanced".to_string() }
fn default_auto_clean_categories() -> Vec<String> { vec!["all".to_string()] }
fn default_auto_clean_min_size() -> u64 { 1024 }
fn default_auto_clean_max_size() -> u64 { u64::MAX }
fn default_auto_clean_report_format() -> String { "text".to_string() }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SafeConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub confirm_all: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WhitelistConfig {
    #[serde(default)]
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlacklistConfig {
    #[serde(default)]
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModulesConfig {
    #[serde(default = "default_true")]
    pub packages: bool,
    #[serde(default = "default_true")]
    pub system: bool,
    #[serde(default = "default_true")]
    pub apps: bool,
    #[serde(default)]
    pub privacy: bool,
    #[serde(default)]
    pub services: bool,
    #[serde(default)]
    pub duplicates: bool,
    #[serde(default)]
    pub disk: bool,
    #[serde(default = "default_disk_min_size")]
    pub disk_min_size: u64,
    #[serde(default)]
    pub containers: bool,
    #[serde(default)]
    pub broken_links: bool,
    #[serde(default)]
    pub empty_files: bool,
    #[serde(default)]
    pub old_downloads: bool,
    #[serde(default)]
    pub old_logs: bool,
    #[serde(default)]
    pub fonts: bool,
    #[serde(default)]
    pub scan_paths: Vec<String>,
    #[serde(default)]
    pub exclude_paths: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackupConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_backup_location")]
    pub location: PathBuf,
    #[serde(default = "default_retention")]
    pub retention_days: u64,
    #[serde(default = "default_max_backups")]
    pub max_backups: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: u16,
    #[serde(default = "default_true")]
    pub show_icons: bool,
    #[serde(default = "default_true")]
    pub show_details: bool,
    #[serde(default = "default_true")]
    pub enable_filter: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
    Dracula,
    Gruvbox,
    Nord,
}

impl Theme {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "light" => Theme::Light,
            "dracula" => Theme::Dracula,
            "gruvbox" => Theme::Gruvbox,
            "nord" => Theme::Nord,
            _ => Theme::Dark,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_backup_location() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("~/.local/share"));
    base.join("windebloat").join("backups")
}

fn default_retention() -> u64 {
    30
}

fn default_max_backups() -> usize {
    10
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_sidebar_width() -> u16 {
    22
}

fn default_disk_min_size() -> u64 {
    100 * 1024 * 1024
}

impl Default for Config {
    fn default() -> Self {
        Config {
            safe: SafeConfig {
                enabled: true,
                confirm_all: true,
            },
            whitelist: WhitelistConfig { paths: Vec::new() },
            blacklist: BlacklistConfig { paths: Vec::new() },
            modules: ModulesConfig {
                packages: true,
                system: true,
                apps: true,
                privacy: false,
                services: false,
                duplicates: false,
                disk: false,
                disk_min_size: 100 * 1024 * 1024,
                containers: false,
                broken_links: true,
                empty_files: true,
                old_downloads: true,
                old_logs: true,
                fonts: false,
                scan_paths: vec![],
                exclude_paths: vec![],
            },
            backup: BackupConfig {
                enabled: true,
                location: default_backup_location(),
                retention_days: 30,
                max_backups: 10,
            },
            ui: UiConfig {
                theme: "dark".to_string(),
                sidebar_width: 22,
                show_icons: true,
                show_details: true,
                enable_filter: true,
            },
            auto_clean: AutoCleanConfig {
                enabled: true,
                profile: "balanced".to_string(),
                safe_only: true,
                benchmark: false,
                predict: false,
                scheduler: false,
                categories: vec!["all".to_string()],
                exclude_categories: vec![],
                min_size: 1024,
                max_size: u64::MAX,
                report_format: "text".to_string(),
                report_output: None,
            },
        }
    }
}
