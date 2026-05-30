use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, ListState},
};

use crate::core::scanner::{Category, CleanItem, ScanResult};

pub struct ContentPanel {
    pub list_state: ListState,
    pub details_focus: bool,
    pub wizard_view: WizardView,
    pub selected_presets: Vec<usize>,
    pub search_query: String,
    pub filter_size_min: Option<u64>,
    pub filter_age_days: Option<u64>,
    pub render_area: Option<Rect>,
    pub active_category: Category,
    pub scroll_offset: usize,
    pub hovered_idx: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContentClickTarget {
    Row(usize),
    Checkbox(usize),
    Scrollbar { y: u16 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum WizardView {
    Overview,
    DirectoryTree,
    FileTypes,
    Containers,
    Presets,
    Search,
    Preview,
    Confirm,
    Done,
}

impl ContentPanel {
    pub fn new() -> Self {
        ContentPanel {
            list_state: ListState::default(),
            details_focus: false,
            wizard_view: WizardView::Overview,
            selected_presets: Vec::new(),
            search_query: String::new(),
            filter_size_min: None,
            filter_age_days: None,
            render_area: None,
            active_category: Category::Packages,
            scroll_offset: 0,
            hovered_idx: None,
        }
    }

    pub fn previous(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let next = if current == 0 { len - 1 } else { current - 1 };
        self.list_state.select(Some(next));
    }

    pub fn next(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let current = self.list_state.selected().unwrap_or(len.saturating_sub(1));
        let next = if current >= len.saturating_sub(1) { 0 } else { current + 1 };
        self.list_state.select(Some(next));
    }

    pub fn hit_test(&self, x: u16, y: u16) -> Option<ContentClickTarget> {
        let area = self.render_area?;
        if self.active_category == Category::Wizard {
            return None; // Wizard handles clicks separately for now
        }
        let list_area = Rect::new(area.x + 1, area.y + 4, area.width - 2, area.height - 5);
        if y < list_area.y || y >= list_area.y + list_area.height {
            return None;
        }
        if x == area.x + area.width - 1 {
            return Some(ContentClickTarget::Scrollbar { y });
        }
        if x < list_area.x || x >= list_area.x + list_area.width - 1 {
            return None;
        }
        
        let index = (y - list_area.y) as usize + self.scroll_offset;
        
        if x >= list_area.x + 1 && x <= list_area.x + 2 {
            return Some(ContentClickTarget::Checkbox(index));
        }
        
        Some(ContentClickTarget::Row(index))
    }

    pub fn toggle_focus(&mut self) {
        self.details_focus = !self.details_focus;
    }

    pub fn toggle_selected(&mut self, items: &mut Vec<CleanItem>) {
        if let Some(i) = self.list_state.selected() {
            if i < items.len() {
                items[i].selected = !items[i].selected;
            }
        }
    }

    pub fn toggle_all(&mut self, items: &mut Vec<CleanItem>) {
        let all_selected = items.iter().all(|i| i.selected);
        for item in items.iter_mut() {
            item.selected = !all_selected;
        }
    }

    pub fn select_safe_only(&mut self, items: &mut Vec<CleanItem>) {
        for item in items.iter_mut() {
            item.selected = item.safe;
        }
    }

    pub fn toggle_preset(&mut self, index: usize) {
        if let Some(pos) = self.selected_presets.iter().position(|&i| i == index) {
            self.selected_presets.remove(pos);
        } else {
            self.selected_presets.push(index);
        }
    }

    pub fn next_view(&mut self) {
        use WizardView::*;
        self.wizard_view = match self.wizard_view {
            Overview => DirectoryTree,
            DirectoryTree => FileTypes,
            FileTypes => Containers,
            Containers => Presets,
            Presets => Search,
            Search => Preview,
            Preview => Confirm,
            Confirm => Done,
            Done => Overview,
        };
        self.list_state.select(Some(0));
    }

    pub fn prev_view(&mut self) {
        use WizardView::*;
        self.wizard_view = match self.wizard_view {
            Overview => Done,
            DirectoryTree => Overview,
            FileTypes => DirectoryTree,
            Containers => FileTypes,
            Presets => Containers,
            Search => Presets,
            Preview => Search,
            Confirm => Preview,
            Done => Confirm,
        };
        self.list_state.select(Some(0));
    }

    pub fn go_home(&mut self) {
        self.list_state.select(Some(0));
    }

    pub fn go_end(&mut self, len: usize) {
        if len > 0 {
            self.list_state.select(Some(len - 1));
        }
    }

    pub fn page_up(&mut self, _len: usize) {
        let current = self.list_state.selected().unwrap_or(0);
        let step = 10;
        self.list_state.select(Some(current.saturating_sub(step)));
    }

    pub fn page_down(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let step = 10;
        let target = current.saturating_add(step);
        self.list_state.select(Some(target.min(len - 1)));
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, result: &Option<ScanResult>, active_category: Category, _filter: &str, focused: bool) {
        self.render_area = Some(area);
        self.active_category = active_category;
        
        if active_category == Category::Wizard {
            self.render_ultra_wizard(area, buf, focused);
            return;
        }
        
        if active_category == Category::AutoClean {
            self.render_auto_clean(area, buf, focused);
            return;
        }

        let colors = ContentColors::dark();
        let border_color = if focused { colors.accent } else { colors.border };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(colors.bg));
        Widget::render(block, area, buf);

        let header_area = Rect::new(area.x + 1, area.y + 1, area.width - 2, 3);

        if let Some(result) = result {
            let title = Line::from(vec![
                Span::styled(result.category.name(), Style::default().fg(colors.header).add_modifier(Modifier::BOLD)),
            ]);
            Paragraph::new(title).render(header_area, buf);

            let info = format!(" {} items  |  {} ", result.total_items, result.total_size_formatted());
            buf.set_string(header_area.x, header_area.y + 1, &info, Style::default().fg(colors.text_dim));

            let list_area = Rect::new(area.x + 1, area.y + 4, area.width - 2, area.height - 5);
            let visible_items: Vec<(usize, &CleanItem)> = result.items.iter().enumerate()
                .filter(|(_, item)| _filter.is_empty() || item.description.to_lowercase().contains(&_filter.to_lowercase()))
                .collect();

            if visible_items.is_empty() {
                buf.set_string(list_area.x, list_area.y, " No items found. Press 'A' to scan.", Style::default().fg(colors.text_dim));
                return;
            }

            let max_visible = list_area.height as usize;
            
            // Auto scroll logic
            let selected = self.list_state.selected().unwrap_or(0);
            if selected < self.scroll_offset {
                self.scroll_offset = selected;
            } else if selected >= self.scroll_offset + max_visible {
                self.scroll_offset = selected - max_visible + 1;
            }

            let visible_slice = visible_items.iter().skip(self.scroll_offset).take(max_visible);

            for (i, (orig_idx, item)) in visible_slice.enumerate() {
                let y = list_area.y + i as u16;
                let is_selected = selected == *orig_idx;
                let marker = if is_selected { "▸" } else { " " };
                let safe_icon = if item.safe { "[*]" } else { "[!]" };
                let sel_icon = if item.selected { "◉" } else { "○" };
                let status_color = if item.safe { colors.success } else { colors.danger };
                let selected_color = if item.selected { colors.selected } else { colors.text };
                let is_hovered = self.hovered_idx == Some(*orig_idx);
                let bg = if is_selected {
                    colors.selected_bg
                } else if is_hovered {
                    Color::Rgb(40, 40, 60)
                } else {
                    colors.bg
                };

                buf.set_string(list_area.x, y, marker, Style::default().fg(colors.accent).bg(bg));
                buf.set_string(list_area.x + 1, y, sel_icon, Style::default().fg(selected_color).bg(bg));
                buf.set_string(list_area.x + 3, y, safe_icon, Style::default().fg(status_color).bg(bg));
                buf.set_string(list_area.x + 7, y, &item.description, Style::default().fg(if is_selected { colors.text_bright } else { colors.text }).bg(bg));

                let size_str = item.size_formatted();
                let size_x = area.x + area.width - size_str.len() as u16 - 3; // reserve 1 space for scrollbar
                buf.set_string(size_x, y, &size_str, Style::default().fg(colors.warning).bg(bg));
            }
            
            // Draw Scrollbar
            let total_items = visible_items.len();
            if total_items > max_visible {
                let scrollbar_x = area.x + area.width - 1;
                for y in list_area.y..(list_area.y + list_area.height) {
                    buf.set_string(scrollbar_x, y, "│", Style::default().fg(colors.border).bg(colors.bg));
                }
                let thumb_size = (max_visible as f32 / total_items as f32 * max_visible as f32).max(1.0) as u16;
                let max_offset = total_items - max_visible;
                let scroll_pos = (self.scroll_offset as f32 / max_offset as f32 * (max_visible as u16 - thumb_size) as f32) as u16;
                for i in 0..thumb_size {
                    buf.set_string(scrollbar_x, list_area.y + scroll_pos + i, "█", Style::default().fg(colors.accent).bg(colors.bg));
                }
            }
        } else {
            buf.set_string(area.x + 2, area.y + 2, " Press 'A' to scan this category", Style::default().fg(colors.text_dim));
        }
    }

    pub fn render_ultra_wizard(&mut self, area: Rect, buf: &mut Buffer, focused: bool) {
        let colors = ContentColors::dark();

        let border_color = if focused { colors.accent } else { Color::Rgb(255, 200, 80) };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(colors.bg));
        Widget::render(block, area, buf);

        let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);

        self.render_wizard_tabs(inner, buf, &colors);
        
        let content_y = inner.y + 6;
        let content_height = inner.height - 8;
        let content_area = Rect::new(inner.x + 1, content_y, inner.width - 2, content_height);

        match self.wizard_view {
            WizardView::Overview => self.render_disk_overview(content_area, buf, &colors),
            WizardView::DirectoryTree => self.render_directory_tree(content_area, buf, &colors),
            WizardView::FileTypes => self.render_file_types(content_area, buf, &colors),
            WizardView::Containers => self.render_containers(content_area, buf, &colors),
            WizardView::Presets => self.render_presets(content_area, buf, &colors),
            WizardView::Search => self.render_search(content_area, buf, &colors),
            WizardView::Preview => self.render_preview(content_area, buf, &colors),
            WizardView::Confirm => self.render_confirm(content_area, buf, &colors),
            WizardView::Done => self.render_done(content_area, buf, &colors),
        }
    }

