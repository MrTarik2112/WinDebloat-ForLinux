use crate::core::scanner::CleanItem;
use crate::utils::error::Result;
use std::process::Command;

pub fn clean_cache() -> Result<Vec<CleanItem>> {
    let size = get_cache_size()?;
    if size > 0 {
        Ok(vec![CleanItem::new(
            "apt-cache",
            "/var/cache/apt",
            size,
            true,
            "APT package cache (downloaded .deb files)",
            "apt clean",
        )])
    } else {
        Ok(vec![])
    }
}

pub fn clean_orphaned() -> Result<Vec<CleanItem>> {
    let mut items = Vec::new();
    if let Ok(output) = Command::new("apt-get").args(["--just-print", "autoremove"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().filter(|l| l.contains("Remv")).collect();
        if !lines.is_empty() {
            items.push(CleanItem::new(
                "apt-orphaned",
                "apt-get autoremove",
                lines.len() as u64 * 1024 * 1024,
                true,
                &format!("Orphaned packages ({} packages)", lines.len()),
                "apt-get autoremove -y",
            ));
        }
    }
    Ok(items)
}

pub fn clean_old_kernels() -> Result<Vec<CleanItem>> {
    let mut items = Vec::new();
    if let Ok(output) = Command::new("dpkg")
        .args(["--list"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let kernel_count = stdout
            .lines()
            .filter(|l| l.contains("linux-image-") && !l.contains("$(uname -r)"))
            .count()
            .saturating_sub(1);
        if kernel_count > 0 {
            items.push(CleanItem::new(
                "apt-old-kernels",
                "linux-image-* (old)",
                kernel_count as u64 * 300 * 1024 * 1024,
                false,
                &format!("Old kernel images (~{} packages)", kernel_count),
                "apt-get autoremove --purge",
            ));
        }
    }
    Ok(items)
}

fn get_cache_size() -> Result<u64> {
    let path = std::path::Path::new("/var/cache/apt");
    if path.exists() {
        Ok(crate::utils::disk::dir_size(path))
    } else {
        Ok(0)
    }
}
