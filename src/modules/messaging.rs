use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::utils::error::Result;
use crate::utils::disk::dir_size;
use std::path::Path;

pub struct MessagingModule;

impl CleanModule for MessagingModule {
    fn id(&self) -> &'static str { "messaging" }
    fn name(&self) -> &'static str { "Messaging" }
    fn category(&self) -> Category { Category::Apps }
    fn description(&self) -> &'static str {
        "Clean messaging app caches (Discord, Slack, Telegram, etc.)"
    }

    fn scan(&self, _os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();
        if let Some(home) = dirs::home_dir() {
            let cache_dirs = vec![
                (home.join(".cache/discord"), "Discord cache"),
                (home.join(".config/discord/Cache"), "Discord app cache"),
                (home.join(".config/discord/Code Cache"), "Discord code cache"),
                (home.join(".config/slack/Cache"), "Slack cache"),
                (home.join(".config/slack/Service Worker/CacheStorage"), "Slack SW cache"),
                (home.join(".config/slack/CachedData"), "Slack cached data"),
                (home.join(".cache/telegram"), "Telegram cache"),
                (home.join(".local/share/TelegramDesktop/tdata"), "Telegram data cache"),
                (home.join(".config/signal"), "Signal cache"),
                (home.join(".config/element"), "Element cache"),
                (home.join(".config/Rambox"), "Rambox cache"),
                (home.join(".config/Ferdium"), "Ferdium cache"),
            ];

            for (path, desc) in &cache_dirs {
                if path.exists() {
                    let size = dir_size(path);
                    if size > 5 * 1024 * 1024 {
                        items.push(CleanItem::new(
                            &format!("msg-{}", path.file_name().unwrap_or_default().to_string_lossy()),
                            &path.to_string_lossy(),
                            size,
                            true,
                            desc,
                            &format!("rm -rf '{}'/*", path.display()),
                        ).with_category("messaging"));
                    }
                }
            }
        }
        Ok(items)
    }
}