    fn render_wizard_tabs(&mut self, area: Rect, buf: &mut Buffer, colors: &ContentColors) {
        let tabs = [
            ("1[Overview]", "Disk overview"),
            ("2[Dirs]", "Directory tree"),
            ("3[Types]", "File types"),
            ("4[Containers]", "Docker/VM"),
            ("5[Presets]", "Quick clean"),
            ("6[Search]", "Find files"),
            ("7[Preview]", "Review"),
            ("8[Confirm]", "Go!"),
        ];

        let tab_width = (area.width - 2) / 8;
        let active_idx = match self.wizard_view {
            WizardView::Overview => 0,
            WizardView::DirectoryTree => 1,
            WizardView::FileTypes => 2,
            WizardView::Containers => 3,
            WizardView::Presets => 4,
            WizardView::Search => 5,
            WizardView::Preview => 6,
            WizardView::Confirm | WizardView::Done => 7,
        };

        for (i, (tab, desc)) in tabs.iter().enumerate() {
            let x = area.x + 1 + (i as u16 * tab_width);
            let is_active = i == active_idx;
            let color = if is_active { Color::Rgb(255, 200, 80) } else { colors.text_dim };

            buf.set_string(x, area.y, tab, Style::default().fg(color).add_modifier(Modifier::BOLD));
            buf.set_string(x, area.y + 1, desc, Style::default().fg(colors.text_dim));
        }

        let sep_y = area.y + 3;
        buf.set_string(area.x + 1, sep_y, &"═".repeat((area.width - 2).min(80) as usize), Style::default().fg(colors.border));
    }

