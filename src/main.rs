mod app;
mod cli;
mod config;
mod core;
mod modules;
mod os;
mod tui;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};
use std::io::{Write, Read};
use utils::error::Result;

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let log_level = if cli.quiet {
        "error"
    } else {
        match cli.verbose {
            0 => "warn",
            1 => "info",
            _ => "debug",
        }
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .format_timestamp_secs()
        .init();

    if cli.verbose > 0 {
        println!("[DEBUG] WinDebloat v{} starting...", env!("CARGO_PKG_VERSION"));
    }

    match &cli.command {
        Some(Commands::Tui { .. }) | None => {
            let mut app = app::App::new(&cli);
            app.run_tui()?;
        }
        Some(Commands::Scan { category, format, min_size, safe_only, json }) => {
            let app = app::App::new(&cli);
            run_cli_scan(app, category, format, min_size.clone(), *safe_only, *json)?;
        }
        Some(Commands::Clean { category, yes, dry_run, format: _ }) => {
            let app = app::App::new(&cli);
            run_cli_clean(app, category, *yes, *dry_run)?;
        }
        Some(Commands::Batch { stdin, dry_run }) => {
            run_batch_mode(*stdin, *dry_run)?;
        }
        Some(Commands::Export { output, category }) => {
            let app = app::App::new(&cli);
            run_export(app, output, category)?;
        }
        Some(Commands::Import { input }) => {
            run_import(input)?;
        }
        Some(Commands::Undo { id, list, verify: _ }) => {
            let app = app::App::new(&cli);
            run_cli_undo(app, id, *list)?;
        }
        Some(Commands::Status) => {
            let app = app::App::new(&cli);
            run_cli_status(app)?;
        }
        Some(Commands::Logs { count }) => {
            let app = app::App::new(&cli);
            run_cli_logs(app, *count)?;
        }
            Some(Commands::Config { action }) => {
                match action {
                    Some(cli::ConfigAction::Show) => {
                        let config = config::loader::load_config();
                        println!("{}", toml::to_string_pretty(&config).unwrap());
                    }
                    Some(cli::ConfigAction::Edit) => {
                        let path = config::loader::config_path();
                        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
                        let status = std::process::Command::new(&editor)
                            .arg(&path)
                            .status()
                            .map_err(|e| utils::error::AppError::Other(format!("Editor error: {}", e)))?;
                        if !status.success() {
                            eprintln!("Editor exited with error.");
                        }
                    }
                    Some(cli::ConfigAction::Reset) => {
                        let path = config::loader::config_path();
                        std::fs::write(&path, toml::to_string_pretty(&config::schema::Config::default()).unwrap())
                            .map_err(|e| utils::error::AppError::Io(e))?;
                        println!("Config reset to defaults.");
                    }
                    None => {
                        let path = config::loader::config_path();
                        println!("Config path: {}", path.display());
                    }
                }
            }
            Some(Commands::AutoClean { categories, yes, dry_run, safe_only, format, output, profile, benchmark, predict, min_size, max_size, install_scheduler, remove_scheduler, verbose }) => {
                run_cli_auto_clean(categories, *yes, *dry_run, *safe_only, format, output, profile, *benchmark, *predict, min_size, max_size, *install_scheduler, *remove_scheduler, *verbose)?;
            }
    }

    Ok(())
}

