use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::cli::Cli;
use crate::config::loader::load_config;
use crate::config::schema::Config;
use crate::core::auto_clean::{AutoCleanEngine, AutoCleanPhase, AutoCleanProgress, AutoCleanReport, AutoCleanState, CategoryFindings, RiskLevel};
use crate::core::backup::Backup;
use crate::core::cleaner::{CleanProgress, Cleaner};
use crate::core::logger::Logger;
use crate::core::scanner::{Category, CleanItem, ScanResult};
use crate::modules::ModuleRegistry;
use crate::os::distro::OSInfo;
use crate::tui::components::dialogs::{AutoCleanStep, Dialog, DiskWizardState, DiskAnalysis, CategoryInfo};
use crate::tui::ui::AppUi;
use crate::utils::disk::dir_size;
use crate::utils::error::Result;
use crate::utils::formatting::format_bytes;

use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use std::io::{stdout, IsTerminal};

#[derive(Debug, Clone)]
enum ScanMsg {
    CategoryDone(Category, ScanResult),
    AllDone(Vec<ScanResult>),
    Error(String),
}

pub struct App {
    pub os: OSInfo,
    pub config: Config,
    pub registry: ModuleRegistry,
    pub logger: Logger,
    pub backup: Backup,
    pub ui: Option<AppUi>,
    pub scan_results: Vec<ScanResult>,
    pub running: bool,
    pub scanning: bool,
    pub scan_start: Option<Instant>,
    pub scan_rx: Option<mpsc::Receiver<ScanMsg>>,
    pub scan_total: usize,
    pub scan_count: usize,
    pub auto_clean_state: Option<AutoCleanState>,
    pub auto_clean_rx: Option<mpsc::Receiver<AutoCleanProgress>>,
    pub auto_clean_report: Option<AutoCleanReport>,
    pub auto_clean_review_cursor: usize,
    pub auto_clean_review_selected: Vec<String>,
}

impl App {
    pub fn new(_cli: &Cli) -> Self {
        let config = load_config();
        let os = OSInfo::detect();

        let backup = Backup::new(
            config.backup.location.clone(),
            config.backup.enabled,
        );

        App {
            os,
            config,
            registry: ModuleRegistry::new(),
            logger: Logger::new(),
            backup,
            ui: None,
            scan_results: Vec::new(),
            running: true,
            scanning: false,
            scan_start: None,
            scan_rx: None,
            scan_total: 0,
            scan_count: 0,
            auto_clean_state: None,
            auto_clean_rx: None,
            auto_clean_report: None,
            auto_clean_review_cursor: 0,
            auto_clean_review_selected: Vec::new(),
        }
    }