    fn render_disk_overview(&mut self, area: Rect, buf: &mut Buffer, colors: &ContentColors) {
        let title = Line::from(vec![Span::styled("◆ DISK USAGE TREEMAP ◆", Style::default().fg(colors.accent).add_modifier(Modifier::BOLD))]);
        Paragraph::new(title).render(Rect::new(area.x, area.y, area.width, 1), buf);

        let disk_data = vec![
            ("/home", 250_u64 * 1024 * 1024 * 1024, 0.45),
            ("/var/log", 50_u64 * 1024 * 1024 * 1024, 0.08),
            ("/tmp", 20_u64 * 1024 * 1024 * 1024, 0.03),
            ("/.cache", 15_u64 * 1024 * 1024 * 1024, 0.02),
            ("/snap", 100_u64 * 1024 * 1024 * 1024, 0.15),
        ];

        let mut y = area.y + 2;
        let max_bar_width = (area.width - 30) as usize;

        buf.set_string(area.x, y, "┌─────────────────────────────────────────────────────────────────┐", Style::default().fg(colors.border));
        y += 1;

        for (path, size, _) in &disk_data {
            let bar_len = (*size as f64 / (300_f64 * 1024_f64 * 1024_f64 * 1024_f64) * max_bar_width as f64) as usize;
            let bar_str = format!("{}{}", "█".repeat(bar_len.min(max_bar_width)), "░".repeat((max_bar_width - bar_len).max(0)));

            buf.set_string(area.x, y, &format!("│ {}: ", path), Style::default().fg(colors.text));
            buf.set_string(area.x + 15, y, &bar_str, Style::default().fg(colors.accent));
            buf.set_string(area.x + 15 + max_bar_width as u16 + 2, y, &format!("{} │", format_size(*size)), Style::default().fg(colors.warning));
            y += 1;
        }

        buf.set_string(area.x, y, "└─────────────────────────────────────────────────────────────────┘", Style::default().fg(colors.border));
        y += 2;

        buf.set_string(area.x, y, " PIE CHART BREAKDOWN ", Style::default().fg(colors.header).add_modifier(Modifier::BOLD));
        y += 1;

        let pie_data = [
            ("Videos", 45, colors.danger),
            ("Downloads", 25, colors.accent),
            ("Documents", 15, Color::Rgb(100, 180, 220)),
            ("Archives", 8, colors.warning),
            ("Other", 7, colors.text_dim),
        ];

        let _total_pct: u32 = pie_data.iter().map(|(_, p, _)| *p).sum();
        for (name, pct, color) in &pie_data {
            let bar_len = *pct as usize * max_bar_width / 100;
            let bar = "█".repeat(bar_len);
            buf.set_string(area.x, y, &format!("[{:>6}] {}", format!("{}%", pct), name), Style::default().fg(*color));
            buf.set_string(area.x + 20, y, &bar, Style::default().fg(*color));
            y += 1;
        }

        let help_y = area.y + area.height - 1;
        buf.set_string(area.x, help_y, "[←/→] Change View  [Space] Select  [Enter] Next Step", Style::default().fg(colors.text_dim));
    }

