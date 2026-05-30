use ratatui::prelude::*;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use super::components::content::ContentPanel;
use super::components::dialogs::{AutoCleanStep, Dialogs, Dialog, DiskWizardState};
use super::components::sidebar::Sidebar;
use crate::config::schema::UiConfig;
use crate::core::auto_clean::AutoCleanState;
use crate::core::backup::BackupManifest;
use crate::core::scanner::{Category, ScanResult};

#[derive(Debug, Clone, PartialEq)]
pub struct ContextMenu {
    pub x: u16,
    pub y: u16,
    pub item_idx: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClickTarget {
    SidebarItem(usize),
    ContentRow(usize),
    ContentCheckbox(usize),
    ContentScrollbar { y: u16 },
    ContextMenuOption(usize),
    TitleBar,
    StatusBar,
    Dialog,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelFocus {
    Sidebar,
    Content,
}

pub struct AppUi {
    pub sidebar: Sidebar,
    pub content: ContentPanel,
    pub dialog: Dialog,
    pub help_visible: bool,
    pub status_bar: String,
    pub distro_name: String,
    pub undo_selected: usize,
    pub backups: Vec<BackupManifest>,
    pub config: UiConfig,
    pub wizard: Option<DiskWizardState>,
    pub auto_clean: Option<AutoCleanState>,
    pub auto_clean_step: AutoCleanStep,
    pub focus: PanelFocus,
    pub render_area: Rect,
    pub context_menu: Option<ContextMenu>,
    pub hovered_sidebar: Option<usize>,
    pub hovered_content_row: Option<usize>,
}



impl AppUi {
    pub fn new(distro_name: &str, config: UiConfig) -> Self {
        AppUi {
            sidebar: Sidebar::new(config.sidebar_width),
            content: ContentPanel::new(),
            dialog: Dialog::None,
            help_visible: false,
            status_bar: String::from("Ready: [S] Scan All  |  [A] Analyze  |  [?] Help  |  [Q] Quit"),
            distro_name: distro_name.to_string(),
            undo_selected: 0,
            backups: Vec::new(),
            config,
            wizard: None,
            auto_clean: None,
            auto_clean_step: AutoCleanStep::Welcome,
            focus: PanelFocus::Sidebar,
            render_area: Rect::default(),
            context_menu: None,
            hovered_sidebar: None,
            hovered_content_row: None,
        }
    }

    pub fn render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        results: &[ScanResult],
        active_category: Category,
    ) {
        self.render_area = area;
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(area);

        self.render_title_bar(layout[0], buf);
        self.render_main_area(layout[1], buf, results, active_category);
        self.render_status_bar(layout[2], buf);

        self.render_dialogs(area, buf);
    }

    pub fn resolve_click(&self, x: u16, y: u16, area: Rect) -> ClickTarget {
        let main = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(area);

        if y == main[0].y {
            return ClickTarget::TitleBar;
        }
        if y == main[2].y {
            return ClickTarget::StatusBar;
        }

        let main_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(self.sidebar.width),
                Constraint::Min(20),
            ])
            .split(main[1]);

        if x >= main_area[0].x && x < main_area[0].x + main_area[0].width {
            if let Some(idx) = self.sidebar.hit_test(x, y) {
                return ClickTarget::SidebarItem(idx);
            }
        }

        if x >= main_area[1].x && x < main_area[1].x + main_area[1].width {
            if let Some(target) = self.content.hit_test(x, y) {
                return match target {
                    super::components::content::ContentClickTarget::Row(idx) => ClickTarget::ContentRow(idx),
                    super::components::content::ContentClickTarget::Checkbox(idx) => ClickTarget::ContentCheckbox(idx),
                    super::components::content::ContentClickTarget::Scrollbar { y } => ClickTarget::ContentScrollbar { y },
                };
            }
        }

        if self.dialog != Dialog::None || self.wizard.is_some() || self.auto_clean.is_some() {
            return ClickTarget::Dialog;
        }

