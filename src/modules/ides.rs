use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::utils::error::Result;
use crate::utils::disk::dir_size;

pub struct IDEsModule;

impl CleanModule for IDEsModule {
    fn id(&self) -> &'static str { "ides" }
    fn name(&self) -> &'static str { "IDEs" }
    fn category(&self) -> Category { Category::Apps }
    fn description(&self) -> &'static str {
        "Clean IDE caches, indexes, and workspace data"
    }

    fn scan(&self, _os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();
        if let Some(home) = dirs::home_dir() {
            let ides = vec![
                (home.join(".cache/JetBrains"), "JetBrains IDEs cache"),
                (home.join(".cache/Code"), "VS Code cache"),
                (home.join(".config/Code/Cache"), "VS Code editor cache"),
                (home.join(".config/Code/CachedData"), "VS Code cached data"),
                (home.join(".android/cache"), "Android Studio cache"),
                (home.join(".gradle/caches"), "Gradle cache (IDE)"),
                (home.join(".cache/unity"), "Unity Editor cache"),
                (home.join(".cache/visualstudio"), "Visual Studio cache"),
                (home.join(".cache/eclipse"), "Eclipse cache"),
            ];

            for (path, desc) in &ides {
                if path.exists() {
                    let size = dir_size(path);
                    if size > 10 * 1024 * 1024 {
                        items.push(CleanItem::new(
                            &format!("ide-{}", path.file_name().unwrap_or_default().to_string_lossy()),
                            &path.to_string_lossy(),
                            size,
                            true,
                            desc,
                            &format!("rm -rf '{}'/*", path.display()),
                        ).with_category("ides"));
                    }
                }
            }

            let workspace_storage = home.join(".config/Code/workspaceStorage");
            if workspace_storage.exists() {
                let size = dir_size(&workspace_storage);
                if size > 10 * 1024 * 1024 {
                    items.push(CleanItem::new(
                        "vscode-workspaces",
                        &workspace_storage.to_string_lossy(),
                        size,
                        true,
                        "VS Code workspace storage",
                        &format!("rm -rf '{}'/*", workspace_storage.display()),
                    ).with_category("ides"));
                }
            }
        }
        Ok(items)
    }
}
