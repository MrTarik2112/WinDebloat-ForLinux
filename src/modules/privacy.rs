use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::utils::error::Result;
use std::path::Path;

pub struct PrivacyModule;

fn shell_history(home: &Path) -> Vec<CleanItem> {
    let mut items = Vec::new();
    let histories = vec![
        (".bash_history", "Bash shell history"),
        (".zsh_history", "Zsh shell history"),
        (".zhistory", "Zsh history"),
        (".fish_history", "Fish shell history"),
        (".python_history", "Python REPL history"),
        (".psql_history", "PostgreSQL history"),
        (".mysql_history", "MySQL history"),
        (".sqlite_history", "SQLite history"),
        (".node_repl_history", "Node.js REPL history"),
    ];

    for (file, desc) in &histories {
        let full_path = home.join(file);
        if full_path.exists() {
            if let Ok(meta) = full_path.metadata() {
                if meta.len() > 1024 {
                    items.push(CleanItem::new(
                        &format!("history-{}", file.trim_start_matches('.')),
                        &full_path.to_string_lossy(),
                        meta.len(),
                        true,
                        desc,
                        &format!("> '{}'", full_path.display()),
                    ));
                }
            }
        }
    }

    items
}

fn recent_files(home: &Path) -> Vec<CleanItem> {
    let mut items = Vec::new();
    let recent_path = home.join(".local/share/recently-used.xbel");
    if recent_path.exists() {
        if let Ok(meta) = recent_path.metadata() {
            if meta.len() > 1024 {
                items.push(CleanItem::new(
                    "recent-files",
                    &recent_path.to_string_lossy(),
                    meta.len(),
                    true,
                    "Recently used files list",
                    &format!("> '{}'", recent_path.display()),
                ));
            }
        }
    }
    items
}

fn clipboard_history(home: &Path) -> Vec<CleanItem> {
    let mut items = Vec::new();
    let cliphist = home.join(".local/share/cliphist");
    if cliphist.exists() {
        let size = crate::utils::disk::dir_size(&cliphist);
        if size > 1024 {
            items.push(CleanItem::new(
                "clipboard-history",
                &cliphist.to_string_lossy(),
                size,
                true,
                "Clipboard history (cliphist)",
                &format!("rm -rf '{}'/*", cliphist.display()),
            ));
        }
    }

    // CopyQ
    let copyq = home.join(".config/copyq");
    if copyq.exists() {
        let size = crate::utils::disk::dir_size(&copyq);
        if size > 1024 * 1024 {
            items.push(CleanItem::new(
                "copyq-data",
                &copyq.to_string_lossy(),
                size,
                true,
                "CopyQ clipboard manager data",
                &format!("rm -rf '{}'/*.dat", copyq.display()),
            ));
        }
    }

    items
}

impl CleanModule for PrivacyModule {
    fn id(&self) -> &'static str {
        "privacy"
    }
    fn name(&self) -> &'static str {
        "Privacy"
    }
    fn category(&self) -> Category {
        Category::Privacy
    }
    fn description(&self) -> &'static str {
        "Clear shell history, recent files, clipboard data, and activity logs"
    }

    fn scan(&self, _os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();

        if let Some(home) = dirs::home_dir() {
            items.extend(shell_history(&home));
            items.extend(recent_files(&home));
            items.extend(clipboard_history(&home));
        }

        Ok(items)
    }
}
