use std::collections::{BTreeSet, HashMap};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::core::backup::Backup;
use crate::modules::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::modules::ModuleRegistry;
use crate::os::distro::OSInfo;
use crate::utils::disk::{dir_size, count_files};
use crate::utils::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AutoCleanPhase {
    PreFlightCheck,
    SystemSnapshot,
    HealthAssessment,
    BenchmarkBefore,
    LoadProfile,
    ScanPackages,
    ScanSystem,
    ScanApps,
    ScanPrivacy,
    ScanServices,
    ScanDuplicates,
    ScanContainers,
    ScanDisk,
    ScanOrphans,
    ScanGames,
    ScanIDEs,
    ScanMessaging,
    ScanMailClients,
    ScanDatabases,
    ScanWebServers,
    ScanDotFiles,
    ScanSSHKeys,
    ScanFonts,
    ScanDesktop,
    ScanNetwork,
    Analyze,
    PredictFuture,
    PredictSavings,
    BackupAndClean,
    VerifyAndReport,
}

impl AutoCleanPhase {
    pub fn all() -> &'static [AutoCleanPhase] {
        &[
            AutoCleanPhase::PreFlightCheck,
            AutoCleanPhase::SystemSnapshot,
            AutoCleanPhase::HealthAssessment,
            AutoCleanPhase::BenchmarkBefore,
            AutoCleanPhase::LoadProfile,
            AutoCleanPhase::ScanPackages,
            AutoCleanPhase::ScanSystem,
            AutoCleanPhase::ScanApps,
            AutoCleanPhase::ScanPrivacy,
            AutoCleanPhase::ScanServices,
            AutoCleanPhase::ScanDuplicates,
            AutoCleanPhase::ScanContainers,
            AutoCleanPhase::ScanDisk,
            AutoCleanPhase::ScanOrphans,
            AutoCleanPhase::ScanGames,
            AutoCleanPhase::ScanIDEs,
            AutoCleanPhase::ScanMessaging,
            AutoCleanPhase::ScanMailClients,
            AutoCleanPhase::ScanDatabases,
            AutoCleanPhase::ScanWebServers,
            AutoCleanPhase::ScanDotFiles,
            AutoCleanPhase::ScanSSHKeys,
            AutoCleanPhase::ScanFonts,
            AutoCleanPhase::ScanDesktop,
            AutoCleanPhase::ScanNetwork,
            AutoCleanPhase::Analyze,
            AutoCleanPhase::PredictFuture,
            AutoCleanPhase::PredictSavings,
            AutoCleanPhase::BackupAndClean,
            AutoCleanPhase::VerifyAndReport,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            AutoCleanPhase::PreFlightCheck => "Pre-Flight Check",
            AutoCleanPhase::SystemSnapshot => "System Snapshot",
            AutoCleanPhase::HealthAssessment => "Health Assessment",
            AutoCleanPhase::BenchmarkBefore => "Benchmark (Before)",
            AutoCleanPhase::LoadProfile => "Load Profile",
            AutoCleanPhase::ScanPackages => "Scanning Packages",
            AutoCleanPhase::ScanSystem => "Scanning System",
            AutoCleanPhase::ScanApps => "Scanning Apps",
            AutoCleanPhase::ScanPrivacy => "Scanning Privacy",
            AutoCleanPhase::ScanServices => "Scanning Services",
            AutoCleanPhase::ScanDuplicates => "Scanning Duplicates",
            AutoCleanPhase::ScanContainers => "Scanning Containers",
            AutoCleanPhase::ScanDisk => "Scanning Disk",
            AutoCleanPhase::ScanOrphans => "Scanning Orphans",
            AutoCleanPhase::ScanGames => "Scanning Games",
            AutoCleanPhase::ScanIDEs => "Scanning IDEs",
            AutoCleanPhase::ScanMessaging => "Scanning Messaging",
            AutoCleanPhase::ScanMailClients => "Scanning Email",
            AutoCleanPhase::ScanDatabases => "Scanning Databases",
            AutoCleanPhase::ScanWebServers => "Scanning Web Servers",
            AutoCleanPhase::ScanDotFiles => "Scanning Dotfiles",
            AutoCleanPhase::ScanSSHKeys => "Scanning SSH/GPG Keys",
            AutoCleanPhase::ScanFonts => "Scanning Fonts",
            AutoCleanPhase::ScanDesktop => "Scanning Desktop",
            AutoCleanPhase::ScanNetwork => "Scanning Network",
            AutoCleanPhase::Analyze => "Analyzing",
            AutoCleanPhase::PredictFuture => "Future Prediction",
            AutoCleanPhase::PredictSavings => "Savings Prediction",
            AutoCleanPhase::BackupAndClean => "Backup and Clean",
            AutoCleanPhase::VerifyAndReport => "Verify and Report",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            AutoCleanPhase::PreFlightCheck => "\u{2705}",
            AutoCleanPhase::SystemSnapshot => "\u{1f4ca}",
            AutoCleanPhase::HealthAssessment => "\u{1f3e5}",
            AutoCleanPhase::BenchmarkBefore => "\u{1f4be}",
            AutoCleanPhase::LoadProfile => "\u{1f464}",
            AutoCleanPhase::ScanPackages => "\u{1f4e6}",
            AutoCleanPhase::ScanSystem => "\u{1f9f9}",
            AutoCleanPhase::ScanApps => "\u{1f4bb}",
            AutoCleanPhase::ScanPrivacy => "\u{1f512}",
            AutoCleanPhase::ScanServices => "\u{2699}",
            AutoCleanPhase::ScanDuplicates => "\u{1f517}",
            AutoCleanPhase::ScanContainers => "\u{1f433}",
            AutoCleanPhase::ScanDisk => "\u{1f4be}",
            AutoCleanPhase::ScanOrphans => "\u{1f5d1}",
            AutoCleanPhase::ScanGames => "\u{1f3ae}",
            AutoCleanPhase::ScanIDEs => "\u{1f4dd}",
            AutoCleanPhase::ScanMessaging => "\u{1f4ac}",
            AutoCleanPhase::ScanMailClients => "\u{1f4e8}",
            AutoCleanPhase::ScanDatabases => "\u{1f5c4}",
            AutoCleanPhase::ScanWebServers => "\u{1f310}",
            AutoCleanPhase::ScanDotFiles => "\u{1f3e0}",
            AutoCleanPhase::ScanSSHKeys => "\u{1f511}",
            AutoCleanPhase::ScanFonts => "\u{1f524}",
            AutoCleanPhase::ScanDesktop => "\u{1f5a5}",
            AutoCleanPhase::ScanNetwork => "\u{1f4e1}",
            AutoCleanPhase::Analyze => "\u{1f4ca}",
            AutoCleanPhase::PredictFuture => "\u{1f52e}",
            AutoCleanPhase::PredictSavings => "\u{1f4b0}",
            AutoCleanPhase::BackupAndClean => "\u{1f9f9}",
            AutoCleanPhase::VerifyAndReport => "\u{2705}",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct PhaseHistoryEntry {
    pub phase: AutoCleanPhase,
    pub status: PhaseStatus,
    pub duration: Duration,
    pub items_found: usize,
    pub size_found: u64,
    pub items_cleaned: usize,
    pub size_cleaned: u64,
    pub errors: usize,
}

#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub timestamp: String,
    pub disks: Vec<DiskMetric>,
    pub memory: MemoryMetric,
    pub swap: SwapMetric,
    pub packages: PackageMetric,
    pub uptime_secs: u64,
    pub cpu_load: f32,
    pub process_count: usize,
}

#[derive(Debug, Clone)]
pub struct DiskMetric {
    pub mount: String,
    pub fs_type: String,
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub usage_pct: f32,
}

#[derive(Debug, Clone)]
pub struct MemoryMetric {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub available: u64,
    pub cached: u64,
}

#[derive(Debug, Clone)]
pub struct SwapMetric {
    pub total: u64,
    pub used: u64,
}

#[derive(Debug, Clone)]
pub struct PackageMetric {
    pub total: usize,
    pub managers: Vec<(String, usize)>,
}

#[derive(Debug, Clone)]
pub struct CategoryFindings {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub items: Vec<CleanItem>,
    pub total_size: u64,
    pub total_items: usize,
    pub risk: RiskAssessment,
    pub file_type_groups: Vec<FileTypeGroup>,
    pub age_groups: Vec<AgeGroup>,
    pub sub_groups: Vec<SubGroup>,
    pub top_10: Vec<(usize, CleanItem)>,
    pub stats: SizeStatistics,
}

#[derive(Debug, Clone)]
pub struct RiskAssessment {
    pub level: RiskLevel,
    pub score: u8,
    pub safe_count: usize,
    pub safe_size: u64,
    pub caution_count: usize,
    pub caution_size: u64,
    pub warning_count: usize,
    pub warning_size: u64,
    pub dangerous_count: usize,
    pub dangerous_size: u64,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe,
    Caution,
    Warning,
    Dangerous,
}

impl RiskLevel {
    pub fn label_tr(&self) -> &'static str {
        match self {
            RiskLevel::Safe => "Safe",
            RiskLevel::Caution => "Caution",
            RiskLevel::Warning => "Warning",
            RiskLevel::Dangerous => "Dangerous",
        }
    }