fn run_cli_scan(
    app: app::App,
    category: &str,
    format: &str,
    min_size: Option<String>,
    safe_only: bool,
    json: bool,
) -> Result<()> {
    use crate::modules::ModuleRegistry;
    use crate::core::scanner::Category as ScanCategory;
    use crate::utils::formatting::parse_size;

    let registry = ModuleRegistry::new();
    let os = app.os;

    let min_size_bytes = min_size.and_then(|s| parse_size(&s));

    let categories: Vec<ScanCategory> = if category == "all" {
        ScanCategory::all().to_vec()
    } else {
        let cat = match category {
            "packages" => Some(ScanCategory::Packages),
            "system" => Some(ScanCategory::System),
            "apps" => Some(ScanCategory::Apps),
            "privacy" => Some(ScanCategory::Privacy),
            "services" => Some(ScanCategory::Services),
            "duplicates" => Some(ScanCategory::Duplicates),
            "containers" => Some(ScanCategory::Containers),
            "disk" => Some(ScanCategory::Disk),
            "broken-links" => Some(ScanCategory::BrokenLinks),
            "empty-files" => Some(ScanCategory::EmptyFiles),
            "old-downloads" => Some(ScanCategory::OldDownloads),
            "old-logs" => Some(ScanCategory::OldLogs),
            "fonts" => Some(ScanCategory::Fonts),
            "kernel" => Some(ScanCategory::Kernel),
            "dev-tools" => Some(ScanCategory::DevTools),
            _ => {
                eprintln!("Invalid category: {}. Use: packages, system, apps, privacy, services, duplicates, containers, disk, broken-links, empty-files, old-downloads, old-logs, fonts, kernel, dev-tools, all", category);
                return Ok(());
            }
        };
        cat.into_iter().collect()
    };

    println!("Scanning for cleanable items...");
    let mut total_size: u64 = 0;
    let mut total_items: usize = 0;

    for cat in &categories {
        match registry.scan_category(*cat, &os) {
            Ok(result) => {
                let filtered_items: Vec<_> = result.items.iter()
                    .filter(|item| {
                        if safe_only && !item.safe { return false; }
                        if let Some(min) = min_size_bytes {
                            if item.size < min { return false; }
                        }
                        true
                    })
                    .cloned()
                    .collect();

                let filtered_size: u64 = filtered_items.iter().map(|i| i.size).sum();
                total_size += filtered_size;
                total_items += filtered_items.len();

                if json {
                    let filtered_result = crate::core::scanner::ScanResult::new(*cat, filtered_items);
                    println!("{}", serde_json::to_string_pretty(&filtered_result).unwrap());
                } else {
                    match format {
                        "table" => {
                            println!("\n╔══════════════════════════════════════════════════════════════════════════════╗");
                            println!("║ {} ({}) ", cat.name(), filtered_items.len());
                            println!("╠══════════════════════════════════════════════════════════════════════════════╣");
                            for item in &filtered_items {
                                let safe_mark = if item.safe { "✓" } else { "✗" };
                                println!("║ [{}] {} ", safe_mark, item.display_path());
                                println!("║      Size: {} | {}", item.size_formatted(), item.description);
                            }
                            println!("╚══════════════════════════════════════════════════════════════════════════════╝");
                        }
                        _ => {
                            println!("\n=== {} ({}) ===", cat.name(), filtered_items.len());
                            for item in &filtered_items {
                                let safe_mark = if item.safe { "[✓]" } else { "[!]" };
                                println!("  {} [{}] {} - {}", 
                                    safe_mark,
                                    if item.selected { "x" } else { " " },
                                    item.display_path(),
                                    item.size_formatted());
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error scanning {}: {}", cat.name(), e.user_message());
            }
        }
    }

    println!("\n--- Total: {} items ({} / {}) ---", 
        total_items, total_size, crate::utils::formatting::format_bytes(total_size));
    
    if min_size_bytes.is_some() {
        println!("(Filtered by minimum size)");
    }
    if safe_only {
        println!("(Safe items only)");
    }
    
    println!("\nUse 'windebloat clean --category {}' to clean.", category);

    Ok(())
}

fn run_cli_clean(app: app::App, category: &str, yes: bool, dry_run: bool) -> Result<()> {
    use crate::modules::ModuleRegistry;
    use crate::core::scanner::Category as ScanCategory;
    use crate::core::cleaner::Cleaner;
    use crate::core::backup::Backup;

    let registry = ModuleRegistry::new();
    let os = app.os;
    let config = app.config;

    let categories: Vec<ScanCategory> = if category == "all" {
        ScanCategory::all().to_vec()
    } else {
        let cat = match category {
            "packages" => Some(ScanCategory::Packages),
            "system" => Some(ScanCategory::System),
            "apps" => Some(ScanCategory::Apps),
            "privacy" => Some(ScanCategory::Privacy),
            "services" => Some(ScanCategory::Services),
            "duplicates" => Some(ScanCategory::Duplicates),
            "containers" => Some(ScanCategory::Containers),
            "disk" => Some(ScanCategory::Disk),
            _ => {
                eprintln!("Invalid category: {}. Use: packages, system, apps, privacy, services, duplicates, containers, disk, all", category);
                return Ok(());
            }
        };
        cat.into_iter().collect()
    };

    println!("Scanning for cleanable items...");
    let mut all_items = Vec::new();

    for cat in &categories {
        match registry.scan_category(*cat, &os) {
            Ok(result) => {
                for mut item in result.items {
                    item.selected = true;
                    all_items.push(item);
                }
            }
            Err(e) => {
                eprintln!("Error scanning {}: {}", cat.name(), e.user_message());
            }
        }
    }

    if all_items.is_empty() {
        println!("No items to clean.");
        return Ok(());
    }

    let total_size: u64 = all_items.iter().map(|i| i.size).sum();
    println!("Found {} items ({} bytes / {})", 
        all_items.len(), total_size, crate::utils::formatting::format_bytes(total_size));

    if dry_run {
        println!("\n--- DRY RUN (no changes made) ---");
        for item in &all_items {
            println!("  Would clean: {} ({})", item.display_path(), item.size_formatted());
        }
        return Ok(());
    }

    if !yes {
        println!("\nProceed with cleaning? [y/N] ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().to_lowercase().starts_with('y') {
            println!("Aborted.");
            return Ok(());
        }
    }

    println!("\nCleaning...");
    let backup = Backup::new(config.backup.location.clone(), config.backup.enabled);
    let cleaner = Cleaner::new(backup);

    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = cleaner.execute(&all_items, tx);
    });

    let mut cleaned = 0;
    loop {
        match rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(crate::core::cleaner::CleanProgress::File(path)) => {
                print!("\rCleaning: {}", path);
                std::io::stdout().flush().ok();
            }
            Ok(crate::core::cleaner::CleanProgress::Done(count)) => {
                cleaned = count;
            }
            Ok(crate::core::cleaner::CleanProgress::Error(e)) => {
                eprintln!("\nError: {}\nRun with --help for usage information.", e);
            }
            Ok(crate::core::cleaner::CleanProgress::Finished) => {
                break;
            }
            Err(_) => continue,
        }
    }

    println!("\n\nDone! Cleaned {} items.", cleaned);

    Ok(())
}

fn run_cli_undo(app: app::App, id: &Option<String>, list: bool) -> Result<()> {
    let backup = app.backup;

    if list {
        match backup.list_backups() {
            Ok(backups) => {
                if backups.is_empty() {
                    println!("No backups found.");
                    return Ok(());
                }
                println!("\n=== Available Backups ===\n");
                for (i, b) in backups.iter().enumerate() {
                    println!("[{}] {}", i, b.id);
                    println!("    Date: {}", b.timestamp);
                    println!("    Items: {}", b.items.len());
                    println!("    Size: {} bytes / {}", 
                        b.total_size, 
                        crate::utils::formatting::format_bytes(b.total_size));
                    println!("    Description: {}", b.description);
                    println!();
                }
                println!("To restore a backup, run: windebloat undo --id <id>");
            }
            Err(e) => {
                eprintln!("Error listing backups: {}\nUse 'windebloat undo --list' to see available backups.", e.user_message());
            }
        }
        return Ok(());
    }

    match id {
        Some(backup_id) => {
            print!("Restoring backup {}... ", backup_id);
            match backup.restore_backup(backup_id) {
                Ok(()) => {
                    println!("Done!");
                    println!("Backup restored successfully.");
                }
                Err(e) => {
                    println!("Failed!");
                    eprintln!("Error: {}\nRun 'windebloat help' for usage information.", e.user_message());
                }
            }
        }
        None => {
            eprintln!("Please specify a backup ID. Use --list to see available backups.");
            eprintln!("Usage: windebloat undo --id <backup-id>");
        }
    }

    Ok(())
}

fn run_cli_status(app: app::App) -> Result<()> {
    use crate::utils::formatting::format_bytes;
    use sysinfo::{System, Disks};

    let os = &app.os;

    println!("╔══════════════════════════════════════════╗");
    println!("║       WinDebloat System Info             ║");
    println!("╚══════════════════════════════════════════╝");
    println!();

    println!("📀 Operating System:");
    println!("   Name:    {}", os.name);
    println!("   Version: {}", os.version.as_deref().unwrap_or("unknown"));
    println!("   Family:  {:?}", os.family);
    println!();

    println!("💾 Disk Usage:");
    let disks = Disks::new_with_refreshed_list();
    for disk in disks.list() {
        let total = disk.total_space();
        let free = disk.available_space();
        let used = total.saturating_sub(free);
        let usage = if total > 0 { (used as f64 / total as f64 * 100.0) as u32 } else { 0 };
        println!("   {}: {} / {} ({}% used)",
            disk.mount_point().to_string_lossy(),
            format_bytes(used),
            format_bytes(total),
            usage);
    }
    println!();

    let mut sys = System::new_all();
    sys.refresh_memory();
    println!("🧠 Memory:");
    println!("   Used:    {}", format_bytes(sys.used_memory()));
    println!("   Total:   {}", format_bytes(sys.total_memory()));
    println!();

    println!("📦 Package Manager:");
    match os.family {
        crate::os::distro::DistroFamily::Debian => {
            if let Ok(output) = std::process::Command::new("dpkg").arg("-l").output() {
                let count = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .filter(|l| l.starts_with("ii"))
                    .count();
                println!("   dpkg:    {} packages installed", count);
            }
        }
        crate::os::distro::DistroFamily::Arch => {
            if let Ok(output) = std::process::Command::new("pacman").args(["-Qq"]).output() {
                let count = String::from_utf8_lossy(&output.stdout).lines().count();
                println!("   pacman:  {} packages installed", count);
            }
        }
        crate::os::distro::DistroFamily::Fedora => {
            if let Ok(output) = std::process::Command::new("rpm").args(["-qa"]).output() {
                let count = String::from_utf8_lossy(&output.stdout).lines().count();
                println!("   rpm:     {} packages installed", count);
            }
        }
        crate::os::distro::DistroFamily::OpenSuse => {
            println!("   zypper:  (use 'zypper pa -i')");
        }
        crate::os::distro::DistroFamily::Alpine => {
            if let Ok(output) = std::process::Command::new("apk").args(["info"]).output() {
                let count = String::from_utf8_lossy(&output.stdout).lines().count();
                println!("   apk:     {} packages installed", count);
            }
        }
        _ => println!("   Unknown package manager")
    }
    println!();

    println!("🐳 Containers:");
    if std::process::Command::new("docker").arg("--version").output().is_ok() {
        println!("   Docker:  installed");
    }
    if std::process::Command::new("podman").arg("--version").output().is_ok() {
        println!("   Podman:  installed");
    }
    println!();

    println!("⚙️  Configuration:");
    println!("   Config:  {:?}", crate::config::loader::config_path());
    println!("   Backup:  {:?}", app.config.backup.location);
    println!("   Safe:    {}", app.config.safe.enabled);
    println!();

    println!("🗂️  Modules Available:");
    let registry = crate::modules::ModuleRegistry::new();
    println!("   {} modules registered", registry.modules.len());

    Ok(())
}

fn run_cli_logs(app: app::App, count: usize) -> Result<()> {
    let logger = app.logger;
    let logs = logger.get_logs(count);

    println!("╔══════════════════════════════════════════╗");
    println!("║       WinDebloat Action Logs             ║");
    println!("╚══════════════════════════════════════════╝");
    println!();

    if logs.is_empty() {
        println!("No log entries found.");
        println!("Log file: {:?}", logger.log_path());
        return Ok(());
    }

    for log in logs {
        println!("{}", log);
    }

    println!();
    println!("Log file: {:?}", logger.log_path());

    Ok(())
}

fn run_batch_mode(stdin: bool, dry_run: bool) -> Result<()> {
    let paths: Vec<String> = if stdin {
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input)?;
        input.lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        println!("Usage: windebloat batch --stdin < paths.txt");
        return Ok(());
    };

    println!("Batch mode: processing {} paths", paths.len());

    for path in &paths {
        if !std::path::Path::new(path).exists() {
            println!("  [SKIP] {} (not found)", path);
            continue;
        }

        let meta = std::fs::metadata(path)?;
        let size = if meta.is_dir() {
            crate::utils::disk::dir_size(std::path::Path::new(path))
        } else {
            meta.len()
        };

        if dry_run {
            println!("  [DRY-RUN] Would clean: {} ({} bytes)", path, size);
        } else {
            println!("  [READY] {} ({} bytes)", path, size);
        }
    }

    if dry_run {
        println!("\n-- Dry run complete. No changes made --");
    } else {
        println!("\nReview the list above. Run with --dry-run to test first.");
    }

    Ok(())
}

