use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::os::pkg_managers::PkgManager;
use crate::utils::error::Result;

pub struct PackagesModule;

impl CleanModule for PackagesModule {
    fn id(&self) -> &'static str {
        "packages"
    }
    fn name(&self) -> &'static str {
        "Packages"
    }
    fn category(&self) -> Category {
        Category::Packages
    }
    fn description(&self) -> &'static str {
        "Clean package manager caches, orphaned packages, and old kernels"
    }

    fn scan(&self, os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();

        // Package cache
        if let Ok(mut cache_items) = PkgManager::clean_cache(os) {
            items.append(&mut cache_items);
        }

        // Orphaned packages
        if let Ok(mut orphaned) = PkgManager::clean_orphaned(os) {
            items.append(&mut orphaned);
        }

        // Old kernels
        if let Ok(mut kernels) = PkgManager::clean_old_kernels(os) {
            items.append(&mut kernels);
        }

        Ok(items)
    }
}
