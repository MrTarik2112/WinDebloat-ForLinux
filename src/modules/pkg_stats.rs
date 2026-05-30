use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::os::distro::DistroFamily;
use crate::utils::error::Result;
use std::process::Command;

pub struct PkgStatsModule;

impl CleanModule for PkgStatsModule {
    fn id(&self) -> &'static str {
        "pkg_stats"
    }
    fn name(&self) -> &'static str {
        "Package Stats"
    }
    fn category(&self) -> Category {
        Category::Packages
    }
    fn description(&self) -> &'static str {
        "Show package manager statistics and outdated packages"
    }

    fn scan(&self, os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();

        match os.family {
            DistroFamily::Debian => {
                items.extend(debian_stats());
            }
            DistroFamily::Arch => {
                items.extend(arch_stats());
            }
            DistroFamily::Fedora => {
                items.extend(fedora_stats());
            }
            DistroFamily::OpenSuse => {
                items.extend(suse_stats());
            }
            DistroFamily::Alpine => {
                items.extend(alpine_stats());
            }
            _ => {}
        }

        Ok(items)
    }
}

fn debian_stats() -> Vec<CleanItem> {
    let mut items = Vec::new();

    if let Ok(output) = Command::new("dpkg").arg("-l").output() {
        let count = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|l| l.starts_with("ii"))
            .count();
        items.push(CleanItem::new(
            "debian-installed",
            "dpkg installed packages",
            0,
            false,
            &format!("{} installed packages", count),
            "",
        ));
    }

    if let Ok(output) = Command::new("apt").args(["list", "--upgradable"]).output() {
        let updatable = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|l| !l.starts_with("Listing"))
            .count();
        if updatable > 0 {
            items.push(CleanItem::new(
                "debian-upgradable",
                "apt upgradable packages",
                updatable as u64 * 50 * 1024 * 1024,
                false,
                &format!("{} upgradable packages", updatable),
                "sudo apt upgrade -y",
            ));
        }
    }

    items
}

fn arch_stats() -> Vec<CleanItem> {
    let mut items = Vec::new();

    if let Ok(output) = Command::new("pacman").args(["-Qq"]).output() {
        let count = String::from_utf8_lossy(&output.stdout)
            .lines()
            .count();
        items.push(CleanItem::new(
            "arch-installed",
            "pacman installed packages",
            0,
            false,
            &format!("{} installed packages", count),
            "",
        ));
    }

    if let Ok(output) = Command::new("checkupdates").output() {
        let count = String::from_utf8_lossy(&output.stdout)
            .lines()
            .count();
        if count > 0 {
            items.push(CleanItem::new(
                "arch-upgradable",
                "arch upgradable packages",
                count as u64 * 100 * 1024 * 1024,
                false,
                &format!("{} packages can be updated", count),
                "sudo pacman -Syu",
            ));
        }
    }

    items
}

fn fedora_stats() -> Vec<CleanItem> {
    let mut items = Vec::new();

    if let Ok(output) = Command::new("rpm").args(["-qa"]).output() {
        let count = String::from_utf8_lossy(&output.stdout)
            .lines()
            .count();
        items.push(CleanItem::new(
            "fedora-installed",
            "rpm installed packages",
            0,
            false,
            &format!("{} installed packages", count),
            "",
        ));
    }

    if let Ok(output) = Command::new("dnf").args(["check-update", "--quiet"]).output() {
        let code = output.status.code();
        if code == Some(100) {
            let update_count = String::from_utf8_lossy(&output.stdout)
                .lines()
                .count()
                .max(1);
            items.push(CleanItem::new(
                "fedora-updates",
                "dnf available updates",
                update_count as u64 * 200 * 1024 * 1024,
                false,
                &format!("{} packages have updates available", update_count),
                "sudo dnf upgrade -y",
            ));
        }
    }

    items
}

fn suse_stats() -> Vec<CleanItem> {
    let mut items = Vec::new();

    if let Ok(output) = Command::new("zypper").args(["pa", "-i"]).output() {
        let count = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|l| !l.starts_with("S |"))
            .count();
        items.push(CleanItem::new(
            "suse-installed",
            "zypper installed packages",
            0,
            false,
            &format!("{} installed packages", count),
            "",
        ));
    }

    items
}

fn alpine_stats() -> Vec<CleanItem> {
    let mut items = Vec::new();

    if let Ok(output) = Command::new("apk").args(["info"]).output() {
        let count = String::from_utf8_lossy(&output.stdout)
            .lines()
            .count();
        items.push(CleanItem::new(
            "alpine-installed",
            "apk installed packages",
            0,
            false,
            &format!("{} installed packages", count),
            "",
        ));
    }

    items
}