fn run_export(app: app::App, output: &str, category: &str) -> Result<()> {
    use crate::modules::ModuleRegistry;
    use crate::core::scanner::Category as ScanCategory;

    let registry = ModuleRegistry::new();
    let os = app.os;

    let categories: Vec<ScanCategory> = if category == "all" {
        ScanCategory::all().to_vec()
    } else {
        let cat = match category {
            "packages" => Some(ScanCategory::Packages),
            "system" => Some(ScanCategory::System),
            "apps" => Some(ScanCategory::Apps),
            "privacy" => Some(ScanCategory::Privacy),
            "services" => Some(ScanCategory::Services),
            "duplicates" => Some(ScanCategory::Duplicates),
            "containers" => Some(ScanCategory::Containers),
            "disk" => Some(ScanCategory::Disk),
            "broken-links" => Some(ScanCategory::BrokenLinks),
            "empty-files" => Some(ScanCategory::EmptyFiles),
            "old-downloads" => Some(ScanCategory::OldDownloads),
            "old-logs" => Some(ScanCategory::OldLogs),
            "fonts" => Some(ScanCategory::Fonts),
            "kernel" => Some(ScanCategory::Kernel),
            "dev-tools" => Some(ScanCategory::DevTools),
            _ => None,
        };
        cat.into_iter().collect()
    };

    let mut all_results = Vec::new();

    for cat in &categories {
        if let Ok(result) = registry.scan_category(*cat, &os) {
            all_results.push(result);
        }
    }

    let json = serde_json::to_string_pretty(&all_results).map_err(|e| {
        utils::error::AppError::Other(format!("JSON error: {}", e))
    })?;

    std::fs::write(output, &json)?;
    println!("Exported {} categories to {}", all_results.len(), output);

    Ok(())
}

