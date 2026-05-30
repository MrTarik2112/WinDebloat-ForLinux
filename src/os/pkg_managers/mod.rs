pub mod apk;
pub mod apt;
pub mod dnf;
pub mod pacman;
pub mod zypper;

use super::distro::{DistroFamily, OSInfo};
use crate::core::scanner::CleanItem;
use crate::utils::error::Result;

pub struct PkgManager;

impl PkgManager {
    pub fn clean_cache(os: &OSInfo) -> Result<Vec<CleanItem>> {
        match os.family {
            DistroFamily::Debian => apt::clean_cache(),
            DistroFamily::Arch => pacman::clean_cache(),
            DistroFamily::Fedora => dnf::clean_cache(),
            DistroFamily::OpenSuse => zypper::clean_cache(),
            DistroFamily::Alpine => apk::clean_cache(),
            _ => Ok(vec![]),
        }
    }

    pub fn clean_orphaned(os: &OSInfo) -> Result<Vec<CleanItem>> {
        match os.family {
            DistroFamily::Debian => apt::clean_orphaned(),
            DistroFamily::Arch => pacman::clean_orphaned(),
            DistroFamily::Fedora => dnf::clean_orphaned(),
            DistroFamily::OpenSuse => zypper::clean_orphaned(),
            DistroFamily::Alpine => apk::clean_orphaned(),
            _ => Ok(vec![]),
        }
    }

    pub fn clean_old_kernels(os: &OSInfo) -> Result<Vec<CleanItem>> {
        match os.family {
            DistroFamily::Debian => apt::clean_old_kernels(),
            DistroFamily::Arch => pacman::clean_old_kernels(),
            DistroFamily::Fedora => dnf::clean_old_kernels(),
            DistroFamily::OpenSuse => zypper::clean_old_kernels(),
            DistroFamily::Alpine => apk::clean_old_kernels(),
            _ => Ok(vec![]),
        }
    }
}
