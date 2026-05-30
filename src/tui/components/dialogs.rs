use ratatui::prelude::*;
use ratatui::widgets::*;
use ratatui::layout::{Alignment, Rect};
use crate::core::scanner::CleanItem;
use crate::core::auto_clean::{AutoCleanPhase, AutoCleanState, AutoCleanProgress, AutoCleanReport, CategoryFindings, PhaseStatus, RiskLevel, PhaseHistoryEntry};
use crate::tui::components::charts::render_bar_chart;
use crate::tui::components::widgets::ProgressWidget;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WizardStep {
    Overview,
    Categories,
    Preview,
    Confirm,
    Cleaning,
    Done,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DiskAnalysis {
    pub disk_usage: Vec<(String, u64, f32)>,
    pub total_cleanable: u64,
    pub by_category: Vec<CategoryInfo>,
}

impl Default for DiskAnalysis {
    fn default() -> Self {
        DiskAnalysis {
            disk_usage: Vec::new(),
            total_cleanable: 0,
            by_category: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CategoryInfo {
    pub name: String,
    pub id: String,
    pub size: u64,
    pub item_count: usize,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DialogAction {
    Confirm,
    Cancel,
    Dismiss,
    SelectBackup(usize),
    RestoreBackup,
    DeleteBackup,
    WizardToggleCategory(usize),
    AutoCleanToggleCategory(usize),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Dialog {
    ConfirmClean { total_items: usize, total_size: String },
    Progress { current: usize, total: usize, current_file: String },
    Report { message: String, success: bool },
    UndoList { selected: usize, backup_count: usize },
    Search { query: String },
    DiskWizard {
        step: WizardStep,
        analysis: DiskAnalysis,
        selected_categories: Vec<String>,
        selected_items: Vec<CleanItem>,
    },
    Quit,
    None,
}

impl Dialog {
    pub fn handle_click(&self, x: u16, y: u16, area: Rect) -> Option<DialogAction> {
        match self {
            Dialog::ConfirmClean { .. } | Dialog::Quit => {
                let inner = if matches!(self, Dialog::ConfirmClean { .. }) {
                    centered_rect(50, 7, area)
                } else {
                    centered_rect(35, 6, area)
                };
                let btn_y = inner.y + inner.height - 2;
                if y == btn_y {
                    let mid = inner.x + inner.width / 2;
                    if x >= inner.x + 4 && x <= mid {
                        return Some(DialogAction::Confirm);
                    }
                    if x >= mid && x <= inner.x + inner.width - 4 {
                        return Some(DialogAction::Cancel);
                    }
                }
            }
            Dialog::UndoList { backup_count, .. } => {
                let height = (*backup_count as u16 + 6).min(20);
                let inner = centered_rect(55, height, area);
                let list_start = inner.y + 3;
                let list_end = list_start + *backup_count as u16;
                if y >= list_start && y < list_end {
                    let idx = (y - list_start) as usize;
                    if idx < *backup_count {
                        return Some(DialogAction::SelectBackup(idx));
                    }
                }
                let bottom = inner.y + inner.height - 2;
                if y == bottom {
                    let parts = inner.width / 4;
                    if x >= inner.x + 2 && x < inner.x + parts {
                        return Some(DialogAction::RestoreBackup);
                    }
                    if x >= inner.x + parts && x < inner.x + 2 * parts {
                        return Some(DialogAction::DeleteBackup);
                    }
                }
            }
            Dialog::Search { .. } => {
                let inner = centered_rect(50, 5, area);
                let input_y = inner.y + 2;
                if y == input_y && x >= inner.x + 10 && x < inner.x + inner.width - 2 {
                    return Some(DialogAction::Confirm);
                }
            }
            Dialog::DiskWizard { step, analysis, .. } => {
                let inner = centered_rect(70, 24, area);
                let content = Rect::new(inner.x + 1, inner.y + 1, inner.width - 2, inner.height - 2);
                let cat_start = content.y + 4;
                match step {
                    WizardStep::Overview => {
                        let cat_count = analysis.by_category.len();
                        if y >= cat_start && y < cat_start + cat_count as u16 {
                            let idx = (y - cat_start) as usize;
                            if idx < cat_count && x >= content.x && x < content.x + inner.width - 2 {
                                return Some(DialogAction::WizardToggleCategory(idx));
                            }
                        }
                    }
                    WizardStep::Categories => {
                        let cat_count = 8usize;
                        if y >= cat_start && y < cat_start + cat_count as u16 {
                            let idx = (y - cat_start) as usize;
                            if idx < cat_count && x >= content.x && x < content.x + inner.width - 2 {
                                return Some(DialogAction::WizardToggleCategory(idx));
                            }
                        }
                    }
                    _ => {}
                }
                return None;
            }
            _ => {}
        }
        Some(DialogAction::Dismiss)
    }
}

pub struct DialogColors;

impl DialogColors {
    pub fn dark() -> DialogColorSet {
        DialogColorSet {
            bg: Color::Rgb(20, 20, 35),
            border: Color::Rgb(80, 80, 120),
            accent: Color::Rgb(100, 220, 180),
            warning: Color::Rgb(255, 200, 80),
            danger: Color::Rgb(255, 100, 100),
            success: Color::Rgb(100, 220, 150),
            text: Color::Rgb(210, 210, 230),
            text_dim: Color::Rgb(100, 100, 130),
        }
    }
}

pub struct DialogColorSet {
    pub bg: Color,
    pub border: Color,
    pub accent: Color,
    pub warning: Color,
    pub danger: Color,
    pub success: Color,
    pub text: Color,
    pub text_dim: Color,
}

pub struct Dialogs;

impl Dialogs {
    pub fn render_confirm(area: Rect, buf: &mut Buffer, total_items: usize, total_size: &str) {
        let colors = DialogColors::dark();
        let inner = centered_rect(50, 7, area);

        let block = Block::default()
            .title(" Confirm Clean ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.warning))
            .style(Style::default().bg(colors.bg));
        Widget::render(block, inner, buf);

        let text = Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!(" You are about to clean {} items ({})", total_items, total_size),
                Style::default().fg(colors.text).bold(),
            )),
            Line::from(""),
            Line::from(Span::styled(
                " This action will be logged and can be undone.",
                Style::default().fg(colors.text_dim),
            )),
            Line::from(""),
            Line::from(Span::styled(
                " [Y] Confirm    [N] Cancel",
                Style::default().fg(colors.warning),
            )),
        ]);

        Paragraph::new(text)
            .alignment(Alignment::Center)
            .render(inner, buf);
    }

    pub fn render_progress(area: Rect, buf: &mut Buffer, current: usize, total: usize, current_file: &str) {
        let colors = DialogColors::dark();
        let inner = centered_rect(55, 8, area);

        let block = Block::default()
            .title(" Cleaning ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.accent))
            .style(Style::default().bg(colors.bg));
        Widget::render(block, inner, buf);

        let progress = if total > 0 { current as f64 / total as f64 } else { 0.0 };

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(colors.accent))
            .label(format!("{}/{} ({}%)", current, total, (progress * 100.0) as u64))
            .ratio(progress);

        let content_area = Rect::new(inner.x + 1, inner.y + 1, inner.width - 2, 3);
        gauge.render(content_area, buf);

        let file_display = if current_file.len() > 45 {
            format!("...{}", &current_file[current_file.len().saturating_sub(42)..])
        } else {
            current_file.to_string()
        };

        Paragraph::new(Text::styled(
            format!(" Now: {}", file_display),
            Style::default().fg(colors.text_dim),
        ))
        .render(Rect::new(content_area.x, content_area.y + 3, content_area.width, 1), buf);

        Paragraph::new(Text::styled(
            " Press Esc to cancel",
            Style::default().fg(colors.text_dim).italic(),
        ))
        .render(Rect::new(content_area.x, content_area.y + 4, content_area.width, 1), buf);
    }

    pub fn render_report(area: Rect, buf: &mut Buffer, message: &str, success: bool) {
        let colors = DialogColors::dark();
        let inner = centered_rect(50, 6, area);
        
        let (border_color, icon) = if success {
            (colors.success, "OK")
        } else {
            (colors.danger, "ERROR")
        };

        let block = Block::default()
            .title(format!(" {} Results ", icon))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(colors.bg));
        Widget::render(block, inner, buf);

        let text = Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(message, Style::default().fg(colors.text))),
            Line::from(""),
            Line::from(Span::styled(
                " Press any key to continue",
                Style::default().fg(colors.text_dim).italic(),
            )),
        ]);

        Paragraph::new(text)
            .alignment(Alignment::Center)
            .render(inner, buf);
    }

    pub fn render_wizard(area: Rect, buf: &mut Buffer, wizard: &DiskWizardState) {
        let colors = DialogColors::dark();
        let inner = centered_rect(70, 24, area);

        let block = Block::default()
            .title(" Disk Cleaning Wizard ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.accent))
            .style(Style::default().bg(colors.bg));
        Widget::render(block, inner, buf);

        let content_area = Rect::new(inner.x + 1, inner.y + 1, inner.width - 2, inner.height - 2);

        match &wizard.step {
            WizardStep::Overview => render_wizard_overview(content_area, buf, wizard, &colors),
            WizardStep::Categories => render_wizard_categories(content_area, buf, wizard, &colors),
            WizardStep::Preview => render_wizard_preview(content_area, buf, wizard, &colors),
            WizardStep::Confirm => render_wizard_confirm(content_area, buf, wizard, &colors),
            WizardStep::Cleaning => render_wizard_cleaning(content_area, buf, wizard, &colors),
            WizardStep::Done => render_wizard_done(content_area, buf, wizard, &colors),
        }
    }

    pub fn render_help(area: Rect, buf: &mut Buffer) {
        let colors = DialogColors::dark();
        let inner = centered_rect(55, 26, area);

        let block = Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.accent))
            .style(Style::default().bg(colors.bg));
        Widget::render(block, inner, buf);

        let help_lines = vec![
            Line::from(vec![Span::styled(" KEYBINDINGS", Style::default().fg(colors.accent).bold())]),
            Line::from(""),
            Line::from(vec![Span::styled(" ↑/↓         Navigate items", Style::default().fg(colors.text))]),
            Line::from(vec![Span::styled(" ←/→         Switch categories", Style::default().fg(colors.text))]),
            Line::from(vec![Span::styled(" Tab         Switch panels", Style::default().fg(colors.text))]),
            Line::from(vec![Span::styled(" Space       Toggle selection", Style::default().fg(colors.text))]),
            Line::from(vec![Span::styled(" A           Analyze category", Style::default().fg(colors.text))]),
            Line::from(vec![Span::styled(" S           Scan all", Style::default().fg(colors.text))]),
            Line::from(vec![Span::styled(" C           Clean selected", Style::default().fg(colors.text))]),
            Line::from(vec![Span::styled(" T           Toggle all", Style::default().fg(colors.text))]),
            Line::from(vec![Span::styled(" R           Select safe only", Style::default().fg(colors.text))]),
            Line::from(vec![Span::styled(" U           Undo / backups", Style::default().fg(colors.text))]),
            Line::from(vec![Span::styled(" W           Disk Cleaning Wizard", Style::default().fg(colors.text))]),
            Line::from(vec![Span::styled(" Q / Esc     Quit", Style::default().fg(colors.text))]),
            Line::from(vec![Span::styled(" ? / H       Toggle help", Style::default().fg(colors.text))]),
            Line::from(""),
            Line::from(vec![Span::styled(" CATEGORIES", Style::default().fg(colors.accent).bold())]),
            Line::from(vec![Span::styled(" Packages | System | Apps | Privacy | Services", Style::default().fg(colors.text_dim))]),
            Line::from(vec![Span::styled(" Duplicates | Containers | Disk Usage | Wizard", Style::default().fg(colors.text_dim))]),
            Line::from(""),
            Line::from(vec![Span::styled(" SAFETY", Style::default().fg(colors.warning).bold())]),
            Line::from(""),
            Line::from(vec![Span::styled(" [!] = Potentially destructive (R to select safe only)", Style::default().fg(colors.text_dim))]),
            Line::from(vec![Span::styled(" [*] = Safe to clean", Style::default().fg(colors.text_dim))]),
            Line::from(vec![Span::styled(" All deletions are backed up for rollback", Style::default().fg(colors.text_dim))]),
            Line::from(""),
            Line::from(vec![Span::styled(" Press any key to close", Style::default().fg(colors.text_dim).italic())]),
        ];

        Paragraph::new(Text::from(help_lines))
            .alignment(Alignment::Left)
            .render(inner, buf);
    }

    pub fn render_undo(area: Rect, buf: &mut Buffer, selected: usize, backups: &[crate::core::backup::BackupManifest]) {
        let colors = DialogColors::dark();
        let height = (backups.len() as u16 + 6).min(20);
        let inner = centered_rect(55, height, area);

        let block = Block::default()
            .title(" Undo / Rollback ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.warning))
            .style(Style::default().bg(colors.bg));
        Widget::render(block, inner, buf);

        let mut lines = vec![
            Line::from(""),
            Line::from(vec![Span::styled(" Available backups:", Style::default().fg(colors.warning).bold())]),
            Line::from(""),
        ];

        for (i, b) in backups.iter().enumerate() {
            let marker = if i == selected { ">" } else { " " };
            let line = format!(" {}  {}  ({} items)", marker, &b.timestamp[..19], b.items.len());
            let style = if i == selected {
                Style::default().fg(colors.text).bold()
            } else {
                Style::default().fg(colors.text_dim)
            };
            lines.push(Line::from(vec![Span::styled(line, style)]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            " [Enter] Restore  [D] Delete  [Esc] Close",
            Style::default().fg(colors.text_dim),
        )]));

        Paragraph::new(Text::from(lines))
            .alignment(Alignment::Left)
            .render(inner, buf);
    }

    pub fn render_quit(area: Rect, buf: &mut Buffer) {
        let colors = DialogColors::dark();
        let inner = centered_rect(35, 6, area);

        let block = Block::default()
            .title(" Quit ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.danger))
            .style(Style::default().bg(colors.bg));
        Widget::render(block, inner, buf);

        let text = Text::from(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                " Quit WinDebloat?",
                Style::default().fg(colors.text).bold(),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                " [Y] Yes   [N] No",
                Style::default().fg(colors.text_dim),
            )]),
        ]);

        Paragraph::new(text)
            .alignment(Alignment::Center)
            .render(inner, buf);
    }

    pub fn render_search(area: Rect, buf: &mut Buffer, query: &str) {
        let colors = DialogColors::dark();
        let inner = centered_rect(50, 5, area);

        let block = Block::default()
            .title(" Search ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.accent))
            .style(Style::default().bg(colors.bg));
        Widget::render(block, inner, buf);

        let text = Text::from(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!(" Search: {}", query),
                Style::default().fg(colors.text).bold(),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                " Press Enter to filter, Esc to cancel",
                Style::default().fg(colors.text_dim).italic(),
            )]),
        ]);

        Paragraph::new(text)
            .alignment(Alignment::Center)
            .render(inner, buf);
    }

    pub fn render_auto_clean(area: Rect, buf: &mut Buffer, state: &AutoCleanState, step: &AutoCleanStep) {
        match step {
            AutoCleanStep::Welcome => Self::render_auto_welcome(area, buf),
            AutoCleanStep::ScanProgress => Self::render_auto_scan_progress(area, buf, state),
            AutoCleanStep::CategoryReview => Self::render_auto_review(area, buf, state),
            AutoCleanStep::Detail => {},
            AutoCleanStep::RiskReview => {},
            AutoCleanStep::Confirm => Self::render_auto_confirm(area, buf, state),
            AutoCleanStep::CleanProgress => Self::render_auto_clean_progress(area, buf, state),
            AutoCleanStep::FinalReport => Self::render_auto_report(area, buf, state),
        }
    }

    fn render_auto_welcome(area: Rect, buf: &mut Buffer) {
        let colors = DialogColors::dark();
        let inner = centered_rect(60, 14, area);

        let block = Block::default()
            .title(" 🚀 ULTRA MEGA AUTO CLEAN ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.accent))
            .style(Style::default().bg(colors.bg));
        Widget::render(block, inner, buf);

        let content = Rect::new(inner.x + 1, inner.y + 1, inner.width - 2, inner.height - 2);
        let mut yi = content.y;

        let lines = vec![
            "",
            "  ╔══════════════════════════════════════╗",
            "  ║     WinDebloat — ULTRA AUTO CLEAN    ║",
            "  ╚══════════════════════════════════════╝",
            "",
            "  30-stage full-detailed system cleaning",
            "",
            "  • All categories are scanned",
            "  • Risk assessment is performed",
            "  • Safe cleaning with backup",
            "  • Detailed report is presented",
            "",
            "  [Enter] Start  [Esc] Cancel",
        ];

        for line in &lines {
            buf.set_string(content.x, yi, line, Style::default().fg(colors.text));
            yi += 1;
        }
    }

    fn render_auto_scan_progress(area: Rect, buf: &mut Buffer, state: &AutoCleanState) {
        let colors = DialogColors::dark();
        let inner = centered_rect(72, 26, area);

        let block = Block::default()
            .title(" 🚀 ULTRA AUTO CLEAN — SCANNING ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.accent))
            .style(Style::default().bg(colors.bg));
        Widget::render(block, inner, buf);

        let content = Rect::new(inner.x + 1, inner.y + 1, inner.width - 2, inner.height - 2);
        let total_phases = state.total_phases.max(1);
        let overall = state.phase_index as f32 / total_phases as f32;

        let pbar = ProgressWidget::new(overall, content.width - 2)
            .with_color(colors.accent);
        pbar.render(buf, content.x, content.y);

        let elapsed = state.start_time.elapsed();
        let info = format!(" ⏱ {:02}:{:02}  |  Stage {}/{}  |  {}",
            elapsed.as_secs() / 60, elapsed.as_secs() % 60,
            state.phase_index + 1, total_phases, state.phase_label);
        buf.set_string(content.x, content.y + 1, &info, Style::default().fg(colors.text));

        let phases = AutoCleanPhase::all();
        let max_rows = (content.height as usize - 5).min(phases.len());
        let mut yi = content.y + 3;

        for (i, phase) in phases.iter().enumerate().take(max_rows) {
            let completed = state.completed_phases.contains(phase);
            let is_current = state.phase_index >= i && state.completed_phases.len() <= i;
            let (icon, status, fg) = if completed {
                ("✅", "✓", colors.success)
            } else if is_current {
                ("⏳", "▸", colors.accent)
            } else {
                ("  ", " ", colors.text_dim)
            };

            buf.set_string(content.x, yi, &format!(" {} {} {} ", icon, phase.icon(), phase.label()),
                Style::default().fg(if completed { colors.success } else if is_current { colors.accent } else { colors.text_dim }));
            yi += 1;
        }

        buf.set_string(content.x, content.y + content.height - 2,
            &format!(" Scanned: {} items | {} | Speed: {} MB/s",
                state.items_scanned,
                crate::utils::formatting::format_bytes(state.bytes_scanned),
                state.scan_speed as u64),
            Style::default().fg(colors.text_dim));
        buf.set_string(content.x, content.y + content.height - 1,
            " [Esc] Cancel",
            Style::default().fg(colors.text_dim).italic());
    }

    fn render_auto_review(area: Rect, buf: &mut Buffer, state: &AutoCleanState) {
        let colors = DialogColors::dark();
        let height = (state.categorized.len() as u16 * 2 + 8).min(28);
        let inner = centered_rect(72, height, area);

        let block = Block::default()
            .title(format!(" 📋 SELECTION — {} items / {}", state.all_items.len(), crate::utils::formatting::format_bytes(state.all_items.iter().map(|i| i.size).sum())))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.accent))
            .style(Style::default().bg(colors.bg));
        Widget::render(block, inner, buf);

        let content = Rect::new(inner.x + 1, inner.y + 1, inner.width - 2, inner.height - 2);
        let mut yi = content.y;

        buf.set_string(content.x, yi, &format!(" SELECTION: {}/{} items | {} / {} | 🟢 {} cat.",
            state.items_cleaned, state.all_items.len(),
            crate::utils::formatting::format_bytes(state.bytes_cleaned),
            crate::utils::formatting::format_bytes(state.all_items.iter().map(|i| i.size).sum()),
            state.selected_category_ids.len()),
            Style::default().fg(colors.warning));
        yi += 1;

        buf.set_string(content.x, yi, &"─".repeat(content.width.min(70) as usize), Style::default().fg(colors.border));
        yi += 1;

        for cat in &state.categorized {
            let is_selected = state.selected_category_ids.contains(&cat.id);
            let marker = if is_selected { "✓" } else { " " };
            let risk_icon = match cat.risk.level {
                RiskLevel::Safe => "\u{1f7e2}",
                RiskLevel::Caution => "\u{1f7e1}",
                RiskLevel::Warning => "\u{1f7e0}",
                RiskLevel::Dangerous => "\u{1f534}",
            };
            let line = format!(" [{}] {} {}  {} items  {}  {}",
                marker, cat.icon, cat.name, cat.total_items,
                crate::utils::formatting::format_bytes(cat.total_size),
                risk_icon);
            buf.set_string(content.x, yi, &line, Style::default().fg(if is_selected { colors.success } else { colors.text }));
            yi += 1;

            if !cat.sub_groups.is_empty() && yi < content.y + content.height {
                for sub in &cat.sub_groups {
                    buf.set_string(content.x + 4, yi,
                        &format!(" ├─ {}: {} items, {}", sub.name, sub.count, crate::utils::formatting::format_bytes(sub.total_size)),
                        Style::default().fg(colors.text_dim));
                    yi += 1;
                    if yi >= content.y + content.height { break; }
                }
            }
        }

        buf.set_string(content.x, content.y + content.height - 1,
            " [↑↓] Navigate  [Space] Toggle  [Enter] Continue  [Esc] Cancel",
            Style::default().fg(colors.text_dim).italic());
    }

    fn render_auto_confirm(area: Rect, buf: &mut Buffer, state: &AutoCleanState) {
        let colors = DialogColors::dark();
        let inner = centered_rect(60, 18, area);

        let block = Block::default()
            .title(" ⚠ FINAL CONFIRMATION — MEGA ULTRA CLEAN ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.danger))
            .style(Style::default().bg(colors.bg));
        Widget::render(block, inner, buf);

        let content = Rect::new(inner.x + 1, inner.y + 1, inner.width - 2, inner.height - 2);
        let mut yi = content.y;

        let warn = " WARNING: Selected items will be PERMANENTLY deleted!";
        buf.set_string(content.x, yi, warn, Style::default().fg(colors.danger).add_modifier(Modifier::BOLD));
        yi += 2;

        let total_items: usize = state.categorized.iter()
            .filter(|c| state.selected_category_ids.contains(&c.id))
            .map(|c| c.total_items)
            .sum();
        let total_size: u64 = state.categorized.iter()
            .filter(|c| state.selected_category_ids.contains(&c.id))
            .map(|c| c.total_size)
            .sum();

        let lines = vec![
            format!(" Selected category:       {}/{}", state.selected_category_ids.len(), state.categorized.len()),
            format!(" Selected items:         {} / {}", total_items, state.all_items.len()),
            format!(" Space to clean:     {} / {}", crate::utils::formatting::format_bytes(total_size), crate::utils::formatting::format_bytes(state.all_items.iter().map(|i| i.size).sum())),
            format!(" Estimated time:        ~{} seconds", (total_size / (50 * 1024 * 1024)).max(10)),
        ];

        for line in &lines {
            buf.set_string(content.x, yi, line, Style::default().fg(colors.text));
            yi += 1;
        }

        yi += 1;
        buf.set_string(content.x, yi, " 🛡 All deleted items will be backed up and are restorable.",
            Style::default().fg(colors.accent));
        yi += 2;

        buf.set_string(content.x, yi, " [Y] CONFIRM  [S] Safe only  [N] Cancel",
            Style::default().fg(colors.danger).add_modifier(Modifier::BOLD));
    }

    fn render_auto_clean_progress(area: Rect, buf: &mut Buffer, state: &AutoCleanState) {
        let colors = DialogColors::dark();
        let inner = centered_rect(72, 22, area);

        let block = Block::default()
            .title(" 🧹 CLEANING PROGRESS ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.accent))
            .style(Style::default().bg(colors.bg));
        Widget::render(block, inner, buf);

        let content = Rect::new(inner.x + 1, inner.y + 1, inner.width - 2, inner.height - 2);
        let elapsed = state.start_time.elapsed();
        let total_items = state.items_cleaned.max(1);
        let overall = state.items_cleaned as f32 / (state.items_cleaned + state.items_failed + state.items_skipped).max(1) as f32;

        let pbar = ProgressWidget::new(overall, content.width - 2)
            .with_color(colors.accent);
        pbar.render(buf, content.x, content.y);

        let info = format!(" ⏱ {:02}:{:02}  |  Speed: {} MB/s  |  Remaining: ~{}sec",
            elapsed.as_secs() / 60, elapsed.as_secs() % 60,
            state.clean_speed as u64,
            if state.clean_speed > 0.0 {
                ((state.bytes_cleaned as f64 / state.clean_speed) / 1024.0 / 1024.0) as u64
            } else { 0 });
        buf.set_string(content.x, content.y + 1, &info, Style::default().fg(colors.text));

        let mut yi = content.y + 3;
        for cat in &state.categorized {
            if yi >= content.y + content.height - 3 { break; }
            let cat_total = cat.total_items;
            let icon = if state.completed_phases.len() > 5 { "✓" } else { "▸" };
            let line = format!(" {} {} {}: {}/{} items  {}",
                icon, cat.icon, cat.name, cat.items.len().min(cat_total), cat_total,
                crate::utils::formatting::format_bytes(cat.total_size));
            buf.set_string(content.x, yi, &line, Style::default().fg(colors.text));
            yi += 1;
        }

        yi = content.y + content.height - 2;
        buf.set_string(content.x, yi,
            &format!(" Total: {}/{} cleaned  |  {}  |  Errors: {}",
                state.items_cleaned, state.all_items.len(),
                crate::utils::formatting::format_bytes(state.bytes_cleaned),
                state.items_failed),
            Style::default().fg(colors.text_dim));
    }

    fn render_auto_report(area: Rect, buf: &mut Buffer, _state: &AutoCleanState) {
        let colors = DialogColors::dark();
        let inner = centered_rect(70, 20, area);

        let block = Block::default()
            .title(" ✅ ULTRA MEGA AUTO CLEAN — COMPLETE! ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.success))
            .style(Style::default().bg(colors.bg));
        Widget::render(block, inner, buf);

        let content = Rect::new(inner.x + 1, inner.y + 1, inner.width - 2, inner.height - 2);
        let mut yi = content.y;

        buf.set_string(content.x, yi,
            &format!(" {} ITEMS CLEANED  |  {} SPACE RECOVERED",
                _state.items_cleaned, crate::utils::formatting::format_bytes(_state.bytes_cleaned)),
            Style::default().fg(colors.success).add_modifier(Modifier::BOLD));
        yi += 2;

        buf.set_string(content.x, yi,
            &format!(" Time: {:02}:{:02}  |  Errors: {}  |  Success: %{:.1}",
                _state.start_time.elapsed().as_secs() / 60,
                _state.start_time.elapsed().as_secs() % 60,
                _state.items_failed,
                if _state.items_cleaned + _state.items_failed > 0 {
                    _state.items_cleaned as f32 / (_state.items_cleaned + _state.items_failed) as f32 * 100.0
                } else { 100.0 }),
            Style::default().fg(colors.text));
        yi += 2;

        buf.set_string(content.x, yi, " ── CATEGORY BREAKDOWN ──",
            Style::default().fg(colors.accent));
        yi += 1;

        for cat in &_state.categorized {
            if yi >= content.y + content.height - 2 { break; }
            let pct = if cat.total_items > 0 {
                let cleaned = cat.items.len().min(cat.total_items);
                cleaned as f32 / cat.total_items as f32 * 100.0
            } else { 0.0 };
            let bar_len = (pct / 100.0 * 25.0) as usize;
            let bar: String = std::iter::repeat('█').take(bar_len).collect();
            buf.set_string(content.x, yi,
                &format!(" {} {} {:<15} {:>3.0}% {:<25} {}",
                    cat.icon, if pct > 90.0 { "✓" } else { " " },
                    cat.name, pct, bar,
                    crate::utils::formatting::format_bytes(cat.total_size)),
                Style::default().fg(if pct > 90.0 { colors.success } else { colors.warning }));
            yi += 1;
        }

        yi = content.y + content.height - 1;
        buf.set_string(content.x, yi,
            " [Enter] Close  [U] Undo  [E] Save report",
            Style::default().fg(colors.text_dim).italic());
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AutoCleanStep {
    Welcome,
    ScanProgress,
    CategoryReview,
    Detail,
    RiskReview,
    Confirm,
    CleanProgress,
    FinalReport,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DiskWizardState {
    pub step: WizardStep,
    pub analysis: DiskAnalysis,
    pub selected_categories: Vec<String>,
    pub selected_items: Vec<CleanItem>,
    pub cursor: usize,
    pub total_size: u64,
}

impl DiskWizardState {
    pub fn new() -> Self {
        DiskWizardState {
            step: WizardStep::Overview,
            analysis: DiskAnalysis::default(),
            selected_categories: Vec::new(),
            selected_items: Vec::new(),
            cursor: 0,
            total_size: 0,
        }
    }
}

impl Default for DiskWizardState {
    fn default() -> Self {
        Self::new()
    }
}

fn render_wizard_overview(area: Rect, buf: &mut Buffer, wizard: &DiskWizardState, colors: &DialogColorSet) {
    let mut lines = vec![
        Line::from(vec![Span::styled(" STEP 1: Disk Overview", Style::default().fg(colors.accent).bold())]),
        Line::from(""),
        Line::from(vec![Span::styled(" Your disk usage summary:", Style::default().fg(colors.text))]),
        Line::from(""),
    ];

    for cat in &wizard.analysis.by_category {
        let selected = wizard.selected_categories.contains(&cat.id);
        let marker = if selected { "[*]" } else { "[ ]" };
        lines.push(Line::from(vec![
            Span::styled(marker, Style::default().fg(if selected { colors.success } else { colors.text_dim })),
            Span::raw(" "),
            Span::styled(&cat.name, Style::default().fg(colors.text)),
            Span::raw(" "),
            Span::styled(format!("({})", format_size(cat.size)), Style::default().fg(colors.text_dim)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        format!(" Total cleanable: {}", format_size(wizard.analysis.total_cleanable)),
        Style::default().fg(colors.warning).bold(),
    )]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        " [Space] Select categories  [Enter] Next  [Esc] Cancel",
        Style::default().fg(colors.text_dim),
    )]));

    Paragraph::new(Text::from(lines))
        .alignment(Alignment::Left)
        .render(area, buf);
}

fn render_wizard_categories(area: Rect, buf: &mut Buffer, wizard: &DiskWizardState, colors: &DialogColorSet) {
    let mut lines = vec![
        Line::from(vec![Span::styled(" STEP 2: Select Categories", Style::default().fg(colors.accent).bold())]),
        Line::from(""),
        Line::from(vec![Span::styled(" Choose what to clean:", Style::default().fg(colors.text))]),
        Line::from(""),
    ];

    let categories = vec![
        ("Browser Caches", "browser", "Clear browser cache files"),
        ("System Caches", "system", "Clean system caches and logs"),
        ("Old Downloads", "downloads", "Remove old downloaded files"),
        ("Large Files", "large", "Find and remove large files"),
        ("Containers", "containers", "Docker, Flatpak, Snap cleanup"),
        ("Development", "dev", "npm, cargo, pip caches"),
        ("Thumbnails", "thumbnails", "Clear thumbnail cache"),
        ("Trash", "trash", "Empty trash directories"),
    ];

    for (i, (name, id, desc)) in categories.iter().enumerate() {
        let selected = wizard.selected_categories.contains(&id.to_string());
        let marker = if i == wizard.cursor { ">" } else { " " };
        let check = if selected { "[*]" } else { "[ ]" };
        lines.push(Line::from(vec![
            Span::styled(marker, Style::default().fg(colors.warning)),
            Span::styled(check, Style::default().fg(if selected { colors.success } else { colors.text_dim })),
            Span::raw(" "),
            Span::styled(*name, Style::default().fg(colors.text)),
            Span::raw(" - "),
            Span::styled(*desc, Style::default().fg(colors.text_dim)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        " [Up/Down] Navigate  [Space] Toggle  [Enter] Next  [Esc] Back",
        Style::default().fg(colors.text_dim),
    )]));

    Paragraph::new(Text::from(lines))
        .alignment(Alignment::Left)
        .render(area, buf);
}

fn render_wizard_preview(area: Rect, buf: &mut Buffer, wizard: &DiskWizardState, colors: &DialogColorSet) {
    let mut lines = vec![
        Line::from(vec![Span::styled(" STEP 3: Preview", Style::default().fg(colors.accent).bold())]),
        Line::from(""),
        Line::from(vec![Span::styled(" Items to be cleaned:", Style::default().fg(colors.text))]),
        Line::from(""),
    ];

    for (i, item) in wizard.selected_items.iter().enumerate().take(15) {
        let style = if i == wizard.cursor {
            Style::default().fg(colors.text).bold()
        } else {
            Style::default().fg(colors.text_dim)
        };
        lines.push(Line::from(vec![
            Span::styled(&item.description, style),
            Span::raw(" "),
            Span::styled(format!("({})", format_size(item.size)), Style::default().fg(colors.warning)),
        ]));
    }

    if wizard.selected_items.len() > 15 {
        let more_msg = format!("... and {} more items", wizard.selected_items.len() - 15);
        lines.push(Line::from(vec![Span::styled(
            more_msg,
            Style::default().fg(colors.text_dim),
        )]));
    }

    lines.push(Line::from(""));
    let total_msg = format!(" Total: {} items ({})", wizard.selected_items.len(), format_size(wizard.total_size));
    lines.push(Line::from(vec![Span::styled(
        total_msg,
        Style::default().fg(colors.warning).bold(),
    )]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        " [Enter] Start Cleaning  [Esc] Back",
        Style::default().fg(colors.text_dim),
    )]));

    Paragraph::new(Text::from(lines))
        .alignment(Alignment::Left)
        .render(area, buf);
}

fn render_wizard_confirm(area: Rect, buf: &mut Buffer, wizard: &DiskWizardState, colors: &DialogColorSet) {
    let delete_msg = format!(" You are about to delete {} items", wizard.selected_items.len());
    let size_msg = format!(" Total size: {}", format_size(wizard.total_size));

    let lines = vec![
        Line::from(vec![Span::styled(" FINAL CONFIRMATION", Style::default().fg(colors.danger).bold())]),
        Line::from(""),
        Line::from(vec![Span::styled(delete_msg, Style::default().fg(colors.text))]),
        Line::from(vec![Span::styled(size_msg, Style::default().fg(colors.warning))]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " All deletions will be backed up for rollback.",
            Style::default().fg(colors.text_dim),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " [Y] Confirm  [N] Cancel",
            Style::default().fg(if wizard.cursor == 0 { colors.success } else { colors.text_dim }),
        )]),
    ];

    Paragraph::new(Text::from(lines))
        .alignment(Alignment::Center)
        .render(area, buf);
}

fn render_wizard_cleaning(area: Rect, buf: &mut Buffer, wizard: &DiskWizardState, colors: &DialogColorSet) {
    let remaining_msg = format!(" {} items remaining", wizard.selected_items.len());

    let lines = vec![
        Line::from(vec![Span::styled(" Cleaning in progress...", Style::default().fg(colors.accent).bold())]),
        Line::from(""),
        Line::from(vec![Span::styled(remaining_msg, Style::default().fg(colors.text))]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " Please wait...",
            Style::default().fg(colors.text_dim).italic(),
        )]),
    ];

    Paragraph::new(Text::from(lines))
        .alignment(Alignment::Center)
        .render(area, buf);
}

fn render_wizard_done(area: Rect, buf: &mut Buffer, wizard: &DiskWizardState, colors: &DialogColorSet) {
    let cleaned_msg = format!(" Successfully cleaned {} items", wizard.selected_items.len());
    let freed_msg = format!(" Freed space: {}", format_size(wizard.total_size));

    let lines = vec![
        Line::from(vec![Span::styled(" Cleaning Complete!", Style::default().fg(colors.success).bold())]),
        Line::from(""),
        Line::from(vec![Span::styled(cleaned_msg, Style::default().fg(colors.text))]),
        Line::from(vec![Span::styled(freed_msg, Style::default().fg(colors.warning))]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " All changes have been logged. Use 'U' to undo.",
            Style::default().fg(colors.text_dim),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " Press any key to close",
            Style::default().fg(colors.text_dim).italic(),
        )]),
    ];

    Paragraph::new(Text::from(lines))
        .alignment(Alignment::Center)
        .render(area, buf);
}

pub fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let x = r.x.saturating_add(r.width.saturating_sub(width) / 2);
    let y = r.y.saturating_add(r.height.saturating_sub(height) / 2);
    Rect::new(x, y, width.min(r.width), height.min(r.height))
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1} {}", size, UNITS[unit_idx])
}