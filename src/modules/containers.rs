use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::utils::error::Result;
use crate::utils::disk::dir_size;
use std::path::Path;
use std::fs;
use std::process::Command;

pub struct ContainerModule;

impl CleanModule for ContainerModule {
    fn id(&self) -> &'static str {
        "containers"
    }
    fn name(&self) -> &'static str {
        "Containers/VMs"
    }
    fn category(&self) -> Category {
        Category::Containers
    }
    fn description(&self) -> &'static str {
        "Clean Docker, Podman, Flatpak, Snap containers and virtual machines"
    }

    fn scan(&self, _os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();

        items.extend(docker_cleanup());
        items.extend(podman_cleanup());
        items.extend(flatpak_cleanup());
        items.extend(snap_cleanup());
        items.extend(virtualbox_cleanup());
        items.extend(libvirt_cleanup());

        Ok(items)
    }
}

fn docker_cleanup() -> Vec<CleanItem> {
    let mut items = Vec::new();

    if Command::new("docker").arg("--version").output().is_err() {
        return items;
    }

    if let Ok(output) = Command::new("docker").args(["images", "-q", "--filter", "dangling=true"]).output() {
        let images: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(|s| s.to_string())
            .collect();
        if !images.is_empty() {
            items.push(CleanItem::new(
                "docker-dangling-images",
                "docker dangling images",
                images.len() as u64 * 100 * 1024 * 1024,
                true,
                &format!("Dangling Docker images ({} images)", images.len()),
                "docker image prune -f",
            ).with_category("docker"));
        }
    }

    if let Ok(output) = Command::new("docker").args(["ps", "-aq", "--filter", "status=exited"]).output() {
        let containers: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(|s| s.to_string())
            .collect();
        if !containers.is_empty() {
            items.push(CleanItem::new(
                "docker-stopped-containers",
                "docker stopped containers",
                containers.len() as u64 * 50 * 1024 * 1024,
                true,
                &format!("Stopped Docker containers ({} containers)", containers.len()),
                "docker container prune -f",
            ).with_category("docker"));
        }
    }

    if let Ok(output) = Command::new("docker").args(["system", "df", "--format", "{{.Size}}"]).output() {
        let sizes: Vec<u64> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|l| {
                let size_str = l.trim();
                if size_str.is_empty() || size_str == "0B" {
                    return None;
                }
                parse_docker_size(size_str)
            })
            .collect();
        let total_size: u64 = sizes.iter().sum();
        if total_size > 100 * 1024 * 1024 {
            items.push(CleanItem::new(
                "docker-build-cache",
                "docker build cache",
                total_size,
                true,
                "Docker build cache and volumes",
                "docker builder prune -af && docker volume prune -f",
            ).with_category("docker"));
        }
    }

    if let Ok(output) = Command::new("docker").args(["system", "df", "-v"]).output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        if output_str.contains("Volumes") {
            items.push(CleanItem::new(
                "docker-volumes",
                "docker volumes",
                100 * 1024 * 1024,
                false,
                "Docker unused volumes",
                "docker volume prune -f",
            ).with_category("docker"));
        }
    }

    items
}

fn podman_cleanup() -> Vec<CleanItem> {
    let mut items = Vec::new();

    if Command::new("podman").arg("--version").output().is_err() {
        return items;
    }

    if let Ok(output) = Command::new("podman").args(["images", "-q", "--filter", "dangling=true"]).output() {
        let images: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(|s| s.to_string())
            .collect();
        if !images.is_empty() {
            items.push(CleanItem::new(
                "podman-dangling-images",
                "podman dangling images",
                images.len() as u64 * 100 * 1024 * 1024,
                true,
                &format!("Dangling Podman images ({} images)", images.len()),
                "podman image prune -f",
            ).with_category("podman"));
        }
    }

    if let Ok(output) = Command::new("podman").args(["ps", "-aq", "--filter", "status=exited"]).output() {
        let containers: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(|s| s.to_string())
            .collect();
        if !containers.is_empty() {
            items.push(CleanItem::new(
                "podman-stopped-containers",
                "podman stopped containers",
                containers.len() as u64 * 50 * 1024 * 1024,
                true,
                &format!("Stopped Podman containers ({} containers)", containers.len()),
                "podman container prune -f",
            ).with_category("podman"));
        }
    }

    items
}

fn flatpak_cleanup() -> Vec<CleanItem> {
    let mut items = Vec::new();

    if Command::new("flatpak").arg("--version").output().is_err() {
        return items;
    }

    if let Ok(output) = Command::new("flatpak").args(["list", "--app", "-d"]).output() {
        let lines: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect();
        if !lines.is_empty() {
            let flatpak_dir = Path::new("/var/lib/flatpak/app");
            if flatpak_dir.exists() {
                let size = dir_size(flatpak_dir);
                if size > 100 * 1024 * 1024 {
                    items.push(CleanItem::new(
                        "flatpak-apps",
                        "/var/lib/flatpak/app",
                        size,
                        false,
                        &format!("Flatpak apps ({} apps)", lines.len()),
                        "flatpak uninstall --unused -y",
                    ).with_category("flatpak"));
                }
            }
        }
    }

    if let Ok(output) = Command::new("flatpak").args(["list", "--runtime", "-d"]).output() {
        let lines: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect();
        if !lines.is_empty() {
            let runtime_dir = Path::new("/var/lib/flatpak/runtime");
            if runtime_dir.exists() {
                let size = dir_size(runtime_dir);
                if size > 100 * 1024 * 1024 {
                    items.push(CleanItem::new(
                        "flatpak-runtimes",
                        "/var/lib/flatpak/runtime",
                        size,
                        false,
                        &format!("Flatpak runtimes ({} runtimes)", lines.len()),
                        "flatpak uninstall --unused -y",
                    ).with_category("flatpak"));
                }
            }
        }
    }

    let cache_dir = Path::new("/var/tmp/flatpak-cache");
    if cache_dir.exists() {
        let size = dir_size(cache_dir);
        if size > 10 * 1024 * 1024 {
            items.push(CleanItem::new(
                "flatpak-cache",
                "/var/tmp/flatpak-cache",
                size,
                true,
                "Flatpak cache",
                "rm -rf /var/tmp/flatpak-cache/*",
            ).with_category("flatpak"));
        }
    }

    items
}