    pub fn run_tui(&mut self) -> Result<()> {
        if !stdout().is_terminal() {
            return Err(crate::utils::error::AppError::NotSupported(
                "WinDebloat requires an interactive terminal to run the TUI. Use CLI commands instead.".to_string(),
            ));
        }
        enable_raw_mode()?;
        let mut stdout = stdout();
        stdout.execute(EnterAlternateScreen)?;
        stdout.execute(crossterm::event::EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        let ui = AppUi::new(
            &format!("{} {}", self.os.name, self.os.version.as_deref().unwrap_or("")),
            self.config.ui.clone(),
        );

        self.ui = Some(ui);

        let result = self.tui_event_loop(&mut terminal);

        disable_raw_mode()?;
        let _ = terminal.backend_mut().execute(crossterm::event::DisableMouseCapture);
        let _ = terminal.backend_mut().execute(LeaveAlternateScreen);
        let _ = terminal.show_cursor();

        result
    }

    fn tui_event_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        let poll_rate = Duration::from_millis(50);
        let mut redraw_counter = 0u32;

        while self.running {
            let active_category = self.ui
                .as_ref()
                .and_then(|ui| ui.sidebar.selected())
                .unwrap_or(Category::Packages);

            terminal.draw(|f| {
                let ui = self.ui.as_mut().unwrap();
                ui.render(
                    f.area(),
                    f.buffer_mut(),
                    &self.scan_results,
                    active_category,
                );
            })?;

            redraw_counter += 1;

            if self.scanning {
                if let Some(ref rx) = self.scan_rx {
                    match rx.try_recv() {
                        Ok(ScanMsg::CategoryDone(cat, result)) => {
                            self.scan_count += 1;
                            if let Some(existing) = self.scan_results.iter_mut().find(|r| r.category == cat) {
                                *existing = result;
                            } else {
                                self.scan_results.push(result);
                            }
                            self.update_scan_status();
                        }
                        Ok(ScanMsg::AllDone(_)) => {
                            self.scanning = false;
                            self.scan_rx = None;
                            if let Some(ui) = self.ui.as_mut() {
                                let elapsed = self.scan_start
                                    .map(|s| s.elapsed().as_secs_f64())
                                    .unwrap_or(0.0);
                                ui.status_bar = format!(
                                    "Scan complete in {:.1}s. {} categories loaded.",
                                    elapsed,
                                    self.scan_results.len()
                                );
                            }
                            self.scan_start = None;
                            terminal.clear()?;
                        }
                        Ok(ScanMsg::Error(e)) => {
                            self.scanning = false;
                            self.scan_rx = None;
                            if let Some(ui) = self.ui.as_mut() {
                                ui.status_bar = format!("Scan error: {}", e);
                            }
                            self.scan_start = None;
                            terminal.clear()?;
                        }
                        Err(mpsc::TryRecvError::Empty) => {}
                        Err(_) => {
                            self.scanning = false;
                            self.scan_rx = None;
                            self.scan_start = None;
                        }
                    }
                }
            }

            if let Some(ref rx) = self.auto_clean_rx {
                match rx.try_recv() {
                    Ok(AutoCleanProgress::PhaseChanged(phase, idx, total)) => {
                        if let Some(ref mut state) = self.auto_clean_state {
                            state.phase = phase;
                            state.phase_index = idx;
                            state.total_phases = total;
                            state.phase_label = phase.label().to_string();
                        }
                        if let Some(ui) = self.ui.as_mut() {
                            if let Some(ref mut ac) = ui.auto_clean {
                                ac.phase = phase;
                                ac.phase_index = idx;
                                ac.total_phases = total;
                                ac.phase_label = phase.label().to_string();
                            }
                        }
                    }
                    Ok(AutoCleanProgress::PhaseProgress(pct, label)) => {
                        if let Some(ref mut state) = self.auto_clean_state {
                            state.phase_progress = pct;
                            state.sub_label = label.clone();
                        }
                        if let Some(ui) = self.ui.as_mut() {
                            if let Some(ref mut ac) = ui.auto_clean {
                                ac.phase_progress = pct;
                                ac.sub_label = label;
                            }
                        }
                    }
                    Ok(AutoCleanProgress::ItemScanned(_path, size)) => {
                        if let Some(ref mut state) = self.auto_clean_state {
                            state.items_scanned += 1;
                            state.bytes_scanned += size;
                        }
                        if let Some(ui) = self.ui.as_mut() {
                            if let Some(ref mut ac) = ui.auto_clean {
                                ac.items_scanned += 1;
                                ac.bytes_scanned += size;
                            }
                        }
                    }
                    Ok(AutoCleanProgress::CategoryFound(_id, findings)) => {
                        if let Some(ref mut state) = self.auto_clean_state {
                            state.categorized.push(findings.clone());
                            state.all_items.extend(findings.items.iter().cloned());
                        }
                        if let Some(ui) = self.ui.as_mut() {
                            if let Some(ref mut ac) = ui.auto_clean {
                                ac.categorized.push(findings);
                            }
                        }
                    }
                    Ok(AutoCleanProgress::ItemCleaned(_path, size, success)) => {
                        if let Some(ref mut state) = self.auto_clean_state {
                            if success {
                                state.items_cleaned += 1;
                                state.bytes_cleaned += size;
                            } else {
                                state.items_failed += 1;
                            }
                        }
                        if let Some(ui) = self.ui.as_mut() {
                            if let Some(ref mut ac) = ui.auto_clean {
                                if success {
                                    ac.items_cleaned += 1;
                                    ac.bytes_cleaned += size;
                                } else {
                                    ac.items_failed += 1;
                                }
                            }
                        }
                    }
                    Ok(AutoCleanProgress::Finished(report)) => {
                        self.auto_clean_report = Some(report);
                        self.auto_clean_rx = None;
                        if let Some(ref mut state) = self.auto_clean_state {
                            state.done = true;
                        }
                        if let Some(ui) = self.ui.as_mut() {
                            if let Some(ref mut ac) = ui.auto_clean {
                                ac.done = true;
                            }
                            // Show review screen after initial scan, not final report
                            if ui.auto_clean_step == AutoCleanStep::ScanProgress {
                                ui.auto_clean_step = AutoCleanStep::CategoryReview;
                                self.auto_clean_review_selected = self.auto_clean_state
                                    .as_ref()
                                    .map(|s| s.categorized.iter().map(|c| c.id.clone()).collect())
                                    .unwrap_or_default();
                                ui.status_bar = "Scan complete! Select categories to clean, then press Enter.".to_string();
                            } else {
                                ui.auto_clean_step = AutoCleanStep::FinalReport;
                                ui.status_bar = "Auto Clean completed!".to_string();
                            }
                        }
                    }
                    Ok(AutoCleanProgress::Cancelled) => {
                        self.auto_clean_rx = None;
                        self.auto_clean_state = None;
                        self.auto_clean_report = None;
                        if let Some(ui) = self.ui.as_mut() {
                            ui.auto_clean = None;
                            ui.status_bar = "Auto Clean cancelled.".to_string();
                        }
                    }
                    Ok(AutoCleanProgress::Error(err)) => {
                        if let Some(ref mut state) = self.auto_clean_state {
                            state.errors.push(err);
                        }
                    }
                    Ok(AutoCleanProgress::Warning(warn)) => {
                        if let Some(ref mut state) = self.auto_clean_state {
                            state.warnings.push(warn);
                        }
                    }
                    Ok(AutoCleanProgress::BackupCreated(id)) => {
                        if let Some(ref mut state) = self.auto_clean_state {
                            state.backup_id = Some(id);
                        }
                    }
                    Err(mpsc::TryRecvError::Empty) => {}
                    Err(_) => {
                        self.auto_clean_rx = None;
                    }
                }
            }

            if event::poll(poll_rate)? {
                match event::read()? {
                    Event::Key(key) => {
                        if key.kind != KeyEventKind::Press {
                            continue;
                        }
                        self.handle_key(key);
                    }
                    Event::Mouse(mouse) => {
                        self.handle_mouse(mouse);
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn update_scan_status(&mut self) {
        if let Some(ui) = self.ui.as_mut() {
            let elapsed = self.scan_start
                .map(|s| s.elapsed().as_secs_f64())
                .unwrap_or(0.0);
            let remaining = self.scan_total.saturating_sub(self.scan_count);
            ui.status_bar = format!(
                "Scanning... {}/{} ({} remaining) in {:.1}s",
                self.scan_count, self.scan_total, remaining, elapsed
            );
        }
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        if self.scanning {
            return;
        }

        if self.ui.as_ref().is_some() && self.ui.as_ref().unwrap().auto_clean.is_some() {
            self.handle_auto_clean_key(key);
            return;
        }

        let mut wizard_triggered = false;
        let mut wizard_analysis: Option<DiskAnalysis> = None;

        {
            let ui = self.ui.as_mut().unwrap();

            match &ui.dialog {
                Dialog::ConfirmClean { .. } => {
                    match key.code {
                        KeyCode::Char('y' | 'Y') => {
                            let dialog = ui.dialog.clone();
                            ui.dialog = Dialog::None;
                            if let Dialog::ConfirmClean { total_items, .. } = dialog {
                                drop(dialog);
                                self.execute_clean(total_items);
                            }
                        }
                        KeyCode::Char('n' | 'N') | KeyCode::Esc => {
                            ui.dialog = Dialog::None;
                        }
                        _ => {}
                    }
                    return;
                }
                Dialog::Report { .. } => {
                    ui.dialog = Dialog::None;
                    return;
                }
                Dialog::Progress { .. } => {
                    return;
                }
                Dialog::Quit => {
                    match key.code {
                        KeyCode::Char('y' | 'Y') => {
                            self.running = false;
                        }
                        _ => {
                            ui.dialog = Dialog::None;
                        }
                    }
                    return;
                }
                Dialog::UndoList { .. } => {
                    match key.code {
                        KeyCode::Up => {
                            ui.undo_selected = ui.undo_selected.saturating_sub(1);
                        }
                        KeyCode::Down => {
                            ui.undo_selected = ui.undo_selected.saturating_add(1)
                                .min(ui.backups.len().saturating_sub(1));
                        }
                        KeyCode::Enter => {
                            if !ui.backups.is_empty() {
                                let idx = ui.undo_selected.min(ui.backups.len() - 1);
                                let id = ui.backups[idx].id.clone();
                                match self.backup.restore_backup(&id) {
                                    Ok(()) => {
                                        ui.dialog = Dialog::Report {
                                            message: format!("Backup {} restored.", &id[..8]),
                                            success: true,
                                        };
                                    }
                                    Err(e) => {
                                        ui.dialog = Dialog::Report {
                                            message: format!("Restore failed: {}", e),
                                            success: false,
                                        };
                                    }
                                }
                            }
                        }
                        KeyCode::Char('d' | 'D') => {
                            ui.dialog = Dialog::None;
                        }
                        KeyCode::Esc => {
                            ui.dialog = Dialog::None;
                        }
                        _ => {}
                    }
                    return;
                }
                Dialog::Search { .. } => {
                    match key.code {
                        KeyCode::Char(c) => {
                            if let Dialog::Search { ref mut query } = ui.dialog {
                                query.push(c);
                            }
                        }
                        KeyCode::Backspace => {
                            if let Dialog::Search { ref mut query } = ui.dialog {
                                query.pop();
                            }
                        }
                        KeyCode::Enter => {
                            if let Dialog::Search { query } = ui.dialog.clone() {
                                if !query.is_empty() {
                                    ui.status_bar = format!("Filtered by: {}", query);
                                }
                                ui.dialog = Dialog::None;
                            }
                        }
                        KeyCode::Esc => {
                            ui.dialog = Dialog::None;
                        }
                        _ => {}
                    }
                    return;
                }
                Dialog::None => {}
                &Dialog::DiskWizard { .. } => {}
            }
        }

        {
            let ui = self.ui.as_mut().unwrap();

            match key.code {
                KeyCode::Char('q' | 'Q') => {
                    ui.dialog = Dialog::Quit;
                }
                KeyCode::Char('?' | 'h' | 'H') => {
                    ui.help_visible = !ui.help_visible;
                }
                KeyCode::Esc => {
                    ui.help_visible = false;
                }
                KeyCode::Up => {
                    if ui.focus == crate::tui::ui::PanelFocus::Sidebar {
                        ui.sidebar.previous();
                        ui.content.list_state.select(Some(0));
                        ui.content.scroll_offset = 0;
                    } else {
                        let len = self.scan_results
                            .iter()
                            .find(|r| r.category == ui.sidebar.selected().unwrap_or(Category::Packages))
                            .map(|r| r.items.len())
                            .unwrap_or(0);
                        ui.content.previous(len);
                    }
                }
                KeyCode::Down => {
                    if ui.focus == crate::tui::ui::PanelFocus::Sidebar {
                        ui.sidebar.next();
                        ui.content.list_state.select(Some(0));
                        ui.content.scroll_offset = 0;
                    } else {
                        let len = self.scan_results
                            .iter()
                            .find(|r| r.category == ui.sidebar.selected().unwrap_or(Category::Packages))
                            .map(|r| r.items.len())
                            .unwrap_or(0);
                        ui.content.next(len);
                    }
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    if ui.focus == crate::tui::ui::PanelFocus::Sidebar {
                        ui.focus = crate::tui::ui::PanelFocus::Content;
                    } else {
                        if let Some(category) = ui.sidebar.selected() {
                            if let Some(result) = self.scan_results.iter_mut().find(|r| r.category == category) {
                                ui.content.toggle_selected(&mut result.items);
                            }
                        }
                    }
                }
                KeyCode::Tab => {
                    ui.focus = if ui.focus == crate::tui::ui::PanelFocus::Sidebar {
                        crate::tui::ui::PanelFocus::Content
                    } else {
                        crate::tui::ui::PanelFocus::Sidebar
                    };
                }
                KeyCode::Left => {
                    ui.focus = crate::tui::ui::PanelFocus::Sidebar;
                }
                KeyCode::Right => {
                    ui.focus = crate::tui::ui::PanelFocus::Content;
                }
                KeyCode::Char('t' | 'T') => {
                    if let Some(category) = ui.sidebar.selected() {
                        if let Some(result) = self.scan_results.iter_mut().find(|r| r.category == category) {
                            ui.content.toggle_all(&mut result.items);
                        }
                    }
                }
                KeyCode::Char('r' | 'R') => {
                    if let Some(category) = ui.sidebar.selected() {
                        if let Some(result) = self.scan_results.iter_mut().find(|r| r.category == category) {
                            ui.content.select_safe_only(&mut result.items);
                        }
                    }
                }
                KeyCode::Char('a' | 'A') => {
                    if let Some(category) = ui.sidebar.selected() {
                        if category == Category::Wizard {
                            let _ = ui;
                            self.start_wizard_internal();
                        } else {
                            ui.status_bar = format!("Scanning {}...", category.name());
                            self.start_scan(ScanMode::Category(category));
                        }
                    }
                }
                KeyCode::Char('s' | 'S') => {
                    if ui.sidebar.selected() == Some(Category::Wizard) {
                        ui.content.next_view();
                        ui.status_bar = format!("Wizard: {:?}", ui.content.wizard_view);
                    } else {
                        ui.status_bar = "Scanning all categories...".to_string();
                        self.start_scan(ScanMode::All);
                    }
                }
                KeyCode::Char('c' | 'C') => {
                    let selected: Vec<&CleanItem> = self.scan_results
                        .iter()
                        .flat_map(|r| r.items.iter())
                        .filter(|i| i.selected)
                        .collect();

                    if selected.is_empty() {
                        ui.status_bar = "No items selected. Use Space to toggle.".to_string();
                        return;
                    }

                    let total_size: u64 = selected.iter().map(|i| i.size).sum();
                    ui.dialog = Dialog::ConfirmClean {
                        total_items: selected.len(),
                        total_size: format_bytes(total_size),
                    };
                }
                KeyCode::Char('u' | 'U') => {
                    match self.backup.list_backups() {
                        Ok(backups) => {
                            if backups.is_empty() {
                                ui.dialog = Dialog::Report {
                                    message: "No backups found.".to_string(),
                                    success: true,
                                };
                            } else {
                                ui.backups = backups;
                                ui.undo_selected = 0;
                                ui.dialog = Dialog::UndoList {
                                    selected: 0,
                                    backup_count: ui.backups.len(),
                                };
                            }
                        }
                        Err(e) => {
                            ui.dialog = Dialog::Report {
                                message: format!("Failed to list backups: {}", e),
                                success: false,
                            };
                        }
                    }
                }
                KeyCode::Char('/') => {
                    ui.dialog = Dialog::Search { query: String::new() };
                }
                KeyCode::Char('w' | 'W') => {
                    wizard_analysis = Some(self.collect_disk_analysis());
                    wizard_triggered = true;
                }
                KeyCode::Char('x' | 'X') => {
                    if self.auto_clean_state.is_none() {
                        self.start_auto_clean();
                    }
                }
                _ => {}
            }
        }

        if wizard_triggered {
            let default_analysis = DiskAnalysis::default();
            if let Some(ui) = self.ui.as_mut() {
                let analysis_opt = wizard_analysis.take();
                let analysis = analysis_opt.unwrap_or(default_analysis);
                let mut wizard = DiskWizardState::new();
                wizard.analysis = analysis;

                ui.wizard = Some(wizard);
                ui.status_bar = "Disk Cleaning Wizard started. Use arrow keys to navigate.".to_string();
            }
        }
    }

    fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        if self.scanning { return; }
        
        let area = if let Some(ui) = self.ui.as_ref() { ui.render_area } else { return };
        
        use crossterm::event::{MouseEventKind, MouseButton};
        let (x, y) = (mouse.column, mouse.row);

        let mut trigger_wizard_analysis = false;

        {
            let ui = self.ui.as_mut().unwrap();
            let target = ui.resolve_click(x, y, area);

            match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    match target {
                        crate::tui::ui::ClickTarget::SidebarItem(idx) => {
                            ui.focus = crate::tui::ui::PanelFocus::Sidebar;
                            ui.sidebar.state.select(Some(idx));
                            ui.content.list_state.select(Some(0));
                            ui.content.scroll_offset = 0;
                            if ui.sidebar.selected() == Some(Category::Wizard) {
                                trigger_wizard_analysis = true;
                            }
                        }
                        crate::tui::ui::ClickTarget::ContentRow(idx) => {
                            ui.focus = crate::tui::ui::PanelFocus::Content;
                            ui.content.list_state.select(Some(idx));
                        }
                        crate::tui::ui::ClickTarget::ContentCheckbox(idx) => {
                            ui.focus = crate::tui::ui::PanelFocus::Content;
                            ui.content.list_state.select(Some(idx));
                            if let Some(category) = ui.sidebar.selected() {
                                if let Some(result) = self.scan_results.iter_mut().find(|r| r.category == category) {
                                    if idx < result.items.len() {
                                        result.items[idx].selected = !result.items[idx].selected;
                                    }
                                }
                            }
                        }
                        crate::tui::ui::ClickTarget::ContentScrollbar { y: thumb_y } => {
                            ui.focus = crate::tui::ui::PanelFocus::Content;
                            // Simple scroll logic on click
                            let list_area = ui.content.render_area.unwrap();
                            let content_y = list_area.y + 4;
                            let content_height = list_area.height - 5;
                            if thumb_y >= content_y && thumb_y < content_y + content_height {
                                let ratio = (thumb_y - content_y) as f32 / content_height as f32;
                                let len = self.scan_results.iter()
                                    .find(|r| r.category == ui.sidebar.selected().unwrap_or(Category::Packages))
                                    .map(|r| r.items.len()).unwrap_or(0);
                                if len > 0 {
                                    let new_idx = (ratio * len as f32) as usize;
                                    ui.content.list_state.select(Some(new_idx.min(len - 1)));
                                }
                            }
                        }
                        crate::tui::ui::ClickTarget::Dialog => {
                            if ui.wizard.is_some() {
                                if let Some(ref wizard) = ui.wizard.clone() {
                                    if x >= ui.render_area.x {
                                        use crate::tui::components::dialogs::WizardStep;
                                        let inner = crate::tui::components::dialogs::centered_rect(70, 24, area);
                                        let content = ratatui::layout::Rect::new(inner.x + 1, inner.y + 1, inner.width - 2, inner.height - 2);
                                        let cat_start = content.y + 4;
                                        match wizard.step {
                                            WizardStep::Overview => {
                                                let cat_count = wizard.analysis.by_category.len();
                                                if y >= cat_start && y < cat_start + cat_count as u16 {
                                                    let idx = (y - cat_start) as usize;
                                                    if idx < cat_count {
                                                        let cat_id = wizard.analysis.by_category[idx].id.clone();
                                                        if let Some(ref mut w) = ui.wizard {
                                                            if let Some(pos) = w.selected_categories.iter().position(|c| *c == cat_id) {
                                                                w.selected_categories.remove(pos);
                                                            } else {
                                                                w.selected_categories.push(cat_id);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            WizardStep::Categories => {
                                                let cat_count = 8usize;
                                                if y >= cat_start && y < cat_start + cat_count as u16 {
                                                    let idx = (y - cat_start) as usize;
                                                    if idx < cat_count {
                                                        let categories = [
                                                            ("browser", "Browser Caches"),
                                                            ("system", "System Caches"),
                                                            ("downloads", "Old Downloads"),
                                                            ("large", "Large Files"),
                                                            ("containers", "Containers"),
                                                            ("dev", "Development"),
                                                            ("thumbnails", "Thumbnails"),
                                                            ("trash", "Trash"),
                                                        ];
                                                        let cat_id = categories[idx].0.to_string();
                                                        if let Some(ref mut w) = ui.wizard {
                                                            if let Some(pos) = w.selected_categories.iter().position(|c| *c == cat_id) {
                                                                w.selected_categories.remove(pos);
                                                            } else {
                                                                w.selected_categories.push(cat_id);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            } else if ui.auto_clean.is_some() && ui.auto_clean_step == crate::tui::components::dialogs::AutoCleanStep::CategoryReview {
                                if let Some(ref ac) = ui.auto_clean.clone() {
                                    let height = (ac.categorized.len() as u16 * 2 + 8).min(28);
                                    let inner = crate::tui::components::dialogs::centered_rect(72, height, area);
                                    let content = ratatui::layout::Rect::new(inner.x + 1, inner.y + 1, inner.width - 2, inner.height - 2);
                                    let mut cat_start = content.y + 2;
                                    for (idx, cat) in ac.categorized.iter().enumerate() {
                                        if y == cat_start {
                                            if x >= content.x + 1 && x < content.x + 4 {
                                                let cat_id = cat.id.clone();
                                                if let Some(pos) = self.auto_clean_review_selected.iter().position(|c| *c == cat_id) {
                                                    self.auto_clean_review_selected.remove(pos);
                                                } else {
                                                    self.auto_clean_review_selected.push(cat_id);
                                                }
                                                if let Some(ref mut state) = self.auto_clean_state {
                                                    state.selected_category_ids = self.auto_clean_review_selected.clone();
                                                }
                                                if let Some(ref mut ac_ui) = ui.auto_clean {
                                                    ac_ui.selected_category_ids = self.auto_clean_review_selected.clone();
                                                }
                                                break;
                                            }
                                        }
                                        cat_start += 1;
                                        cat_start += 1 + cat.sub_groups.len() as u16;
                                        if cat_start > content.y + content.height { break; }
                                    }
                                }
                            } else {
                                if let Some(action) = ui.dialog.handle_click(x, y, area) {
                                    match action {
                                        crate::tui::components::dialogs::DialogAction::Dismiss => {
                                            ui.dialog = Dialog::None;
                                        }
                                        crate::tui::components::dialogs::DialogAction::Confirm => {
                                            let dialog = ui.dialog.clone();
                                            ui.dialog = Dialog::None;
                                            if let Dialog::ConfirmClean { total_items, .. } = dialog {
                                                self.execute_clean(total_items);
                                            } else if let Dialog::Quit = dialog {
                                                self.running = false;
                                            }
                                        }
                                        crate::tui::components::dialogs::DialogAction::Cancel => {
                                            ui.dialog = Dialog::None;
                                        }
                                        crate::tui::components::dialogs::DialogAction::SelectBackup(idx) => {
                                            ui.undo_selected = idx;
                                        }
                                        crate::tui::components::dialogs::DialogAction::RestoreBackup => {
                                            if !ui.backups.is_empty() {
                                                let idx = ui.undo_selected.min(ui.backups.len() - 1);
                                                let id = ui.backups[idx].id.clone();
                                                match self.backup.restore_backup(&id) {
                                                    Ok(()) => ui.dialog = Dialog::Report { message: format!("Backup {} restored.", &id[..8]), success: true },
                                                    Err(e) => ui.dialog = Dialog::Report { message: format!("Restore failed: {}", e), success: false },
                                                }
                                            }
                                        }
                                        crate::tui::components::dialogs::DialogAction::DeleteBackup => {
                                            ui.dialog = Dialog::None;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                MouseEventKind::Down(MouseButton::Right) => {
                    if let crate::tui::ui::ClickTarget::ContentRow(idx) | crate::tui::ui::ClickTarget::ContentCheckbox(idx) = target {
                        ui.focus = crate::tui::ui::PanelFocus::Content;
                        ui.content.list_state.select(Some(idx));
                        if let Some(category) = ui.sidebar.selected() {
                            if let Some(result) = self.scan_results.iter_mut().find(|r| r.category == category) {
                                if idx < result.items.len() {
                                    result.items[idx].selected = !result.items[idx].selected;
                                }
                            }
                        }
                    }
                }
                MouseEventKind::Moved => {
                    ui.hovered_sidebar = None;
                    ui.hovered_content_row = None;
                    match target {
                        crate::tui::ui::ClickTarget::SidebarItem(idx) => {
                            ui.hovered_sidebar = Some(idx);
                        }
                        crate::tui::ui::ClickTarget::ContentRow(idx) | crate::tui::ui::ClickTarget::ContentCheckbox(idx) => {
                            ui.hovered_content_row = Some(idx);
                        }
                        _ => {}
                    }
                }
                MouseEventKind::Drag(MouseButton::Left) => {
                    ui.hovered_sidebar = None;
                    ui.hovered_content_row = None;
                    match target {
                        crate::tui::ui::ClickTarget::SidebarItem(idx) => {
                            ui.hovered_sidebar = Some(idx);
                        }
                        crate::tui::ui::ClickTarget::ContentRow(idx) | crate::tui::ui::ClickTarget::ContentCheckbox(idx) => {
                            ui.hovered_content_row = Some(idx);
                            
                            // Multi-select painting feature! Checking checkboxes on drag
                            if let Some(category) = ui.sidebar.selected() {
                                if let Some(result) = self.scan_results.iter_mut().find(|r| r.category == category) {
                                    if idx < result.items.len() {
                                        result.items[idx].selected = true;
                                    }
                                }
                            }
                        }
                        crate::tui::ui::ClickTarget::ContentScrollbar { y: thumb_y } => {
                            let list_area = ui.content.render_area.unwrap();
                            let content_y = list_area.y + 4;
                            let content_height = list_area.height - 5;
                            if thumb_y >= content_y && thumb_y < content_y + content_height {
                                let ratio = (thumb_y - content_y) as f32 / content_height as f32;
                                let len = self.scan_results.iter()
                                    .find(|r| r.category == ui.sidebar.selected().unwrap_or(Category::Packages))
                                    .map(|r| r.items.len()).unwrap_or(0);
                                if len > 0 {
                                    let new_idx = (ratio * len as f32) as usize;
                                    ui.content.list_state.select(Some(new_idx.min(len - 1)));
                                }
                            }
                        }
                        _ => {}
                    }
                }
                MouseEventKind::ScrollUp => {
                    match ui.focus {
                        crate::tui::ui::PanelFocus::Sidebar => ui.sidebar.previous(),
                        crate::tui::ui::PanelFocus::Content => {
                            let len = self.scan_results.iter()
                                .find(|r| r.category == ui.sidebar.selected().unwrap_or(Category::Packages))
                                .map(|r| r.items.len()).unwrap_or(0);
                            ui.content.previous(len);
                        }
                    }
                }
                MouseEventKind::ScrollDown => {
                    match ui.focus {
                        crate::tui::ui::PanelFocus::Sidebar => ui.sidebar.next(),
                        crate::tui::ui::PanelFocus::Content => {
                            let len = self.scan_results.iter()
                                .find(|r| r.category == ui.sidebar.selected().unwrap_or(Category::Packages))
                                .map(|r| r.items.len()).unwrap_or(0);
                            ui.content.next(len);
                        }
                    }
                }
                _ => {}
            }
        }

        if trigger_wizard_analysis {
            let default_analysis = DiskAnalysis::default();
            let analysis = self.collect_disk_analysis();
            if let Some(ui) = self.ui.as_mut() {
                let mut wizard = DiskWizardState::new();
                wizard.analysis = analysis;
                ui.wizard = Some(wizard);
                ui.status_bar = "Disk Cleaning Wizard started. Use arrow keys to navigate.".to_string();
            }
        }
    }

    fn start_wizard(&mut self, ui: &mut AppUi) {
        let analysis = self.collect_disk_analysis();
        let mut wizard = DiskWizardState::new();
        wizard.analysis = analysis;

        ui.wizard = Some(wizard);
        ui.status_bar = "Disk Cleaning Wizard started. Use arrow keys to navigate.".to_string();
    }

    fn start_scan(&mut self, mode: ScanMode) {
        self.scanning = true;
        self.scan_start = Some(Instant::now());
        self.scan_count = 0;
        self.scan_results.clear();

        let os = self.os.clone();
        let registry = ModuleRegistry::new();

        let (tx, rx) = mpsc::channel();

        match mode {
            ScanMode::Category(cat) => {
                self.scan_total = 1;
                thread::spawn(move || {
                    match registry.scan_category(cat, &os) {
                        Ok(result) => {
                            let _ = tx.send(ScanMsg::CategoryDone(cat, result.clone()));
                            let _ = tx.send(ScanMsg::AllDone(vec![result]));
                        }
                        Err(e) => {
                            let _ = tx.send(ScanMsg::Error(e.to_string()));
                        }
                    }
                });
            }
            ScanMode::All => {
                let categories: Vec<Category> = Category::all().to_vec();
                self.scan_total = categories.len();

                thread::spawn(move || {
                    let mut results = Vec::new();
                    for cat in categories {
                        match registry.scan_category(cat, &os) {
                            Ok(result) => {
                                let _ = tx.send(ScanMsg::CategoryDone(cat, result.clone()));
                                results.push(result);
                            }
                            Err(_) => {
                                let result = ScanResult::new(cat, vec![]);
                                let _ = tx.send(ScanMsg::CategoryDone(cat, result.clone()));
                                results.push(result);
                            }
                        }
                    }
                    let _ = tx.send(ScanMsg::AllDone(results));
                });
            }
        }

        self.scan_rx = Some(rx);
    }

    fn execute_clean(&mut self, total_items: usize) {
        let ui = self.ui.as_mut().unwrap();
        ui.dialog = Dialog::Progress {
            current: 0,
            total: total_items,
            current_file: "Starting...".to_string(),
        };

        let selected: Vec<CleanItem> = self.scan_results
            .iter()
            .flat_map(|r| r.items.iter())
            .filter(|i| i.selected)
            .cloned()
            .collect();

        let (tx, rx) = mpsc::channel();
        let backup = Backup::new(
            self.config.backup.location.clone(),
            self.config.backup.enabled,
        );
        let cleaner = Cleaner::new(backup);

        thread::spawn(move || {
            let _ = cleaner.execute(&selected, tx);
        });

        let mut success = true;
        let mut final_message = String::new();

        loop {
            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(CleanProgress::File(path)) => {
                    if let Some(ui) = self.ui.as_mut() {
                        if let Dialog::Progress { ref mut current_file, .. } = ui.dialog {
                            *current_file = path;
                        }
                    }
                }
                Ok(CleanProgress::Done(count)) => {
                    if let Some(ui) = self.ui.as_mut() {
                        if let Dialog::Progress { ref mut current, .. } = ui.dialog {
                            *current = count;
                        }
                    }
                }
                Ok(CleanProgress::Error(e)) => {
                    success = false;
                    final_message = format!("Error: {}", e);
                }
                Ok(CleanProgress::Finished) => {
                    if success {
                        final_message = format!(
                            "Successfully cleaned {} items. Use 'U' to undo.",
                            total_items
                        );
                    }
                    break;
                }
                Err(_) => break,
            }
        }

        if let Some(ui) = self.ui.as_mut() {
            self.logger.action("CLEAN", &format!("{} items", total_items), 0);
            let _ = self.backup.cleanup_old(
                self.config.backup.retention_days,
                self.config.backup.max_backups,
            );

            ui.dialog = Dialog::Report {
                message: final_message,
                success,
            };
        }
    }
}

enum ScanMode {
    Category(Category),
    All,
}

impl App {
    fn start_auto_clean(&mut self) {
        let os = self.os.clone();
        let registry = ModuleRegistry::new();
        let backup = Backup::new(
            self.config.backup.location.clone(),
            self.config.backup.enabled,
        );

        let (tx, rx) = mpsc::channel();

        self.auto_clean_state = Some(AutoCleanState::default());
        self.auto_clean_rx = Some(rx);
        self.auto_clean_report = None;
        self.auto_clean_review_cursor = 0;
        self.auto_clean_review_selected = Vec::new();

        if let Some(ui) = self.ui.as_mut() {
            ui.auto_clean = Some(AutoCleanState::default());
            ui.auto_clean_step = AutoCleanStep::ScanProgress;
            ui.status_bar = "Auto Clean started... [Esc] to cancel.".to_string();
        }

        thread::spawn(move || {
            AutoCleanEngine::run(&os, &registry, &backup, tx, true, None);
        });
    }

    fn handle_auto_clean_key(&mut self, key: crossterm::event::KeyEvent) {
        let step = self.ui
            .as_ref()
            .map(|u| u.auto_clean_step.clone())
            .unwrap_or(AutoCleanStep::Welcome);

        match step {
            AutoCleanStep::ScanProgress => {
                if let KeyCode::Esc = key.code {
                    self.auto_clean_rx = None;
                    self.auto_clean_state = None;
                    self.auto_clean_report = None;
                    if let Some(ui) = self.ui.as_mut() {
                        ui.auto_clean = None;
                        ui.status_bar = "Auto Clean cancelled.".to_string();
                    }
                }
            }
            AutoCleanStep::CategoryReview => {
                let count = self.auto_clean_state
                    .as_ref()
                    .map(|s| s.categorized.len())
                    .unwrap_or(0);

                match key.code {
                    KeyCode::Up => {
                        self.auto_clean_review_cursor = self.auto_clean_review_cursor.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        let max = count.saturating_sub(1);
                        self.auto_clean_review_cursor = self.auto_clean_review_cursor.min(max).saturating_add(1).min(max);
                    }
                    KeyCode::Char(' ') => {
                        if let Some(state) = self.auto_clean_state.as_ref() {
                            if self.auto_clean_review_cursor < state.categorized.len() {
                                let id = state.categorized[self.auto_clean_review_cursor].id.clone();
                                if let Some(pos) = self.auto_clean_review_selected.iter().position(|x| *x == id) {
                                    self.auto_clean_review_selected.remove(pos);
                                } else {
                                    self.auto_clean_review_selected.push(id);
                                }
                            }
                        }
                    }
                    KeyCode::Enter => {
                        // Update the state with selected category ids
                        if let Some(ref mut state) = self.auto_clean_state {
                            state.selected_category_ids = self.auto_clean_review_selected.clone();
                        }
                        if let Some(ui) = self.ui.as_mut() {
                            if let Some(ref mut ac) = ui.auto_clean {
                                ac.selected_category_ids = self.auto_clean_review_selected.clone();
                            }
                            ui.auto_clean_step = AutoCleanStep::Confirm;
                        }
                        return;
                    }
                    KeyCode::Esc => {
                        self.auto_clean_rx = None;
                        self.auto_clean_state = None;
                        self.auto_clean_report = None;
                        if let Some(ui) = self.ui.as_mut() {
                            ui.auto_clean = None;
                            ui.status_bar = "Auto Clean cancelled.".to_string();
                        }
                        return;
                    }
                    _ => {}
                }
            }
            AutoCleanStep::Confirm => {
                match key.code {
                    KeyCode::Char('y' | 'Y') => {
                        if let Some(ref mut state) = self.auto_clean_state {
                            state.cleaning = true;
                        }
                        if let Some(ui) = self.ui.as_mut() {
                            if let Some(ref mut ac) = ui.auto_clean {
                                ac.cleaning = true;
                            }
                            ui.auto_clean_step = AutoCleanStep::CleanProgress;
                            ui.status_bar = "Cleaning in progress...".to_string();
                        }
                        self.execute_auto_clean();
                    }
                    KeyCode::Char('s' | 'S') => {
                        // Safe-only mode
                        if let Some(ref state) = self.auto_clean_state {
                            self.auto_clean_review_selected = state.categorized.iter()
                                .filter(|c| c.risk.level <= RiskLevel::Caution)
                                .map(|c| c.id.clone())
                                .collect();
                        }
                        if let Some(ui) = self.ui.as_mut() {
                            if let Some(ref mut ac) = ui.auto_clean {
                                ac.selected_category_ids = self.auto_clean_review_selected.clone();
                            }
                            ui.auto_clean_step = AutoCleanStep::Confirm;
                        }
                    }
                    KeyCode::Char('n' | 'N') | KeyCode::Esc => {
                        if let Some(ui) = self.ui.as_mut() {
                            ui.auto_clean_step = AutoCleanStep::CategoryReview;
                        }
                    }
                    _ => {}
                }
            }
            AutoCleanStep::CleanProgress => {
                if let KeyCode::Esc = key.code {
                    self.auto_clean_rx = None;
                    if let Some(ref mut state) = self.auto_clean_state {
                        state.cancelled = true;
                    }
                }
            }
            AutoCleanStep::FinalReport => {
                match key.code {
                    KeyCode::Enter | KeyCode::Esc | KeyCode::Char(' ') => {
                        self.auto_clean_rx = None;
                        self.auto_clean_state = None;
                        self.auto_clean_report = None;
                        if let Some(ui) = self.ui.as_mut() {
                            ui.auto_clean = None;
                            ui.auto_clean_step = AutoCleanStep::Welcome;
                            ui.status_bar = "Ready. Press [X] for Auto Clean.".to_string();
                        }
                    }
                    KeyCode::Char('u' | 'U') => {
                        if let Some(ref state) = self.auto_clean_state.clone() {
                            if let Some(ref backup_id) = state.backup_id {
                                match self.backup.restore_backup(backup_id) {
                                    Ok(()) => {
                                        if let Some(ui) = self.ui.as_mut() {
                                            ui.dialog = Dialog::Report {
                                                message: format!("Backup {} restored successfully.", &backup_id[..8]),
                                                success: true,
                                            };
                                        }
                                    }
                                    Err(e) => {
                                        if let Some(ui) = self.ui.as_mut() {
                                            ui.dialog = Dialog::Report {
                                                message: format!("Restore failed: {}", e),
                                                success: false,
                                            };
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            AutoCleanStep::Welcome | AutoCleanStep::Detail | AutoCleanStep::RiskReview => {
                match key.code {
                    KeyCode::Enter => {
                        if let Some(ui) = self.ui.as_mut() {
                            ui.auto_clean_step = AutoCleanStep::ScanProgress;
                        }
                        self.start_auto_clean();
                    }
                    KeyCode::Esc => {
                        self.auto_clean_state = None;
                        self.auto_clean_report = None;
                        if let Some(ui) = self.ui.as_mut() {
                            ui.auto_clean = None;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn execute_auto_clean(&mut self) {
        let selected: Vec<CleanItem> = self.auto_clean_state
            .as_ref()
            .map(|state| {
                state.categorized.iter()
                    .filter(|c| state.selected_category_ids.contains(&c.id))
                    .flat_map(|c| c.items.iter().cloned())
                    .collect()
            })
            .unwrap_or_default();

        let total_items = selected.len();
        if total_items == 0 {
            if let Some(ui) = self.ui.as_mut() {
                ui.auto_clean_step = AutoCleanStep::CategoryReview;
                ui.status_bar = "No items selected. Please select categories.".to_string();
            }
            return;
        }

        let (tx, rx) = mpsc::channel();
        let backup = Backup::new(
            self.config.backup.location.clone(),
            self.config.backup.enabled,
        );
        let cleaner = Cleaner::new(backup);

        thread::spawn(move || {
            let _ = cleaner.execute(&selected, tx);
        });

        loop {
            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(CleanProgress::File(path)) => {
                    if let Some(ref mut state) = self.auto_clean_state {
                        state.sub_label = format!("Cleaning: {}", path);
                    }
                }
                Ok(CleanProgress::Done(count)) => {
                    if let Some(ref mut state) = self.auto_clean_state {
                        state.items_cleaned = count;
                    }
                }
                Ok(CleanProgress::Error(e)) => {
                    if let Some(ref mut state) = self.auto_clean_state {
                        state.items_failed += 1;
                        state.errors.push(crate::core::auto_clean::CleanError {
                            phase: AutoCleanPhase::BackupAndClean,
                            item_path: String::new(),
                            message: e,
                            recoverable: true,
                        });
                    }
                }
                Ok(CleanProgress::Finished) => {
                    break;
                }
                Err(_) => break,
            }
        }

        self.auto_clean_rx = None;
        if let Some(ui) = self.ui.as_mut() {
            if let Some(ref mut ac) = ui.auto_clean {
                if let Some(ref state) = self.auto_clean_state {
                    ac.items_cleaned = state.items_cleaned;
                    ac.items_failed = state.items_failed;
                    ac.bytes_cleaned = state.bytes_cleaned;
                    ac.done = true;
                }
            }
            ui.auto_clean_step = AutoCleanStep::FinalReport;
            ui.status_bar = format!("Cleaned {} items. [U] to undo, [Enter] to close.", total_items);
        }
    }

    fn start_wizard_internal(&mut self) {
        let analysis = self.collect_disk_analysis();
        let mut wizard = crate::tui::components::dialogs::DiskWizardState::new();
        wizard.analysis = analysis;
        
        if let Some(ui) = self.ui.as_mut() {
            ui.wizard = Some(wizard);
            ui.status_bar = "Disk Cleaning Wizard started.".to_string();
        }
    }

    fn collect_disk_analysis(&self) -> DiskAnalysis {
        let mut analysis = DiskAnalysis::default();
        let home = dirs::home_dir();

        let category_configs: Vec<(&str, &str, Option<u64>)> = vec![
            ("Browser Caches", "browser", home.as_ref().and_then(|h| {
                let p = h.join(".cache");
                if p.exists() { Some(dir_size(&p)) } else { None }
            })),
            ("Thumbnails", "thumbnails", home.as_ref().and_then(|h| {
                let p = h.join(".cache/thumbnails");
                if p.exists() { Some(dir_size(&p)) } else { None }
            })),
            ("Trash", "trash", home.as_ref().and_then(|h| {
                let p = h.join(".local/share/Trash");
                if p.exists() { Some(dir_size(&p)) } else { None }
            })),
            ("Downloads", "downloads", home.as_ref().and_then(|h| {
                let p = h.join("Downloads");
                if p.exists() { Some(dir_size(&p)) } else { None }
            })),
            ("Log Files", "logs", {
                let p = std::path::Path::new("/var/log");
                if p.exists() { Some(dir_size(p)) } else { None }
            }),
            ("Temp Files", "temp", {
                let p = std::path::Path::new("/tmp");
                if p.exists() { Some(dir_size(p)) } else { None }
            }),
        ];

        for (name, id, size_opt) in category_configs {
            if let Some(size) = size_opt {
                if size > 1024 * 1024 {
                    analysis.by_category.push(CategoryInfo {
                        name: name.to_string(),
                        id: id.to_string(),
                        size,
                        item_count: 1,
                        description: format!("{} files", name),
                    });
                    analysis.total_cleanable += size;
                }
            }
        }

        for result in &self.scan_results {
            if result.category == Category::Disk {
                for item in &result.items {
                    if !item.safe {
                        continue;
                    }
                    if item.size > 0 {
                        let cat_id = item.category.clone();
                        if !cat_id.is_empty() {
                            if let Some(cat) = analysis.by_category.iter_mut().find(|c| c.id == cat_id) {
                                cat.size += item.size;
                                cat.item_count += item.file_count.map(|f| f as usize).unwrap_or(1);
                            }
                        }
                    }
                }
            }
        }

        analysis.by_category.sort_by(|a, b| b.size.cmp(&a.size));
        analysis
    }
}