    fn render_directory_tree(&mut self, area: Rect, buf: &mut Buffer, colors: &ContentColors) {
        let title = Line::from(vec![Span::styled("◆ NESTED DIRECTORY TREE ◆", Style::default().fg(colors.accent).add_modifier(Modifier::BOLD))]);
        Paragraph::new(title).render(Rect::new(area.x, area.y, area.width, 1), buf);

        let tree_data = vec![
            ("📁", "/home/trk", 0.0_f64, false),
            ("├──", "Downloads", 45.2_f64, true),
            ("│   ├──", "Videos", 25.1_f64, true),
            ("│   │   └──", "Movies", 18.5_f64, false),
            ("│   ├──", "Documents", 12.3_f64, true),
            ("│   │   ├──", "Projects", 8.2_f64, false),
            ("│   │   └──", "Archives", 3.1_f64, false),
            ("│   └──", "Music", 7.8_f64, true),
            ("├──", ".cache", 22.1_f64, true),
            ("│   ├──", "thumbnails", 2.3_f64, false),
            ("│   ├──", "mozilla", 8.5_f64, false),
            ("│   └──", "pip", 4.2_f64, false),
            ("├──", ".local", 35.0_f64, true),
            ("│   ├──", "share", 28.0_f64, false),
            ("│   └──", "state", 7.0_f64, false),
            ("└──", ".config", 15.0_f64, true),
        ];

        let mut y = area.y + 2;
        let max_items = (area.height as usize - 4).min(tree_data.len());

        for (i, (icon, name, size, expandable)) in tree_data.iter().enumerate().take(max_items) {
            let is_selected = self.list_state.selected() == Some(i);
            let marker = if is_selected { "▸" } else { " " };
            let color = if is_selected { colors.text_bright } else { colors.text };
            let bg = if is_selected { colors.selected_bg } else { colors.bg };

            buf.set_string(area.x, y, &format!("{} {} {}", marker, icon, name), Style::default().fg(color).bg(bg));
            
            if *size > 0.0 {
                let size_str = if *size >= 1.0 {
                    format!(" {:.1} GB", size)
                } else {
                    format!(" {:.0} MB", size * 1024.0)
                };
                let size_x = area.x + area.width - 15;
                buf.set_string(size_x, y, &size_str, Style::default().fg(colors.warning).bg(bg));
            }

            if *expandable {
                buf.set_string(area.x + area.width - 10, y, "[+]", Style::default().fg(colors.accent).bg(bg));
            }

            y += 1;
        }

        let help_y = area.y + area.height - 1;
        buf.set_string(area.x, help_y, "[↑↓] Navigate  [+] Expand  [Space] Select  [Enter] Select", Style::default().fg(colors.text_dim));
    }

    fn render_file_types(&mut self, area: Rect, buf: &mut Buffer, colors: &ContentColors) {
        let title = Line::from(vec![Span::styled("◆ FILE TYPE BREAKDOWN ◆", Style::default().fg(colors.accent).add_modifier(Modifier::BOLD))]);
        Paragraph::new(title).render(Rect::new(area.x, area.y, area.width, 1), buf);

        let file_types = vec![
            ("Videos", vec![".mp4", ".mkv", ".avi", ".mov", ".wmv"], 134.5_f64, 53),
            ("Archives", vec![".zip", ".tar", ".gz", ".rar", ".7z"], 22.1_f64, 9),
            ("Audio", vec![".mp3", ".flac", ".wav", ".aac"], 18.2_f64, 7),
            ("Images", vec![".jpg", ".png", ".raw", ".gif"], 15.8_f64, 6),
            ("Documents", vec![".pdf", ".doc", ".docx", ".txt"], 12.3_f64, 5),
            ("ISO/Img", vec![".iso", ".img", ".dmg"], 8.5_f64, 3),
            ("Other", vec![], 62.4_f64, 25),
        ];

        let mut y = area.y + 2;
        let max_bar_width = 40;

        for (i, (name, exts, size, pct)) in file_types.iter().enumerate() {
            let is_selected = self.list_state.selected() == Some(i);
            let marker = if is_selected { "▸" } else { " " };
            let color = if is_selected { colors.text_bright } else { colors.text };
            let bg = if is_selected { colors.selected_bg } else { colors.bg };

            let bar_len = *pct as usize * max_bar_width / 100;
            let bar = "█".repeat(bar_len);

            buf.set_string(area.x, y, &format!("{} {}", marker, name), Style::default().fg(color).bg(bg));
            buf.set_string(area.x + 12, y, "[", Style::default().fg(colors.text_dim).bg(bg));
            buf.set_string(area.x + 13, y, &bar, Style::default().fg(colors.accent).bg(bg));
            buf.set_string(area.x + 13 + max_bar_width as u16, y, &format!("] {}%", pct), Style::default().fg(colors.text_dim).bg(bg));
            
            let size_str = format!("{:.1} GB", size);
            let size_x = area.x + area.width - size_str.len() as u16 - 3;
            buf.set_string(size_x, y, &size_str, Style::default().fg(colors.warning).bg(bg));

            y += 1;
            buf.set_string(area.x + 4, y, &format!("Extensions: {}", exts.join(", ")), Style::default().fg(colors.text_dim).bg(bg));
            y += 2;
        }

        let help_y = area.y + area.height - 1;
        buf.set_string(area.x, help_y, "[↑↓] Navigate  [Space] Select Category  [Enter] Scan Files", Style::default().fg(colors.text_dim));
    }

