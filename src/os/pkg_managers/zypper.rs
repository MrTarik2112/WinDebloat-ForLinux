use crate::core::scanner::CleanItem;
use crate::utils::error::Result;
use std::process::Command;

pub fn clean_cache() -> Result<Vec<CleanItem>> {
    let size = get_cache_size()?;
    if size > 0 {
        Ok(vec![CleanItem::new(
            "zypper-cache",
            "/var/cache/zypp",
            size,
            true,
            "Zypper package cache",
            "zypper clean",
        )])
    } else {
        Ok(vec![])
    }
}

pub fn clean_orphaned() -> Result<Vec<CleanItem>> {
    let mut items = Vec::new();
    if let Ok(output) = Command::new("zypper")
        .args(["packages", "--unneeded"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let count = stdout.lines().filter(|l| l.contains("|").into()).count();
        if count > 0 {
            items.push(CleanItem::new(
                "zypper-orphaned",
                "zypper packages --unneeded",
                count as u64 * 5 * 1024 * 1024,
                true,
                &format!("Unneeded packages ({})", count),
                "zypper rm $(zypper packages --unneeded)",
            ));
        }
    }
    Ok(items)
}

pub fn clean_old_kernels() -> Result<Vec<CleanItem>> {
    let mut items = Vec::new();
    if let Ok(output) = Command::new("rpm").args(["-q", "kernel"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let count = stdout.lines().count();
        if count > 1 {
            items.push(CleanItem::new(
                "zypper-old-kernels",
                "kernel (old)",
                (count as u64 - 1) * 200 * 1024 * 1024,
                false,
                &format!("Old kernels ({} installed)", count),
                "zypper rm kernel-old",
            ));
        }
    }
    Ok(items)
}

fn get_cache_size() -> Result<u64> {
    let path = std::path::Path::new("/var/cache/zypp");
    if path.exists() {
        Ok(crate::utils::disk::dir_size(path))
    } else {
        Ok(0)
    }
}