fn run_import(input: &str) -> Result<()> {
    let content = std::fs::read_to_string(input)?;
    let results: Vec<crate::core::scanner::ScanResult> = serde_json::from_str(&content)
        .map_err(|e| utils::error::AppError::Other(format!("Parse error: {}", e)))?;

    let total_size: u64 = results.iter().map(|r| r.total_size).sum();
    let total_items: usize = results.iter().map(|r| r.total_items).sum();

    println!("Imported scan results from {}", input);
    println!("  Categories: {}", results.len());
    println!("  Total items: {}", total_items);
    println!("  Total size: {}", crate::utils::formatting::format_bytes(total_size));

    for result in &results {
        println!("  - {}: {} items ({} bytes)", 
            result.category.name(), 
            result.total_items, 
            result.total_size);
    }

    Ok(())
}

fn run_cli_auto_clean(
    categories: &str,
    yes: bool,
    dry_run: bool,
    safe_only: bool,
    format: &str,
    output: &Option<String>,
    profile: &str,
    benchmark: bool,
    predict: bool,
    min_size: &Option<String>,
    max_size: &Option<String>,
    install_scheduler: bool,
    remove_scheduler: bool,
    verbose: bool,
) -> Result<()> {
    use crate::core::auto_clean::{AutoCleanEngine, AutoCleanProgress};
    use crate::modules::ModuleRegistry;
    use crate::os::distro::OSInfo;
    use crate::core::backup::Backup;
    use std::sync::mpsc;

    println!("╔══════════════════════════════════════════╗");
    println!("║     WinDebloat — ULTRA AUTO CLEAN        ║");
    println!("╚══════════════════════════════════════════╝");
    println!();

    if install_scheduler {
        println!("Installing auto-clean scheduler...");
        let scheduler = crate::core::scheduler::Scheduler::new();
        match scheduler.install_cron("0 3 * * *", "windebloat auto-clean --yes") {
            Ok(()) => println!("✅ Scheduler installed. Runs daily at 3 AM."),
            Err(e) => eprintln!("❌ Failed: {}", e),
        }
        return Ok(());
    }

    if remove_scheduler {
        println!("Removing auto-clean scheduler...");
        let scheduler = crate::core::scheduler::Scheduler::new();
        match scheduler.remove_cron("windebloat auto-clean") {
            Ok(()) => println!("✅ Scheduler removed."),
            Err(e) => eprintln!("❌ Could not remove: {}", e),
        }
        return Ok(());
    }

    let os = OSInfo::detect();
    let registry = ModuleRegistry::new();
    let config = crate::config::loader::load_config();
    let backup = Backup::new(config.backup.location.clone(), config.backup.enabled);

    println!("📀 OS: {} {}", os.name, os.version.as_deref().unwrap_or(""));
    println!("📦 Modules: {}", registry.modules.len());
    println!("👤 Profile: {}", profile);
    if safe_only { println!("🛡  Safe-only: ON"); }
    if dry_run { println!("👁  DRY RUN — no changes will be made"); }
    println!();

    let (tx, rx) = mpsc::channel();
    let os_clone = os.clone();
    let reg_clone = ModuleRegistry::new();
    let backup_clone = Backup::new(config.backup.location.clone(), config.backup.enabled);

    std::thread::spawn(move || {
        AutoCleanEngine::run(&os_clone, &reg_clone, &backup_clone, tx, safe_only, None);
    });

    let mut report_opt = None;

    loop {
        match rx.recv() {
            Ok(AutoCleanProgress::PhaseChanged(phase, idx, total)) => {
                if verbose {
                    println!("  [{}/{}] {} {}", idx + 1, total, phase.icon(), phase.label());
                } else {
                    print!("\r  Phase {}/{}: {} ...", idx + 1, total, phase.label());
                    std::io::stdout().flush().ok();
                }
            }
            Ok(AutoCleanProgress::PhaseProgress(pct, label)) => {
                if verbose {
                    println!("    {:>5.1}% - {}", pct * 100.0, label);
                }
            }
            Ok(AutoCleanProgress::CategoryFound(name, findings)) => {
                println!("\n  📁 {} ({}): {} items, {}",
                    name,
                    findings.icon,
                    findings.total_items,
                    crate::utils::formatting::format_bytes(findings.total_size));
                for item in &findings.items[..std::cmp::min(findings.items.len(), 5)] {
                    println!("    {} {} ({})",
                        if item.safe { "✓" } else { "⚠" },
                        item.display_path(),
                        item.size_formatted());
                }
            }
            Ok(AutoCleanProgress::Error(err)) => {
                eprintln!("\n  ❌ Error in {:?}: {}", err.phase, err.message);
            }
            Ok(AutoCleanProgress::Warning(warn)) => {
                println!("\n  ⚠ Warning: {}", warn);
            }
            Ok(AutoCleanProgress::Finished(r)) => {
                report_opt = Some(r);
                break;
            }
            Ok(AutoCleanProgress::Cancelled) => {
                println!("\n  ❌ Auto Clean cancelled.");
                return Ok(());
            }
            _ => {}
        }
    }

    if dry_run {
        println!("\n\n✅ DRY RUN complete. No changes made.");
        return Ok(());
    }

    if let Some(report) = report_opt {
        println!("\n\n📊 Auto Clean Report:");
        println!("  Duration: {:.1}s", report.duration_secs);
        println!("  Total scanned: {}", report.summary.total_scanned);
        println!();
        println!("  Categories scanned: {}", report.categories.len());

        match format {
            "json" => {
                let json = report.to_json();
                println!("{}", json);
                if let Some(path) = output {
                    std::fs::write(path, &json)?;
                    println!("Report saved to {}", path);
                }
            }
            "markdown" => {
                let md = report.to_markdown();
                println!("{}", md);
                if let Some(path) = output {
                    std::fs::write(path, &md)?;
                    println!("Report saved to {}", path);
                }
            }
            _ => {
                let text = report.to_text();
                println!("{}", text);
                if let Some(path) = output {
                    std::fs::write(path, &text)?;
                    println!("Report saved to {}", path);
                }
            }
        }

        if predict {
            println!("\n🔮 Growth Predictions:\n");
            println!("  (Requires system snapshots for accurate predictions)");
        }

        if benchmark {
            println!("\n💾 Benchmark Results:\n");
            let bench_result = crate::core::benchmark::Benchmark::run();
            println!("  Read:  {:.0} MB/s", bench_result.read_speed_mbps);
            println!("  Write: {:.0} MB/s", bench_result.write_speed_mbps);
            println!("  IOPS:  {:.0}", bench_result.iops);
        }

        if !yes {
            println!("\nProceed with cleaning? [y/N] ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().to_lowercase().starts_with('y') {
                println!("Aborted.");
                return Ok(());
            }
        }
    }

    println!("\n✨ Auto Clean complete!");
    Ok(())
}
