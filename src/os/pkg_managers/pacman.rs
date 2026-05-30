use crate::core::scanner::CleanItem;
use crate::utils::error::Result;
use std::process::Command;

pub fn clean_cache() -> Result<Vec<CleanItem>> {
    let size = get_cache_size()?;
    if size > 0 {
        Ok(vec![CleanItem::new(
            "pacman-cache",
            "/var/cache/pacman/pkg",
            size,
            true,
            "Pacman package cache (.pkg.tar files)",
            "pacman -Sc",
        )])
    } else {
        Ok(vec![])
    }
}

pub fn clean_orphaned() -> Result<Vec<CleanItem>> {
    let mut items = Vec::new();
    if let Ok(output) = Command::new("pacman").args(["-Qtdq"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let count = stdout.lines().filter(|l| !l.is_empty()).count();
        if count > 0 {
            items.push(CleanItem::new(
                "pacman-orphaned",
                "pacman -Qtdq",
                count as u64 * 5 * 1024 * 1024,
                true,
                &format!("Orphaned packages ({})", count),
                "pacman -Rns $(pacman -Qtdq)",
            ));
        }
    }
    Ok(items)
}

pub fn clean_old_kernels() -> Result<Vec<CleanItem>> {
    let mut items = Vec::new();
    if let Ok(output) = Command::new("pacman").args(["-Q", "linux"]).output() {
        let _stdout = String::from_utf8_lossy(&output.stdout);
        // On arch only keep current kernel
        if let Ok(ls) = Command::new("ls").args(["/boot/vmlinuz-*"]).output() {
            let boot = String::from_utf8_lossy(&ls.stdout);
            let kernel_files: Vec<&str> = boot.lines().collect();
            if kernel_files.len() > 1 {
                items.push(CleanItem::new(
                    "pacman-old-kernels",
                    "/boot/vmlinuz-* (old)",
                    (kernel_files.len() as u64 - 1) * 50 * 1024 * 1024,
                    false,
                    &format!(
                        "Old kernels in /boot ({} extra)",
                        kernel_files.len() - 1
                    ),
                    "pacman -R linux-lts (manual)",
                ));
            }
        }
    }
    Ok(items)
}

fn get_cache_size() -> Result<u64> {
    let path = std::path::Path::new("/var/cache/pacman/pkg");
    if path.exists() {
        Ok(crate::utils::disk::dir_size(path))
    } else {
        Ok(0)
    }
}

pub fn clean_unused_snapshots() -> Result<Vec<CleanItem>> {
    Ok(Vec::new())
}
