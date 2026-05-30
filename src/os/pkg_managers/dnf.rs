use crate::core::scanner::CleanItem;
use crate::utils::error::Result;
use std::process::Command;

pub fn clean_cache() -> Result<Vec<CleanItem>> {
    let size = get_cache_size()?;
    if size > 0 {
        Ok(vec![CleanItem::new(
            "dnf-cache",
            "/var/cache/dnf",
            size,
            true,
            "DNF package cache",
            "dnf clean all",
        )])
    } else {
        Ok(vec![])
    }
}

pub fn clean_orphaned() -> Result<Vec<CleanItem>> {
    let mut items = Vec::new();
    if let Ok(output) = Command::new("dnf")
        .args(["repoquery", "--unneeded"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let count = stdout.lines().filter(|l| !l.is_empty()).count();
        if count > 0 {
            items.push(CleanItem::new(
                "dnf-orphaned",
                "dnf repoquery --unneeded",
                count as u64 * 10 * 1024 * 1024,
                true,
                &format!("Unneeded packages ({})", count),
                "dnf autoremove",
            ));
        }
    }
    Ok(items)
}

pub fn clean_old_kernels() -> Result<Vec<CleanItem>> {
    let mut items = Vec::new();
    if let Ok(output) = Command::new("dnf").args(["list", "installed"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let old_kernels = stdout
            .lines()
            .filter(|l| l.contains("kernel-core") || l.contains("kernel-devel"))
            .count();
        if old_kernels > 2 {
            items.push(CleanItem::new(
                "dnf-old-kernels",
                "kernel-core (old)",
                (old_kernels as u64 - 1) * 200 * 1024 * 1024,
                false,
                &format!("Old kernels ({} installed)", old_kernels),
                "dnf remove --oldinstallonly",
            ));
        }
    }
    Ok(items)
}

fn get_cache_size() -> Result<u64> {
    let path = std::path::Path::new("/var/cache/dnf");
    if path.exists() {
        Ok(crate::utils::disk::dir_size(path))
    } else {
        Ok(0)
    }
}