fn snap_cleanup() -> Vec<CleanItem> {
    let mut items = Vec::new();

    if Command::new("snap").arg("version").output().is_err() {
        return items;
    }

    if let Ok(output) = Command::new("bash").args(["-c", "snap list --all 2>/dev/null | tail -n +2 | wc -l"]).output() {
        let count_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if let Ok(count) = count_str.parse::<u64>() {
            if count > 0 {
                let snap_dir = Path::new("/snap");
                if snap_dir.exists() {
                    let size = dir_size(snap_dir);
                    if size > 100 * 1024 * 1024 {
                        items.push(CleanItem::new(
                            "snap-packages",
                            "/snap",
                            size,
                            false,
                            &format!("Snap packages ({} installed, multiple versions)", count),
                            "sudo snap remove --purge $(snap list --all | tail -n +2 | awk '{print $1}' | sort -u)",
                        ).with_category("snap"));
                    }
                }
            }
        }
    }

    items
}

fn virtualbox_cleanup() -> Vec<CleanItem> {
    let mut items = Vec::new();

    let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
    let vbox_dir = Path::new("/home").join(&username).join("VirtualBox VMs");
    let alt_vbox_dir = Path::new("/root/VirtualBox VMs");

    for dir in [vbox_dir.clone(), alt_vbox_dir.to_path_buf()].iter() {
        if dir.exists() {
            let mut total_size: u64 = 0;
            let mut disk_count = 0;

            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.extension().map(|e| e == "vdi" || e == "vmdk" || e == "vhd").unwrap_or(false) {
                        if let Ok(meta) = path.metadata() {
                            total_size += meta.len();
                            disk_count += 1;
                        }
                    }
                }
            }

            if total_size > 100 * 1024 * 1024 && disk_count > 0 {
                items.push(CleanItem::new(
                    "virtualbox-disks",
                    &dir.to_string_lossy(),
                    total_size,
                    false,
                    &format!("VirtualBox disk images ({} disks)", disk_count),
                    &format!("# Review VMs before deletion in {}", dir.display()),
                ).with_category("virtualbox"));
            }
        }
    }

    items
}

fn libvirt_cleanup() -> Vec<CleanItem> {
    let mut items = Vec::new();

    let libvirt_images = Path::new("/var/lib/libvirt/images");
    if libvirt_images.exists() {
        let mut total_size: u64 = 0;
        let mut disk_count = 0;

        if let Ok(entries) = fs::read_dir(libvirt_images) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().map(|e| e == "qcow2" || e == "qcow" || e == "img").unwrap_or(false) {
                    if let Ok(meta) = path.metadata() {
                        total_size += meta.len();
                        disk_count += 1;
                    }
                }
            }
        }

        if total_size > 100 * 1024 * 1024 && disk_count > 0 {
            items.push(CleanItem::new(
                "libvirt-images",
                "/var/lib/libvirt/images",
                total_size,
                false,
                &format!("Libvirt/KVM disk images ({} images)", disk_count),
                "# Review VMs before deletion",
            ).with_category("libvirt"));
        }
    }

    let libvirt_dump = Path::new("/var/lib/libvirt/qemu/dump");
    if libvirt_dump.exists() {
        let size = dir_size(libvirt_dump);
        if size > 100 * 1024 * 1024 {
            items.push(CleanItem::new(
                "libvirt-coredumps",
                "/var/lib/libvirt/qemu/dump",
                size,
                true,
                "Libvirt VM crash dumps",
                "rm -rf /var/lib/libvirt/qemu/dump/*",
            ).with_category("libvirt"));
        }
    }

    items
}

fn parse_docker_size(size_str: &str) -> Option<u64> {
    let size_str = size_str.trim();
    let mut multiplier: u64 = 1;

    let size_str = if size_str.ends_with("GB") {
        multiplier = 1024 * 1024 * 1024;
        &size_str[..size_str.len() - 2]
    } else if size_str.ends_with("MB") {
        multiplier = 1024 * 1024;
        &size_str[..size_str.len() - 2]
    } else if size_str.ends_with("KB") || size_str.ends_with("kB") {
        multiplier = 1024;
        &size_str[..size_str.len() - 2]
    } else if size_str.ends_with("B") {
        &size_str[..size_str.len() - 1]
    } else {
        size_str
    };

    size_str.trim().parse::<f64>().ok().map(|v| (v * multiplier as f64) as u64)
}