    fn render_containers(&mut self, area: Rect, buf: &mut Buffer, colors: &ContentColors) {
        let title = Line::from(vec![Span::styled("◆ CONTAINER & VM MANAGEMENT ◆", Style::default().fg(colors.accent).add_modifier(Modifier::BOLD))]);
        Paragraph::new(title).render(Rect::new(area.x, area.y, area.width, 1), buf);

        let containers = vec![
            ("🐳", "Docker", true, vec![
                ("Images (15)", 8.2_f64),
                ("Containers (3)", 1.1_f64),
                ("Volumes (8)", 4.3_f64),
                ("Build Cache", 2.8_f64),
            ]),
            ("📦", "Podman", true, vec![
                ("Images (8)", 3.5_f64),
                ("Containers (1)", 0.3_f64),
            ]),
            ("🎮", "Flatpak", true, vec![
                ("Apps", 3.2_f64),
                ("Runtimes", 4.1_f64),
                ("Cache", 0.8_f64),
            ]),
            ("🐛", "Snap", false, vec![
                ("Packages (12)", 15.0_f64),
            ]),
            ("💻", "VirtualBox", false, vec![
                ("Win10_dev.vdi", 40.0_f64),
                ("Ubuntu_test.vdi", 25.0_f64),
            ]),
            ("☁️", "Libvirt/KVM", false, vec![
                ("Images (3)", 60.0_f64),
                ("Coredumps", 0.5_f64),
            ]),
        ];

        let mut y = area.y + 2;
        let max_items = (area.height as usize - 4).min(15);

        for (i, (icon, name, installed, items)) in containers.iter().enumerate().take(max_items) {
            let is_selected = self.list_state.selected() == Some(i);
            let marker = if is_selected { "▸" } else { " " };
            let color = if is_selected { colors.text_bright } else { colors.text };
            let bg = if is_selected { colors.selected_bg } else { colors.bg };
            let total_size: f64 = items.iter().map(|(_, s)| *s).sum();

            buf.set_string(area.x, y, &format!("{} {} {}", marker, icon, name), Style::default().fg(color).add_modifier(Modifier::BOLD).bg(bg));
            
            if *installed {
                buf.set_string(area.x + area.width - 15, y, "[Installed]", Style::default().fg(colors.success).bg(bg));
            } else {
                buf.set_string(area.x + area.width - 12, y, "[Not Found]", Style::default().fg(colors.text_dim).bg(bg));
            }
            y += 1;

            for (item_name, size) in items {
                let bar_width = (*size as f64 / total_size * 30.0) as usize;
                let bar = "█".repeat(bar_width);
                buf.set_string(area.x + 4, y, &format!("├── {} ", item_name), Style::default().fg(colors.text_dim).bg(bg));
                buf.set_string(area.x + 25, y, &bar, Style::default().fg(colors.warning).bg(bg));
                buf.set_string(area.x + 56, y, &format!("{:.1} GB", size), Style::default().fg(colors.warning).bg(bg));
                y += 1;
            }
            y += 1;
        }

        let help_y = area.y + area.height - 1;
        buf.set_string(area.x, help_y, "[↑↓] Navigate  [Space] Select  [C] Clean Selected  [A] Analyze All", Style::default().fg(colors.text_dim));
    }

