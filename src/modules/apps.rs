use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::utils::error::Result;
use std::path::Path;
use std::process::Command;

pub struct AppsModule;

fn browser_cache(home: &Path) -> Vec<CleanItem> {
    let mut items = Vec::new();

    let browsers = vec![
        (".cache/chromium", "Chromium cache"),
        (".cache/google-chrome", "Google Chrome cache"),
        (".cache/mozilla/firefox", "Firefox cache"),
        (".cache/brave-browser", "Brave cache"),
        (".cache/vivaldi", "Vivaldi cache"),
        (".cache/msedge", "Edge cache"),
        (".mozilla/firefox/*.default*/cache2", "Firefox cache2"),
    ];

    for (rel_path, desc) in &browsers {
        // Handle glob patterns
        if rel_path.contains('*') {
            let full_pattern = format!("{}/{}", home.display(), rel_path);
            if let Ok(entries) = glob::glob(&full_pattern) {
                for entry in entries.flatten() {
                    if entry.exists() {
                        let size = crate::utils::disk::dir_size(&entry);
                        if size > 1024 * 1024 {
                            items.push(CleanItem::new(
                                &format!("browser-cache-{}", entry.to_string_lossy().replace('/', "-")),
                                &entry.to_string_lossy(),
                                size,
                                true,
                                desc,
                                &format!("rm -rf '{}'", entry.display()),
                            ));
                        }
                    }
                }
            }
        } else {
            let full_path = home.join(rel_path);
            if full_path.exists() {
                let size = crate::utils::disk::dir_size(&full_path);
                if size > 1024 * 1024 {
                    items.push(CleanItem::new(
                        &format!("browser-cache-{}", rel_path.replace('/', "-")),
                        &full_path.to_string_lossy(),
                        size,
                        true,
                        desc,
                        &format!("rm -rf '{}'", full_path.display()),
                    ));
                }
            }
        }
    }

    items
}

fn flatpak_cleanup() -> Vec<CleanItem> {
    let mut items = Vec::new();
    let path = Path::new("/var/lib/flatpak");
    if path.exists() {
        if let Ok(output) = Command::new("flatpak").args(["list", "--columns=application"]).output() {
            let _stdout = String::from_utf8_lossy(&output.stdout);
            let _size = crate::utils::disk::dir_size(path);
            // Just report unused runtimes
            if let Ok(unused) = Command::new("flatpak").args(["unused", "--columns=size"]).output() {
                let out = String::from_utf8_lossy(&unused.stdout);
                if let Ok(s) = out.trim().parse::<u64>() {
                    if s > 0 {
                        items.push(CleanItem::new(
                            "flatpak-unused",
                            "flatpak unused runtimes",
                            s,
                            true,
                            "Unused Flatpak runtimes",
                            "flatpak uninstall --unused",
                        ));
                    }
                }
            }
        }
    }
    items
}

fn snap_cleanup() -> Vec<CleanItem> {
    let mut items = Vec::new();
    let path = Path::new("/var/lib/snapd");
    if !path.exists() {
        return items;
    }

    // Check for old snap revisions
    if let Ok(output) = Command::new("snap").args(["list", "--all"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let revisions: Vec<&str> = stdout.lines().skip(1).collect();
        if revisions.len() > 1 {
            items.push(CleanItem::new(
                "snap-revisions",
                "snap old revisions",
                revisions.len() as u64 * 50 * 1024 * 1024,
                true,
                "Old Snap package revisions (disabled)",
                "snap list --all | awk '/disabled/{print $1, $3}' | while read snap rev; do sudo snap remove --revision=\"$rev\" \"$snap\"; done",
            ));
        }
    }

    items
}

fn docker_cleanup() -> Vec<CleanItem> {
    let mut items = Vec::new();
    if Command::new("docker").arg("--version").output().is_err() {
        return items;
    }

    // Check for dangling images
    if let Ok(output) = Command::new("docker")
        .args(["images", "--filter", "dangling=true", "-q"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            items.push(CleanItem::new(
                "docker-dangling",
                "docker dangling images",
                500 * 1024 * 1024,
                true,
                "Docker dangling images",
                "docker image prune -f",
            ));
        }
    }

    items
}

fn dev_caches(home: &Path) -> Vec<CleanItem> {
    let mut items = Vec::new();

    let dev_dirs: Vec<(&str, &str, bool)> = vec![
        (".cargo/registry/cache", "Cargo registry cache", true),
        (".npm/_cacache", "npm cache", true),
        (".cache/pip", "pip cache", true),
        (".cache/yarn", "Yarn cache", true),
        (".cache/go-build", "Go build cache", false),
        (".gradle/caches", "Gradle cache", false),
        (".local/share/flatpak/runtime", "Flatpak runtimes cache", false),
    ];

    for (rel_path, desc, is_safe) in &dev_dirs {
        let full_path = home.join(rel_path);
        if full_path.exists() {
            let size = crate::utils::disk::dir_size(&full_path);
            if size > 1024 * 1024 {
                items.push(CleanItem::new(
                    &format!("dev-cache-{}", rel_path.replace('/', "-")),
                    &full_path.to_string_lossy(),
                    size,
                    *is_safe,
                    desc,
                    &format!("rm -rf '{}'/*", full_path.display()),
                ));
            }
        }
    }

    items
}

impl CleanModule for AppsModule {
    fn id(&self) -> &'static str {
        "apps"
    }
    fn name(&self) -> &'static str {
        "Applications"
    }
    fn category(&self) -> Category {
        Category::Apps
    }
    fn description(&self) -> &'static str {
        "Clean browser caches, Flatpak, Snap, Docker, and dev tool caches"
    }

    fn scan(&self, _os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();

        if let Some(home) = dirs::home_dir() {
            items.extend(browser_cache(&home));
            items.extend(dev_caches(&home));
        }

        items.extend(flatpak_cleanup());
        items.extend(snap_cleanup());
        items.extend(docker_cleanup());

        Ok(items)
    }
}
