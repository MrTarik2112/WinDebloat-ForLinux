use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::utils::error::Result;

pub struct DotFilesModule;

impl CleanModule for DotFilesModule {
    fn id(&self) -> &'static str { "dotfiles" }
    fn name(&self) -> &'static str { "Dotfiles" }
    fn category(&self) -> Category { Category::Privacy }
    fn description(&self) -> &'static str {
        "Clean old dotfile backups and temporary config files"
    }

    fn scan(&self, _os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();
        if let Some(home) = dirs::home_dir() {
            let patterns = vec![
                (home.join(".bashrc.old"), "Old bashrc backup"),
                (home.join(".zshrc.old"), "Old zshrc backup"),
                (home.join(".profile.old"), "Old profile backup"),
                (home.join(".vimrc.old"), "Old vimrc backup"),
                (home.join(".gitconfig.old"), "Old gitconfig backup"),
                (home.join(".tmux.conf.old"), "Old tmux backup"),
            ];

            for (path, desc) in &patterns {
                if path.exists() {
                    if let Ok(meta) = path.metadata() {
                        items.push(CleanItem::new(
                            &format!("dot-{}", path.file_name().unwrap_or_default().to_string_lossy()),
                            &path.to_string_lossy(),
                            meta.len(),
                            true,
                            desc,
                            &format!("rm -f '{}'", path.display()),
                        ).with_category("dotfiles"));
                    }
                }
            }

            let swap_patterns = vec![
                home.join(".config"),
                home,
            ];
            for dir in &swap_patterns {
                if dir.exists() {
                    if let Ok(entries) = std::fs::read_dir(dir) {
                        for entry in entries.filter_map(|e| e.ok()) {
                            let name = entry.file_name().to_string_lossy().to_string();
                            if name.ends_with(".swp") || name.ends_with(".swo") || name.ends_with(".swn") {
                                if let Ok(meta) = entry.metadata() {
                                    items.push(CleanItem::new(
                                        &format!("swap-{}", name),
                                        &entry.path().to_string_lossy(),
                                        meta.len(),
                                        true,
                                        "Vim/Nano swap file",
                                        &format!("rm -f '{}'", entry.path().display()),
                                    ).with_category("dotfiles"));
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