    fn render_presets(&mut self, area: Rect, buf: &mut Buffer, colors: &ContentColors) {
        let title = Line::from(vec![Span::styled("◆ CLEANING PRESETS ◆", Style::default().fg(colors.accent).add_modifier(Modifier::BOLD))]);
        Paragraph::new(title).render(Rect::new(area.x, area.y, area.width, 1), buf);

        let presets = vec![
            (0, "🎯", "Conservative (Safe)", "Browser cache, thumbnails, trash, system temp", "~5-15 GB", true),
            (1, "⚖️", "Balanced (Recommended)", " + Old downloads (90d), dev caches, Docker prune", "~20-50 GB", true),
            (2, "🔥", "Aggressive (Deep Clean)", " + Flatpak unused, empty dirs, old logs, thumbnails", "~50-100 GB", false),
            (3, "💣", "Nuclear (Everything)", "⚠️ Remove ALL cleanable items including containers", "~100+ GB", false),
        ];

        let mut y = area.y + 2;
        let max_items = (area.height as usize - 4).min(presets.len());

        for (i, (idx, icon, name, desc, estimate, safe)) in presets.iter().take(max_items).enumerate() {
            let is_selected = self.list_state.selected() == Some(i);
            let marker = if is_selected { "▸" } else { " " };
            let color = if is_selected { colors.text_bright } else { colors.text };
            let bg = if is_selected { colors.selected_bg } else { colors.bg };
            let selected = self.selected_presets.contains(idx);

            buf.set_string(area.x, y, &format!("{} {}", marker, icon), Style::default().fg(if selected { colors.success } else { color }).add_modifier(Modifier::BOLD).bg(bg));
            buf.set_string(area.x + 4, y, name, Style::default().fg(color).add_modifier(Modifier::BOLD).bg(bg));
            buf.set_string(area.x + 30, y, &format!("[{}]", if selected { "SELECTED" } else { "-----" }), Style::default().fg(if selected { colors.success } else { colors.text_dim }).bg(bg));
            y += 1;

            buf.set_string(area.x + 4, y, desc, Style::default().fg(colors.text_dim).bg(bg));
            y += 1;

            buf.set_string(area.x + 4, y, &format!("Est. Space: {}", estimate), Style::default().fg(if *safe { colors.warning } else { colors.danger }).bg(bg));
            y += 2;

            let sep = "─".repeat((area.width - 4).min(60) as usize);
            buf.set_string(area.x + 2, y, &sep, Style::default().fg(colors.border).bg(colors.bg));
            y += 1;
        }

        let help_y = area.y + area.height - 1;
        buf.set_string(area.x, help_y, "[↑↓] Navigate  [Space] Toggle Preset  [T] Select All Safe  [Enter] Apply", Style::default().fg(colors.text_dim));
    }

    fn render_search(&mut self, area: Rect, buf: &mut Buffer, colors: &ContentColors) {
        let title = Line::from(vec![Span::styled("◆ SEARCH & FILTER ◆", Style::default().fg(colors.accent).add_modifier(Modifier::BOLD))]);
        Paragraph::new(title).render(Rect::new(area.x, area.y, area.width, 1), buf);

        let y = area.y + 2;
        buf.set_string(area.x, y, "Search: [", Style::default().fg(colors.text_dim));
        buf.set_string(area.x + 9, y, &format!("{:<30}", if self.search_query.is_empty() { "type here..." } else { &self.search_query }), Style::default().fg(if self.search_query.is_empty() { colors.text_dim } else { colors.text }).bg(Color::Rgb(40, 40, 60)));
        buf.set_string(area.x + 40, y, "]", Style::default().fg(colors.text_dim));

        buf.set_string(area.x, y + 2, "Filters:", Style::default().fg(colors.header).add_modifier(Modifier::BOLD));
        buf.set_string(area.x + 10, y + 2, "Size: ", Style::default().fg(colors.text));
        buf.set_string(area.x + 16, y + 2, &format!("{:>10}", self.filter_size_min.map(|s| format_size(s)).unwrap_or_else(|| "Any".to_string())), Style::default().fg(colors.warning));
        buf.set_string(area.x + 28, y + 2, "Age: ", Style::default().fg(colors.text));
        buf.set_string(area.x + 33, y + 2, &format!("{:>10}", self.filter_age_days.map(|d| format!("{}d", d)).unwrap_or_else(|| "Any".to_string())), Style::default().fg(colors.warning));

        buf.set_string(area.x, y + 4, "Results:", Style::default().fg(colors.header).add_modifier(Modifier::BOLD));

        let search_results = vec![
            ("large_video_final_v3.mp4", 4.2_f64, 2024, "~/Downloads"),
            ("old_video_project.mp4", 2.1_f64, 2023, "~/Videos"),
            ("archive_backup.tar.gz", 1.8_f64, 2023, "~/Documents"),
            ("tutorial_hd.mp4", 1.2_f64, 2023, "~/Downloads"),
            ("project_backup.zip", 0.8_f64, 2024, "~/Documents"),
        ];

        let mut ry = y + 5;
        for (i, (name, size, year, path)) in search_results.iter().enumerate().take((area.height as usize - 10).min(search_results.len())) {
            let is_selected = self.list_state.selected() == Some(i);
            let marker = if is_selected { "▸" } else { " " };
            let color = if is_selected { colors.text_bright } else { colors.text };
            let bg = if is_selected { colors.selected_bg } else { colors.bg };

            buf.set_string(area.x, ry, marker, Style::default().fg(colors.accent).bg(bg));
            buf.set_string(area.x + 2, ry, name, Style::default().fg(color).bg(bg));
            buf.set_string(area.x + 30, ry, &format!("{:.1} GB", size), Style::default().fg(colors.warning).bg(bg));
            buf.set_string(area.x + 40, ry, &format!("{}", year), Style::default().fg(colors.text_dim).bg(bg));
            buf.set_string(area.x + 46, ry, path, Style::default().fg(colors.text_dim).bg(bg));
            ry += 1;
        }

        let help_y = area.y + area.height - 1;
        buf.set_string(area.x, help_y, "[↑↓] Navigate  [Space] Select  [/] Type Search  [F] Filter  [Enter] Preview", Style::default().fg(colors.text_dim));
    }