        ClickTarget::None
    }

    fn render_title_bar(&self, area: Rect, buf: &mut Buffer) {
        let colors = ThemeColors::dark();
        
        let logo = "◆ WinDebloat";
        let version = env!("CARGO_PKG_VERSION");
        let distro = &self.distro_name;
        
        let left = format!(" {} v{} ", logo, version);
        let right = format!(" {} ", distro);
        
        let width = area.width as usize;
        let left_len = left.len();
        let right_len = right.len();
        let sep_len = width.saturating_sub(left_len + right_len);
        
        let separator = "─".repeat(sep_len.min(200));

        buf.set_string(area.x, area.y, &left, Style::default()
            .fg(colors.accent)
            .add_modifier(Modifier::BOLD)
            .bg(colors.bg));
            
        buf.set_string(area.x + left_len as u16, area.y, &separator, Style::default()
            .fg(colors.border)
            .bg(colors.bg));
            
        let right_x = area.x + (left_len + sep_len) as u16;
        buf.set_string(right_x, area.y, &right, Style::default()
            .fg(colors.text_dim)
            .bg(colors.bg));
    }

    fn render_main_area(&mut self, area: Rect, buf: &mut Buffer, results: &[ScanResult], active_category: Category) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(self.sidebar.width),
                Constraint::Min(20),
            ])
            .split(area);

        for result in results {
            self.sidebar.set_count(result.category, result.total_items);
        }

        self.sidebar.hovered_idx = self.hovered_sidebar;
        self.sidebar.render(layout[0], buf, &self.config, self.focus == PanelFocus::Sidebar);

        let current_result = results
            .iter()
            .find(|r| r.category == active_category)
            .cloned();

        self.content.hovered_idx = self.hovered_content_row;
        self.content.render(layout[1], buf, &current_result, active_category, "", self.focus == PanelFocus::Content);
    }

    fn render_status_bar(&self, area: Rect, buf: &mut Buffer) {
        let colors = ThemeColors::dark();
        
        let is_scanning = self.status_bar.contains("Scanning");
        let base_color = if is_scanning { colors.warning } else { colors.accent };

        buf.set_string(area.x, area.y, " ", Style::default().bg(base_color).fg(base_color));

        buf.set_string(area.x + 1, area.y, &self.status_bar, Style::default()
            .fg(colors.bg)
            .bg(base_color));

        let padding_len = (area.width as usize).saturating_sub(self.status_bar.len() + 2);
        let padding = " ".repeat(padding_len);
        buf.set_string(area.x + 1 + self.status_bar.len() as u16, area.y, &padding, Style::default()
            .fg(base_color)
            .bg(base_color));
    }

    fn render_dialogs(&mut self, area: Rect, buf: &mut Buffer) {
        if let Some(ref wizard) = self.wizard {
            Dialogs::render_wizard(area, buf, wizard);
            return;
        }

        if let Some(ref auto_clean) = self.auto_clean {
            Dialogs::render_auto_clean(area, buf, auto_clean, &self.auto_clean_step);
            return;
        }

        match &self.dialog {
            Dialog::ConfirmClean { total_items, total_size } => {
                Dialogs::render_confirm(area, buf, *total_items, total_size);
            }
            Dialog::Progress { current, total, current_file } => {
                Dialogs::render_progress(area, buf, *current, *total, current_file);
            }
            Dialog::Report { message, success } => {
                Dialogs::render_report(area, buf, message, *success);
            }
            Dialog::UndoList { selected, backup_count: _ } => {
                Dialogs::render_undo(area, buf, *selected, &self.backups);
            }
            Dialog::Search { query } => {
                Dialogs::render_search(area, buf, query);
            }
            Dialog::Quit => {
                Dialogs::render_quit(area, buf);
            }
            Dialog::DiskWizard { .. } => {
                if let Some(ref mut wizard) = self.wizard {
                    Dialogs::render_wizard(area, buf, wizard);
                }
            }
            Dialog::None => {}
        }

        if self.help_visible {
            Dialogs::render_help(area, buf);
        }
    }
}

pub struct ThemeColors {
    pub bg: Color,
    pub border: Color,
    pub header: Color,
    pub text: Color,
    pub text_dim: Color,
    pub accent: Color,
    pub warning: Color,
    pub danger: Color,
    pub success: Color,
    pub selected: Color,
    pub selected_bg: Color,
}

impl ThemeColors {
    pub fn dark() -> Self {
        ThemeColors {
            bg: Color::Rgb(30, 30, 46),
            border: Color::Rgb(60, 60, 90),
            header: Color::Rgb(100, 180, 220),
            text: Color::Rgb(210, 210, 230),
            text_dim: Color::Rgb(100, 100, 120),
            accent: Color::Rgb(100, 220, 180),
            warning: Color::Rgb(255, 200, 80),
            danger: Color::Rgb(255, 100, 100),
            success: Color::Rgb(100, 220, 150),
            selected: Color::Rgb(255, 220, 120),
            selected_bg: Color::Rgb(50, 50, 80),
        }
    }
}