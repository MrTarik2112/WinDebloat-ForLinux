use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::utils::error::Result;
use crate::utils::disk::dir_size;

pub struct GamesModule;

impl CleanModule for GamesModule {
    fn id(&self) -> &'static str { "games" }
    fn name(&self) -> &'static str { "Games" }
    fn category(&self) -> Category { Category::Apps }
    fn description(&self) -> &'static str {
        "Clean Steam, Lutris, Wine, and game caches"
    }

    fn scan(&self, _os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();
        if let Some(home) = dirs::home_dir() {
            let steam_dir = home.join(".steam");
            if steam_dir.exists() {
                let compat = home.join(".local/share/Steam/steamapps/compatdata");
                if compat.exists() {
                    let size = dir_size(&compat);
                    if size > 100 * 1024 * 1024 {
                        items.push(CleanItem::new(
                            "steam-compatdata",
                            &compat.to_string_lossy(),
                            size,
                            false,
                            "Steam Proton prefix (compatdata)",
                            &format!("rm -rf '{}'/*", compat.display()),
                        ).with_category("games"));
                    }
                }
                let workshop = home.join(".local/share/Steam/steamapps/workshop");
                if workshop.exists() {
                    let size = dir_size(&workshop);
                    if size > 100 * 1024 * 1024 {
                        items.push(CleanItem::new(
                            "steam-workshop",
                            &workshop.to_string_lossy(),
                            size,
                            true,
                            "Steam Workshop content (re-downloadable)",
                            &format!("rm -rf '{}'/*", workshop.display()),
                        ).with_category("games"));
                    }
                }
            }

            let shader_cache = home.join(".local/share/Steam/steamapps/shadercache");
            if shader_cache.exists() {
                let size = dir_size(&shader_cache);
                if size > 50 * 1024 * 1024 {
                    items.push(CleanItem::new(
                        "steam-shadercache",
                        &shader_cache.to_string_lossy(),
                        size,
                        true,
                        "Steam shader cache",
                        &format!("rm -rf '{}'/*", shader_cache.display()),
                    ).with_category("games"));
                }
            }

            let lutris = home.join(".local/share/lutris");
            if lutris.exists() {
                let runners = lutris.join("runners");
                if runners.exists() {
                    let size = dir_size(&runners);
                    if size > 100 * 1024 * 1024 {
                        items.push(CleanItem::new(
                            "lutris-runners",
                            &runners.to_string_lossy(),
                            size,
                            false,
                            "Lutris wine runners (eski versiyonlar)",
                            &format!("rm -rf '{}'/*", runners.display()),
                        ).with_category("games"));
                    }
                }
            }

            let wine_prefixes = home.join(".wine");
            if wine_prefixes.exists() {
                let size = dir_size(&wine_prefixes);
                if size > 100 * 1024 * 1024 {
                    items.push(CleanItem::new(
                        "wine-prefix",
                        &wine_prefixes.to_string_lossy(),
                        size,
                        false,
                        "Wine prefix (genel)",
                        &format!("rm -rf '{}'", wine_prefixes.display()),
                    ).with_category("games"));
                }
            }
        }
        Ok(items)
    }
}