    fn render_preview(&mut self, area: Rect, buf: &mut Buffer, colors: &ContentColors) {
        let title = Line::from(vec![Span::styled("◆ PREVIEW - ITEMS TO CLEAN ◆", Style::default().fg(colors.accent).add_modifier(Modifier::BOLD))]);
        Paragraph::new(title).render(Rect::new(area.x, area.y, area.width, 1), buf);

        let items = vec![
            ("Browser Caches", "~/.cache/*", 12.1_f64, 847, true),
            ("Old Downloads", "~/Downloads/*.mp4 (90d+)", 25.3_f64, 23, true),
            ("Thumbnails", "~/.cache/thumbnails/*", 2.8_f64, 2340, true),
            ("Docker Prune", "docker system prune -af", 15.2_f64, 18, false),
            ("Flatpak Unused", "flatpak uninstall --unused", 4.1_f64, 8, false),
            ("Trash", "~/.local/share/Trash/*", 1.8_f64, 156, true),
        ];

        let total_size: f64 = items.iter().map(|(_, _, s, _, _)| *s).sum();
        let total_items: usize = items.iter().map(|(_, _, _, c, _)| *c).sum();

        buf.set_string(area.x, area.y + 2, &format!("Total: {} items ( {:.1} GB )", total_items, total_size), Style::default().fg(colors.warning).add_modifier(Modifier::BOLD));

        let mut y = area.y + 4;
        let max_items = (area.height as usize - 6).min(items.len());

        for (i, (name, path, size, count, safe)) in items.iter().enumerate().take(max_items) {
            let is_selected = self.list_state.selected() == Some(i);
            let marker = if is_selected { "▸" } else { " " };
            let color = if is_selected { colors.text_bright } else { colors.text };
            let bg = if is_selected { colors.selected_bg } else { colors.bg };
            let safe_icon = if *safe { "[*]" } else { "[!]" };

            buf.set_string(area.x, y, marker, Style::default().fg(colors.accent).bg(bg));
            buf.set_string(area.x + 2, y, name, Style::default().fg(color).add_modifier(Modifier::BOLD).bg(bg));
            buf.set_string(area.x + 25, y, safe_icon, Style::default().fg(if *safe { colors.success } else { colors.danger }).bg(bg));
            buf.set_string(area.x + 30, y, path, Style::default().fg(colors.text_dim).bg(bg));
            buf.set_string(area.x + area.width - 20, y, &format!("{} ({})", format_size(*size as u64), count), Style::default().fg(colors.warning).bg(bg));
            y += 1;
        }

        let help_y = area.y + area.height - 1;
        buf.set_string(area.x, help_y, "[↑↓] Navigate  [Space] Toggle  [T] Select All  [C] Start Cleaning", Style::default().fg(colors.text_dim));
    }

    fn render_confirm(&mut self, area: Rect, buf: &mut Buffer, colors: &ContentColors) {
        let title = Line::from(vec![Span::styled("⚠ FINAL CONFIRMATION ⚠", Style::default().fg(colors.danger).add_modifier(Modifier::BOLD))]);
        Paragraph::new(title).render(Rect::new(area.x, area.y, area.width, 1), buf);

        let y = area.y + 3;
        
        buf.set_string(area.x, y, "╔══════════════════════════════════════════════════════════════╗", Style::default().fg(colors.danger));
        buf.set_string(area.x, y + 1, "║  WARNING: The following items will be permanently deleted:   ║", Style::default().fg(colors.danger));
        buf.set_string(area.x, y + 2, "╠══════════════════════════════════════════════════════════════╣", Style::default().fg(colors.danger));

        let items = [
            "Browser caches (~/.cache/*)",
            "90+ day old downloads",
            "Thumbnail cache",
            "Docker unused images/containers",
            "Trash contents",
        ];

        for (i, item) in items.iter().enumerate() {
            buf.set_string(area.x, y + 3 + i as u16, &format!("║  ◉ {}", item), Style::default().fg(colors.text));
        }

        buf.set_string(area.x, y + 8, "╠══════════════════════════════════════════════════════════════╣", Style::default().fg(colors.danger));
        buf.set_string(area.x, y + 9, "║  Total space to be freed: ~62.5 GB                          ║", Style::default().fg(colors.warning).add_modifier(Modifier::BOLD));
        buf.set_string(area.x, y + 10, "║  Items to be deleted: 3,391                                ║", Style::default().fg(colors.text));
        buf.set_string(area.x, y + 11, "║  Backup created: Yes (can be undone)                        ║", Style::default().fg(colors.success));
        buf.set_string(area.x, y + 12, "╚══════════════════════════════════════════════════════════════╝", Style::default().fg(colors.danger));

        let help_y = area.y + area.height - 1;
        buf.set_string(area.x, help_y, "[Y] CONFIRM CLEAN  [N] Cancel  [←] Go Back", Style::default().fg(colors.danger));
    }