    pub fn color_hex(&self) -> &'static str {
        match self {
            RiskLevel::Safe => "\u{1f7e2}",
            RiskLevel::Caution => "\u{1f7e1}",
            RiskLevel::Warning => "\u{1f7e0}",
            RiskLevel::Dangerous => "\u{1f534}",
        }
    }

    pub fn sort_order(&self) -> u8 {
        match self {
            RiskLevel::Safe => 0,
            RiskLevel::Caution => 1,
            RiskLevel::Warning => 2,
            RiskLevel::Dangerous => 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileTypeGroup {
    pub mime_type: String,
    pub extension: String,
    pub count: usize,
    pub total_size: u64,
    pub percentage: f32,
}

#[derive(Debug, Clone)]
pub struct AgeGroup {
    pub label: String,
    pub min_days: u64,
    pub max_days: u64,
    pub count: usize,
    pub total_size: u64,
}

#[derive(Debug, Clone)]
pub struct SubGroup {
    pub id: String,
    pub name: String,
    pub count: usize,
    pub total_size: u64,
}

#[derive(Debug, Clone)]
pub struct SizeStatistics {
    pub min: u64,
    pub max: u64,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub p10: u64,
    pub p25: u64,
    pub p75: u64,
    pub p90: u64,
    pub p95: u64,
    pub p99: u64,
    pub gini_coefficient: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub priority: u8,
    pub category_id: String,
    pub message: String,
    pub reason: String,
    pub impact_size: u64,
    pub impact_items: usize,
    pub risk: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanError {
    pub phase: AutoCleanPhase,
    pub item_path: String,
    pub message: String,
    pub recoverable: bool,
}

#[derive(Debug, Clone)]
pub enum AutoCleanProgress {
    PhaseChanged(AutoCleanPhase, usize, usize),
    PhaseProgress(f32, String),
    ItemScanned(String, u64),
    ItemCleaned(String, u64, bool),
    CategoryFound(String, CategoryFindings),
    Error(CleanError),
    Warning(String),
    BackupCreated(String),
    Finished(AutoCleanReport),
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCleanReport {
    pub timestamp: String,
    pub duration_secs: f64,
    pub summary: ReportSummary,
    pub categories: Vec<CategoryReport>,
    pub before_after: Vec<DiskComparison>,
    pub errors: Vec<CleanError>,
    pub warnings: Vec<String>,
    pub backup_id: Option<String>,
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total_scanned: usize,
    pub total_cleanable: usize,
    pub total_cleanable_size: u64,
    pub total_cleaned: usize,
    pub total_cleaned_size: u64,
    pub total_failed: usize,
    pub scan_speed_avg: f64,
    pub clean_speed_avg: f64,
    pub disk_improvement_pct: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryReport {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub found_items: usize,
    pub found_size: u64,
    pub cleaned_items: usize,
    pub cleaned_size: u64,
    pub failed_items: usize,
    pub success_rate: f32,
    pub file_types: Vec<(String, usize, u64)>,
    pub age_distribution: Vec<(String, usize, u64)>,
    pub risk_distribution: Vec<(String, usize, u64)>,
    pub top_files: Vec<(String, u64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskComparison {
    pub mount: String,
    pub before_used: u64,
    pub after_used: u64,
    pub before_avail: u64,
    pub after_avail: u64,
    pub before_pct: f32,
    pub after_pct: f32,
    pub freed: u64,
}

#[derive(Debug, Clone)]
pub struct AutoCleanState {
    pub phase: AutoCleanPhase,
    pub phase_index: usize,
    pub total_phases: usize,
    pub phase_progress: f32,
    pub phase_label: String,
    pub sub_label: String,
    pub start_time: Instant,
    pub phase_start: Instant,
    pub estimated_remaining: Duration,
    pub completed_phases: BTreeSet<AutoCleanPhase>,
    pub items_scanned: usize,
    pub bytes_scanned: u64,
    pub scan_speed: f64,
    pub items_cleaned: usize,
    pub items_failed: usize,
    pub items_skipped: usize,
    pub bytes_cleaned: u64,
    pub clean_speed: f64,
    pub all_items: Vec<CleanItem>,
    pub categorized: Vec<CategoryFindings>,
    pub system_before: Option<SystemSnapshot>,
    pub system_after: Option<SystemSnapshot>,
    pub errors: Vec<CleanError>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<Suggestion>,
    pub backup_id: Option<String>,
    pub selected_category_ids: Vec<String>,
    pub cleaning: bool,
    pub done: bool,
    pub cancelled: bool,
    pub phase_history: Vec<PhaseHistoryEntry>,
}

impl Default for AutoCleanState {
    fn default() -> Self {
        Self {
            phase: AutoCleanPhase::PreFlightCheck,
            phase_index: 0,
            total_phases: AutoCleanPhase::all().len(),
            phase_progress: 0.0,
            phase_label: String::new(),
            sub_label: String::new(),
            start_time: Instant::now(),
            phase_start: Instant::now(),
            estimated_remaining: Duration::from_secs(0),
            completed_phases: BTreeSet::new(),
            items_scanned: 0,
            bytes_scanned: 0,
            scan_speed: 0.0,
            items_cleaned: 0,
            items_failed: 0,
            items_skipped: 0,
            bytes_cleaned: 0,
            clean_speed: 0.0,
            all_items: Vec::new(),
            categorized: Vec::new(),
            system_before: None,
            system_after: None,
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
            backup_id: None,
            selected_category_ids: Vec::new(),
            cleaning: false,
            done: false,
            cancelled: false,
            phase_history: Vec::new(),
        }
    }
}

pub struct AutoCleanEngine;

impl AutoCleanEngine {
    pub fn run(
        os: &OSInfo,
        registry: &ModuleRegistry,
        backup: &Backup,
        tx: mpsc::Sender<AutoCleanProgress>,
        safe_only: bool,
        selected_categories: Option<&[String]>,
    ) {
        let start = Instant::now();

        let phases = AutoCleanPhase::all();

        for (idx, phase) in phases.iter().enumerate() {
            if tx.send(AutoCleanProgress::PhaseChanged(*phase, idx, phases.len())).is_err() {
                return;
            }

            let result = Self::execute_phase(*phase, os, registry, backup, safe_only, &tx);

            match result {
                Ok(Some(items)) => {
                    let findings = Self::build_findings(phase, &items);
                    let total_size: u64 = items.iter().map(|i| i.size).sum();
                    let _ = tx.send(AutoCleanProgress::CategoryFound(
                        phase.label().to_string(),
                        findings,
                    ));
                    let _ = tx.send(AutoCleanProgress::PhaseProgress(1.0, format!("{} completed", phase.label())));
                }
                Ok(None) => {}
                Err(e) => {
                    let _ = tx.send(AutoCleanProgress::Error(CleanError {
                        phase: *phase,
                        item_path: String::new(),
                        message: e.to_string(),
                        recoverable: true,
                    }));
                }
            }

            let progress = (idx + 1) as f32 / phases.len() as f32;
            let elapsed = start.elapsed();
            let remaining = if progress > 0.0 {
                Duration::from_secs_f64((elapsed.as_secs_f64() / progress as f64) * (1.0 - progress as f64))
            } else {
                Duration::from_secs(0)
            };
            let _ = tx.send(AutoCleanProgress::PhaseProgress(
                progress,
                format!("Remaining: ~{}.{}min", remaining.as_secs() / 60, (remaining.as_secs() % 60) / 6),
            ));
        }

        let report = AutoCleanReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_secs: start.elapsed().as_secs_f64(),
            summary: ReportSummary {
                total_scanned: 0,
                total_cleanable: 0,
                total_cleanable_size: 0,
                total_cleaned: 0,
                total_cleaned_size: 0,
                total_failed: 0,
                scan_speed_avg: 0.0,
                clean_speed_avg: 0.0,
                disk_improvement_pct: 0.0,
            },
            categories: Vec::new(),
            before_after: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            backup_id: None,
            suggestions: Vec::new(),
        };
        let _ = tx.send(AutoCleanProgress::Finished(report));
    }

    fn execute_phase(
        phase: AutoCleanPhase,
        _os: &OSInfo,
        registry: &ModuleRegistry,
        _backup: &Backup,
        _safe_only: bool,
        _tx: &mpsc::Sender<AutoCleanProgress>,
    ) -> Result<Option<Vec<CleanItem>>> {
        match phase {
            AutoCleanPhase::ScanPackages => {
                let result = registry.scan_category(Category::Packages, _os)?;
                Ok(Some(result.items))
            }
            AutoCleanPhase::ScanSystem => {
                let result = registry.scan_category(Category::System, _os)?;
                Ok(Some(result.items))
            }
            AutoCleanPhase::ScanApps => {
                let result = registry.scan_category(Category::Apps, _os)?;
                Ok(Some(result.items))
            }
            AutoCleanPhase::ScanPrivacy => {
                let result = registry.scan_category(Category::Privacy, _os)?;
                Ok(Some(result.items))
            }
            AutoCleanPhase::ScanServices => {
                let result = registry.scan_category(Category::Services, _os)?;
                Ok(Some(result.items))
            }
            AutoCleanPhase::ScanDuplicates => {
                let result = registry.scan_category(Category::Duplicates, _os)?;
                Ok(Some(result.items))
            }
            AutoCleanPhase::ScanContainers => {
                let result = registry.scan_category(Category::Containers, _os)?;
                Ok(Some(result.items))
            }
            AutoCleanPhase::ScanDisk => {
                let result = registry.scan_category(Category::Disk, _os)?;
                Ok(Some(result.items))
            }
            AutoCleanPhase::ScanOrphans => {
                let module = crate::modules::broken_links::BrokenLinksModule;
                let items = module.scan(_os)?;
                let module2 = crate::modules::empty_files::EmptyFilesModule;
                let items2 = module2.scan(_os)?;
                let combined = items.into_iter().chain(items2).collect();
                Ok(Some(combined))
            }
            AutoCleanPhase::ScanFonts => {
                let module = crate::modules::fonts::FontsModule;
                let items = module.scan(_os)?;
                Ok(Some(items))
            }
            AutoCleanPhase::ScanDesktop => {
                let mut items = Vec::new();
                if let Some(home) = dirs::home_dir() {
                    let cache_dir = home.join(".cache");
                    let mime_dir = cache_dir.join("mime");
                    if mime_dir.exists() {
                        let size = dir_size(&mime_dir);
                        if size > 1024 * 1024 {
                            items.push(CleanItem::new(
                                "mime-cache",
                                &mime_dir.to_string_lossy(),
                                size,
                                true,
                                "MIME type cache",
                                &format!("rm -rf '{}'", mime_dir.display()),
                            ));
                        }
                    }
                    let icon_dir = cache_dir.join("icon-cache.kcache");
                    if icon_dir.exists() {
                        if let Ok(meta) = icon_dir.metadata() {
                            if meta.len() > 1024 * 1024 {
                                items.push(CleanItem::new(
                                    "icon-cache",
                                    &icon_dir.to_string_lossy(),
                                    meta.len(),
                                    true,
                                    "Icon cache",
                                    &format!("rm -f '{}'", icon_dir.display()),
                                ));
                            }
                        }
                    }
                }
                Ok(Some(items))
            }
            AutoCleanPhase::ScanNetwork => {
                let mut items = Vec::new();
                let nm_dir = std::path::Path::new("/etc/NetworkManager/system-connections");
                if nm_dir.exists() {
                    let size = crate::utils::disk::dir_size(nm_dir);
                    items.push(CleanItem::new(
                        "network-configs",
                        &nm_dir.to_string_lossy(),
                        size,
                        false,
                        "Network configs (informational)",
                        "# Informational, will not be deleted",
                    ));
                }
                Ok(Some(items))
            }
            _ => {
                let _ = phase;
                Ok(None)
            }
        }
    }

    fn build_findings(phase: &AutoCleanPhase, items: &[CleanItem]) -> CategoryFindings {
        let total_size: u64 = items.iter().map(|i| i.size).sum();
        let total_items = items.len();

        let safe_count = items.iter().filter(|i| i.safe).count();
        let unsafe_count = items.len() - safe_count;

        let mut sorted = items.to_vec();
        sorted.sort_by(|a, b| b.size.cmp(&a.size));
        let top_10 = sorted.iter().take(10).enumerate().map(|(i, item)| (i, item.clone())).collect();

        let mut ext_map: HashMap<String, (usize, u64)> = HashMap::new();
        for item in items {
            let ext = if let Some(dot) = item.path.rfind('.') {
                item.path[dot..].to_string()
            } else {
                "(none)".to_string()
            };
            let entry = ext_map.entry(ext).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += item.size;
        }
        let mut file_type_groups: Vec<FileTypeGroup> = ext_map.into_iter()
            .map(|(ext, (count, size))| FileTypeGroup {
                mime_type: ext.clone(),
                extension: ext.clone(),
                count,
                total_size: size,
                percentage: if total_size > 0 { size as f32 / total_size as f32 } else { 0.0 },
            })
            .collect();
        file_type_groups.sort_by(|a, b| b.total_size.cmp(&a.total_size));

        let sizes: Vec<u64> = items.iter().map(|i| i.size).collect();
        let stats = compute_stats(&sizes);

        let age_groups = vec![
            AgeGroup { label: "< 7 days".into(), min_days: 0, max_days: 7, count: 0, total_size: 0 },
            AgeGroup { label: "7-30 days".into(), min_days: 7, max_days: 30, count: 0, total_size: 0 },
            AgeGroup { label: "30-90 days".into(), min_days: 30, max_days: 90, count: 0, total_size: 0 },
            AgeGroup { label: "> 90 days".into(), min_days: 90, max_days: u64::MAX, count: 0, total_size: 0 },
        ];

        CategoryFindings {
            id: phase.label().to_string(),
            name: phase.label().to_string(),
            icon: phase.icon().to_string(),
            items: items.to_vec(),
            total_size,
            total_items,
            risk: RiskAssessment {
                level: if unsafe_count == 0 { RiskLevel::Safe } else if unsafe_count < 5 { RiskLevel::Caution } else { RiskLevel::Warning },
                score: if unsafe_count == 0 { 0 } else { (unsafe_count as u8 * 255 / total_items.max(1) as u8).min(100) },
                safe_count,
                safe_size: items.iter().filter(|i| i.safe).map(|i| i.size).sum(),
                caution_count: unsafe_count / 2,
                caution_size: 0,
                warning_count: unsafe_count / 3,
                warning_size: 0,
                dangerous_count: 0,
                dangerous_size: 0,
                reasons: Vec::new(),
            },
            file_type_groups,
            age_groups,
            sub_groups: Vec::new(),
            top_10,
            stats,
        }
    }
}

fn compute_stats(sizes: &[u64]) -> SizeStatistics {
    if sizes.is_empty() {
        return SizeStatistics {
            min: 0, max: 0, mean: 0.0, median: 0.0, std_dev: 0.0,
            p10: 0, p25: 0, p75: 0, p90: 0, p95: 0, p99: 0, gini_coefficient: 0.0,
        };
    }
    let mut sorted = sizes.to_vec();
    sorted.sort();
    let len = sorted.len();
    let sum: u64 = sorted.iter().sum();
    let mean = sum as f64 / len as f64;
    let variance = sorted.iter().map(|s| {
        let diff = *s as f64 - mean;
        diff * diff
    }).sum::<f64>() / len as f64;
    let std_dev = variance.sqrt();

    let p = |pct: f64| -> u64 {
        let idx = (len as f64 * pct / 100.0).ceil() as usize;
        sorted[idx.min(len - 1)]
    };

    let median = if len % 2 == 0 {
        (sorted[len/2 - 1] + sorted[len/2]) as f64 / 2.0
    } else {
        sorted[len/2] as f64
    };

    let gini = if sum > 0 {
        let mut numerator = 0u64;
        for (i, &s) in sorted.iter().enumerate() {
            let add = 2 * (i as i64) + 1 - (len as i64);
            if add >= 0 {
                numerator = numerator.wrapping_add(s.wrapping_mul(add as u64));
            } else {
                numerator = numerator.wrapping_sub(s.wrapping_mul(add.unsigned_abs()));
            }
        }
        numerator as f64 / (len as f64 * sum as f64)
    } else {
        0.0
    };

    SizeStatistics {
        min: sorted[0],
        max: sorted[len - 1],
        mean,
        median,
        std_dev,
        p10: p(10.0),
        p25: p(25.0),
        p75: p(75.0),
        p90: p(90.0),
        p95: p(95.0),
        p99: p(99.0),
        gini_coefficient: gini.max(0.0),
    }
}
