use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::utils::error::Result;
use std::path::Path;

pub struct KernelModule;

impl CleanModule for KernelModule {
    fn id(&self) -> &'static str { "kernel" }
    fn name(&self) -> &'static str { "Kernel" }
    fn category(&self) -> Category { Category::Kernel }
    fn description(&self) -> &'static str {
        "Clean old kernel versions and modules"
    }

    fn is_available(&self, os: &OSInfo) -> bool {
        matches!(os.family, 
            crate::os::distro::DistroFamily::Debian |
            crate::os::distro::DistroFamily::Arch |
            crate::os::distro::DistroFamily::Fedora
        )
    }

    fn scan(&self, _os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();

        // Old kernels (Debian/Ubuntu)
        if let Ok(output) = std::process::Command::new("dpkg")
            .args(["-l", "linux-image-*-generic"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let pkgs: Vec<&str> = output_str
                .lines()
                .filter(|l| l.starts_with("ii"))
                .collect();
            
            if pkgs.len() > 1 {
                items.push(CleanItem::new(
                    "old-kernels-debian",
                    "/boot",
                    (pkgs.len() - 1) as u64 * 80 * 1024 * 1024,
                    true,
                    &format!("{} old kernels (keep current)", pkgs.len() - 1),
                    "sudo apt-get autoremove -y",
                ).with_file_count((pkgs.len() - 1) as u64));
            }
        }

        // Old kernels (Arch)
        if let Ok(output) = std::process::Command::new("pacman")
            .args(["-q", "-i", "^linux$"])
            .output()
        {
            let current = String::from_utf8_lossy(&output.stdout);
            if !current.is_empty() {
                let current_ver = current.lines().next().unwrap_or("");
                
                if let Ok(pkgs_output) = std::process::Command::new("pacman")
                    .args(["-q", "-l", "linux"])
                    .output()
                {
                    let pkgs_str = String::from_utf8_lossy(&pkgs_output.stdout);
                    let installed: Vec<&str> = pkgs_str
                        .lines()
                        .filter(|l| !l.contains(current_ver))
                        .collect();
                    
                    if !installed.is_empty() {
                        items.push(CleanItem::new(
                            "old-kernels-arch",
                            "/boot",
                            installed.len() as u64 * 100 * 1024 * 1024,
                            true,
                            &format!("{} old Arch kernels", installed.len()),
                            "sudo pacman -Rcs $(pacman -Qq linux | grep -v $(uname -r))",
                        ).with_file_count(installed.len() as u64));
                    }
                }
            }
        }

        // Old kernels (Fedora)
        if let Ok(output) = std::process::Command::new("rpm")
            .args(["-qa", "kernel"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let kernels: Vec<&str> = output_str
                .lines()
                .collect();
            
            if kernels.len() > 1 {
                items.push(CleanItem::new(
                    "old-kernels-fedora",
                    "/boot",
                    (kernels.len() - 1) as u64 * 120 * 1024 * 1024,
                    true,
                    &format!("{} old Fedora kernels", kernels.len() - 1),
                    "sudo dnf remove --oldest-kernels",
                ).with_file_count((kernels.len() - 1) as u64));
            }
        }

        // Old initramfs
        let boot_dir = Path::new("/boot");
        if boot_dir.exists() {
            let mut initramfs_count = 0u64;
            let mut initramfs_size = 0u64;
            
            if let Ok(entries) = std::fs::read_dir(boot_dir) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    let name_str = file_name.to_string_lossy();
                    if name_str.contains("initramfs") && !name_str.contains("$(uname)") {
                        if let Ok(meta) = entry.metadata() {
                            initramfs_count += 1;
                            initramfs_size += meta.len();
                        }
                    }
                }
            }

            if initramfs_count > 1 {
                items.push(CleanItem::new(
                    "old-initramfs",
                    "/boot",
                    initramfs_size,
                    true,
                    &format!("{} old initramfs files", initramfs_count),
                    "sudo update-initramfs -d",
                ).with_file_count(initramfs_count));
            }
        }

        Ok(items)
    }
}