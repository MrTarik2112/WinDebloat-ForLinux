pub mod apps;
pub mod broken_links;
pub mod containers;
pub mod databases;
pub mod dev_tools;
pub mod disk;
pub mod dotfiles;
pub mod duplicates;
pub mod empty_files;
pub mod fonts;
pub mod games;
pub mod ides;
pub mod kernel;
pub mod logs;
pub mod mail_clients;
pub mod messaging;
pub mod old_downloads;
pub mod old_logs;
pub mod packages;
pub mod pkg_stats;
pub mod privacy;
pub mod services;
pub mod ssh_keys;
pub mod system;
pub mod web_servers;

use super::core::scanner::{Category, CleanItem, ScanResult};
use super::os::distro::OSInfo;
use super::utils::error::Result;
use rayon::prelude::*;

pub trait CleanModule: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn category(&self) -> Category;
    fn description(&self) -> &'static str;
    fn is_available(&self, _os: &OSInfo) -> bool { true }
    fn scan(&self, os: &OSInfo) -> Result<Vec<CleanItem>>;
}

pub struct ModuleRegistry {
    pub modules: Vec<Box<dyn CleanModule>>,
}

impl ModuleRegistry {
    pub fn module_count(&self) -> usize {
        self.modules.len()
    }

    pub fn new() -> Self {
        let modules: Vec<Box<dyn CleanModule>> = vec![
            Box::new(packages::PackagesModule),
            Box::new(system::SystemModule),
            Box::new(logs::LogsModule),
            Box::new(apps::AppsModule),
            Box::new(privacy::PrivacyModule),
            Box::new(services::ServicesModule),
            Box::new(services::StartupModule),
            Box::new(duplicates::DuplicatesModule),
            Box::new(disk::DiskModule::new(100 * 1024 * 1024)),
            Box::new(containers::ContainerModule),
            Box::new(pkg_stats::PkgStatsModule),
            Box::new(broken_links::BrokenLinksModule),
            Box::new(empty_files::EmptyFilesModule),
            Box::new(old_downloads::OldDownloadsModule::default()),
            Box::new(old_logs::OldLogsModule::default()),
            Box::new(fonts::FontsModule),
            Box::new(kernel::KernelModule),
            Box::new(dev_tools::DevToolsModule),
            Box::new(games::GamesModule),
            Box::new(ides::IDEsModule),
            Box::new(messaging::MessagingModule),
            Box::new(mail_clients::MailClientsModule),
            Box::new(databases::DatabasesModule),
            Box::new(web_servers::WebServersModule),
            Box::new(dotfiles::DotFilesModule),
            Box::new(ssh_keys::SSHKeysModule),
        ];
        ModuleRegistry { modules }
    }

    pub fn scan_all(&self, os: &OSInfo) -> Result<Vec<ScanResult>> {
        let mut results = Vec::new();
        for module in &self.modules {
            if module.is_available(os) {
                if let Ok(items) = module.scan(os) {
                    let result = ScanResult::new(module.category(), items);
                    results.push(result);
                }
            }
        }
        Ok(results)
    }

    pub fn scan_all_parallel(&self, os: &OSInfo) -> Vec<ScanResult> {
        let available_modules: Vec<_> = self.modules.iter()
            .filter(|m| m.is_available(os))
            .collect();

        available_modules.par_iter()
            .filter_map(|module| {
                match module.scan(os) {
                    Ok(items) => Some(ScanResult::new(module.category(), items)),
                    Err(_) => None,
                }
            })
            .collect()
    }

    pub fn scan_category(&self, category: Category, os: &OSInfo) -> Result<ScanResult> {
        for module in &self.modules {
            if module.category() == category && module.is_available(os) {
                let items = module.scan(os)?;
                return Ok(ScanResult::new(category, items));
            }
        }
        Ok(ScanResult::new(category, vec![]))
    }
}