    fn render_done(&mut self, area: Rect, buf: &mut Buffer, colors: &ContentColors) {
        let title = Line::from(vec![Span::styled("✅ CLEANING COMPLETE!", Style::default().fg(colors.success).add_modifier(Modifier::BOLD))]);
        Paragraph::new(title).render(Rect::new(area.x, area.y, area.width, 1), buf);

        let y = area.y + 3;

        buf.set_string(area.x, y, "┌────────────────────────────────────────────────────────────────┐", Style::default().fg(colors.success));
        buf.set_string(area.x, y + 1, "│              BEFORE          AFTER           SAVED        │", Style::default().fg(colors.header));
        buf.set_string(area.x, y + 2, "│              250.0 GB    →    187.5 GB    =    62.5 GB     │", Style::default().fg(colors.warning).add_modifier(Modifier::BOLD));
        buf.set_string(area.x, y + 3, "└────────────────────────────────────────────────────────────────┘", Style::default().fg(colors.success));

        buf.set_string(area.x, y + 5, " BREAKDOWN:", Style::default().fg(colors.header).add_modifier(Modifier::BOLD));

        let breakdown = [
            ("Downloads (old files)", 25.3_f64),
            ("Docker system prune", 15.2_f64),
            ("Browser caches", 12.1_f64),
            ("Package caches", 8.4_f64),
            ("Thumbnails", 2.8_f64),
            ("Other cleanups", 3.2_f64),
        ];

        let mut bar_y = y + 6;
        for (name, size) in &breakdown {
            let bar_len = (*size / 62.5_f64 * 40.0) as usize;
            let bar = "█".repeat(bar_len);
            buf.set_string(area.x, bar_y, &format!("  {}:", name), Style::default().fg(colors.text));
            buf.set_string(area.x + 25, bar_y, &bar, Style::default().fg(colors.accent));
            buf.set_string(area.x + 66, bar_y, &format!("{:.1} GB", size), Style::default().fg(colors.warning));
            bar_y += 1;
        }

        buf.set_string(area.x, bar_y + 2, " ✓ All items backed up to ~/.local/share/windebloat/backups/", Style::default().fg(colors.success));
        buf.set_string(area.x, bar_y + 3, " ✓ Full report saved", Style::default().fg(colors.success));

        let help_y = area.y + area.height - 1;
        buf.set_string(area.x, help_y, "[Enter] Done  [U] Undo Changes  [R] Run Again", Style::default().fg(colors.text_dim));
    }

    fn render_auto_clean(&mut self, area: Rect, buf: &mut Buffer, focused: bool) {
        let colors = ContentColors::dark();
        let border_color = if focused { colors.accent } else { Color::Rgb(100, 220, 180) };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(colors.bg));
        Widget::render(block, area, buf);

        let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
        let mut y = inner.y;

        let title = "\u{1f680} MEGA ULTRA AUTO CLEAN";
        buf.set_string(inner.x, y, title, Style::default().fg(colors.accent).add_modifier(Modifier::BOLD));
        y += 2;

        let lines = [
            "This option scans, analyzes and cleans 24+ categories in 30 stages.",
            "",
            "Features:",
            "  \u{1f4ca} 30-stage deep scan (packages, system, apps, privacy, ...)",
            "  \u{1f4c8} Future disk usage prediction and growth analysis",
            "  \u{1f4be} Disk benchmark (before/after comparison)",
            "  \u{1f3ae} Games, IDE, messaging, email caches included",
            "  \u{1f9f9} Automatic backup and restore support",
            "  \u{1f4b0} Detailed category report (JSON/MD/TXT)",
            "",
            "Usage:",
            "  [X] Open this panel  [Enter] Start  [Esc] Cancel",
            "",
            "Profiles:",
            "  [1] Safe (default) - Only safe items",
            "  [2] Balanced - Recommended settings",
            "  [3] Aggressive - All cleanable items",
        ];

        for line in &lines {
            buf.set_string(inner.x, y, line, Style::default().fg(if line.starts_with("  \\u") { colors.text_dim } else { colors.text }));
            y += 1;
        }
    }
}

pub struct ContentColors {
    pub bg: Color,
    pub border: Color,
    pub header: Color,
    pub text: Color,
    pub text_dim: Color,
    pub text_bright: Color,
    pub accent: Color,
    pub warning: Color,
    pub danger: Color,
    pub success: Color,
    pub selected_bg: Color,
    pub selected: Color,
}

impl ContentColors {
    pub fn dark() -> ContentColors {
        ContentColors {
            bg: Color::Rgb(30, 30, 46),
            border: Color::Rgb(60, 60, 90),
            header: Color::Rgb(100, 180, 220),
            text: Color::Rgb(210, 210, 230),
            text_dim: Color::Rgb(100, 100, 120),
            text_bright: Color::Rgb(255, 255, 255),
            accent: Color::Rgb(100, 220, 180),
            warning: Color::Rgb(255, 200, 80),
            danger: Color::Rgb(255, 100, 100),
            success: Color::Rgb(100, 220, 150),
            selected_bg: Color::Rgb(50, 50, 80),
            selected: Color::Rgb(255, 220, 120),
        }
    }
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