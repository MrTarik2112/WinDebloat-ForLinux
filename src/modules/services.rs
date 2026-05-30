use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::utils::error::Result;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub struct ServicesModule;

const BLOAT_SERVICES: &[(&str, &str)] = &[
    ("bluetooth", "Bluetooth service - often unused on desktops"),
    ("cups", "CUPS printing service - disable if no printer"),
    ("cups-browsed", "Network printer discovery"),
    ("avahi-daemon", "mDNS/Bonjour - rarely needed on desktop"),
    ("ModemManager", "Modem manager for mobile broadband"),
    ("whoopsie", "Ubuntu crash reporting daemon"),
    ("apport", "Apport crash reporting"),
    ("pcscd", "Smart card daemon"),
    ("abrtd", "ABRT automatic bug reporting tool"),
    ("whoopsie", "Canonical crash reporter"),
    ("packagekit", "Package updates daemon"),
];

impl CleanModule for ServicesModule {
    fn id(&self) -> &'static str { "services" }
    fn name(&self) -> &'static str { "Startup Services" }
    fn category(&self) -> Category { Category::Services }
    fn description(&self) -> &'static str {
        "View/disable unnecessary startup services"
    }

    fn is_available(&self, _os: &OSInfo) -> bool {
        std::path::Path::new("/run/systemd/system").exists()
    }

    fn scan(&self, _os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();

        for (service, desc) in BLOAT_SERVICES {
            if let Ok(output) = Command::new("systemctl")
                .args(["is-enabled", service])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let status = stdout.trim();
                if status == "enabled" || status == "static" {
                    items.push(CleanItem::new(
                        &format!("service-{}", service.trim()),
                        service,
                        0,
                        false,
                        desc,
                        &format!("sudo systemctl disable {} --now", service),
                    ));
                }
            }
        }

        Ok(items)
    }
}

pub struct StartupModule;

impl CleanModule for StartupModule {
    fn id(&self) -> &'static str { "startup" }
    fn name(&self) -> &'static str { "Startup Apps" }
    fn category(&self) -> Category { Category::Services }
    fn description(&self) -> &'static str {
        "View startup applications"
    }

    fn scan(&self, _os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();
        let mut autostart_dirs: Vec<PathBuf> = vec![
            PathBuf::from("/etc/xdg/autostart"),
        ];
        if let Some(home) = dirs::home_dir() {
            autostart_dirs.push(home.join(".config/autostart"));
        }

        for dir in autostart_dirs {
            if dir.exists() {
                if let Ok(entries) = fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map(|e| e == "desktop").unwrap_or(false) {
                            if let Ok(content) = fs::read_to_string(&path) {
                                let name = content.lines()
                                    .find(|l| l.starts_with("Name="))
                                    .map(|l| l.trim_start_matches("Name="))
                                    .unwrap_or("Unknown")
                                    .to_string();

                                let hidden = content.lines()
                                    .any(|l| l.starts_with("Hidden=true"));

                                if !hidden {
                                    items.push(CleanItem::new(
                                        &format!("autostart-{}", path.file_stem().unwrap_or_default().to_string_lossy()),
                                        &path.to_string_lossy(),
                                        0,
                                        false,
                                        &name,
                                        &format!("rm '{}'", path.display()),
                                    ));
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