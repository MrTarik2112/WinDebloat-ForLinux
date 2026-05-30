use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::utils::error::Result;
use crate::utils::disk::dir_size;
use std::path::Path;

pub struct MailClientsModule;

impl CleanModule for MailClientsModule {
    fn id(&self) -> &'static str { "mail_clients" }
    fn name(&self) -> &'static str { "Mail Clients" }
    fn category(&self) -> Category { Category::Apps }
    fn description(&self) -> &'static str {
        "Clean mail client caches and attachments"
    }

    fn scan(&self, _os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();
        if let Some(home) = dirs::home_dir() {
            let mail_caches = vec![
                (home.join(".cache/thunderbird"), "Thunderbird cache"),
                (home.join(".thunderbird/Cache"), "Thunderbird profile cache"),
                (home.join(".config/evolution/cache"), "Evolution cache"),
                (home.join(".local/share/evolution/mail"), "Evolution mail cache"),
                (home.join(".config/kmail2"), "KMail cache"),
                (home.join(".local/share/kmail2"), "KMail data cache"),
                (home.join(".config/geary"), "Geary cache"),
                (home.join(".local/share/geary"), "Geary data cache"),
                (home.join(".cache/mailspring"), "Mailspring cache"),
            ];

            for (path, desc) in &mail_caches {
                if path.exists() {
                    let size = dir_size(path);
                    if size > 10 * 1024 * 1024 {
                        items.push(CleanItem::new(
                            &format!("mail-{}", path.file_name().unwrap_or_default().to_string_lossy()),
                            &path.to_string_lossy(),
                            size,
                            false,
                            desc,
                            &format!("rm -rf '{}'/*", path.display()),
                        ).with_category("mail"));
                    }
                }
            }
        }
        Ok(items)
    }
}
