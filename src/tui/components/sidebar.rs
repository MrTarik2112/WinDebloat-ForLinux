use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, ListState},
};

use crate::core::scanner::Category;

pub struct Sidebar {
    pub state: ListState,
    pub items: Vec<SidebarItem>,
    pub width: u16,
    pub show_wizard: bool,
    pub render_area: Option<Rect>,
    pub hovered_idx: Option<usize>,
}

pub struct SidebarItem {
    pub category: Category,
    pub label: String,
    pub count: usize,
    pub icon_filled: String,
    pub icon_empty: String,
    pub is_wizard: bool,
}

pub struct SidebarColors {
    pub bg: Color,
    pub border: Color,
    pub text: Color,
    pub text_dim: Color,
    pub selected: Color,
    pub selected_bg: Color,
    pub count_bg: Color,
    pub accent: Color,
}

impl SidebarColors {
    pub fn dark() -> Self {
        SidebarColors {
            bg: Color::Rgb(25, 25, 40),
            border: Color::Rgb(50, 50, 80),
            text: Color::Rgb(180, 180, 200),
            text_dim: Color::Rgb(80, 80, 100),
            selected: Color::Rgb(100, 220, 180),
            selected_bg: Color::Rgb(40, 40, 65),
            count_bg: Color::Rgb(60, 100, 140),
            accent: Color::Rgb(100, 180, 220),
        }
    }
}

impl Sidebar {
    pub fn new(width: u16) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        let items: Vec<SidebarItem> = vec![
            SidebarItem {
                category: Category::Packages,
                label: "Packages".into(),
                count: 0,
                icon_filled: "📦".into(),
                icon_empty: "📦".into(),
                is_wizard: false,
            },
            SidebarItem {
                category: Category::System,
                label: "System".into(),
                count: 0,
                icon_filled: "🧹".into(),
                icon_empty: "🧹".into(),
                is_wizard: false,
            },
            SidebarItem {
                category: Category::Apps,
                label: "Apps".into(),
                count: 0,
                icon_filled: "💻".into(),
                icon_empty: "💻".into(),
                is_wizard: false,
            },
            SidebarItem {
                category: Category::Privacy,
                label: "Privacy".into(),
                count: 0,
                icon_filled: "🔒".into(),
                icon_empty: "🔒".into(),
                is_wizard: false,
            },
            SidebarItem {
                category: Category::Services,
                label: "Services".into(),
                count: 0,
                icon_filled: "⚙".into(),
                icon_empty: "⚙".into(),
                is_wizard: false,
            },
            SidebarItem {
                category: Category::Duplicates,
                label: "Duplicates".into(),
                count: 0,
                icon_filled: "📋".into(),
                icon_empty: "📋".into(),
                is_wizard: false,
            },
            SidebarItem {
                category: Category::Containers,
                label: "Containers".into(),
                count: 0,
                icon_filled: "📦".into(),
                icon_empty: "📦".into(),
                is_wizard: false,
            },
            SidebarItem {
                category: Category::Disk,
                label: "Disk Usage".into(),
                count: 0,
                icon_filled: "💾".into(),
                icon_empty: "💾".into(),
                is_wizard: false,
            },
            SidebarItem {
                category: Category::BrokenLinks,
                label: "Broken Links".into(),
                count: 0,
                icon_filled: "🔗".into(),
                icon_empty: "🔗".into(),
                is_wizard: false,
            },
            SidebarItem {
                category: Category::EmptyFiles,
                label: "Empty Files".into(),
                count: 0,
                icon_filled: "📄".into(),
                icon_empty: "📄".into(),
                is_wizard: false,
            },
            SidebarItem {
                category: Category::OldDownloads,
                label: "Old Downloads".into(),
                count: 0,
                icon_filled: "📥".into(),
                icon_empty: "📥".into(),
                is_wizard: false,
            },
            SidebarItem {
                category: Category::OldLogs,
                label: "Old Logs".into(),
                count: 0,
                icon_filled: "📋".into(),
                icon_empty: "📋".into(),
                is_wizard: false,
            },
            SidebarItem {
                category: Category::Fonts,
                label: "Fonts".into(),
                count: 0,
                icon_filled: "🔤".into(),
                icon_empty: "🔤".into(),
                is_wizard: false,
            },
            SidebarItem {
                category: Category::Kernel,
                label: "Kernel".into(),
                count: 0,
                icon_filled: "⚙️".into(),
                icon_empty: "⚙️".into(),
                is_wizard: false,
            },
            SidebarItem {
                category: Category::DevTools,
                label: "Dev Tools".into(),
                count: 0,
                icon_filled: "💡".into(),
                icon_empty: "💡".into(),
                is_wizard: false,
            },
            SidebarItem {
                category: Category::Wizard,
                label: "Disk Wizard".into(),
                count: 0,
                icon_filled: "✨".into(),
                icon_empty: "✨".into(),
                is_wizard: true,
            },
            SidebarItem {
                category: Category::AutoClean,
                label: "M.Auto Clean".into(),
                count: 0,
                icon_filled: "🚀".into(),
                icon_empty: "🚀".into(),
                is_wizard: true,
            },
        ];

        Sidebar {
            state: ListState::default().with_selected(Some(0)),
            items,
            width,
            show_wizard: true,
            render_area: None,
            hovered_idx: None,
        }
    }

    pub fn hit_test(&self, x: u16, y: u16) -> Option<usize> {
        let area = self.render_area?;
        let list_x = area.x + 1;
        let list_y = area.y + 2;
        let list_height = area.height - 3;

        if x < list_x || x >= list_x + area.width - 2 {
            return None;
        }
        if y < list_y || y >= list_y + list_height {
            return None;
        }

        let index = (y - list_y) as usize;
        if index < self.items.len() {
            Some(index)
        } else {
            None
        }
    }

    pub fn select(&mut self, index: usize) {
        self.state.select(Some(index.min(self.items.len().saturating_sub(1))));
    }

    pub fn selected(&self) -> Option<Category> {
        self.state
            .selected()
            .and_then(|i| self.items.get(i))
            .map(|item| item.category)
    }

    pub fn set_count(&mut self, category: Category, count: usize) {
        if let Some(item) = self.items.iter_mut().find(|i| i.category == category) {
            item.count = count;
        }
    }

    pub fn next(&mut self) {
        let i = self
            .state
            .selected()
            .map(|i| (i + 1) % self.items.len())
            .unwrap_or(0);
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = self.state.selected().map(|i| {
            if i == 0 {
                self.items.len() - 1
            } else {
                i - 1
            }
        }).unwrap_or(0);
        self.state.select(Some(i));
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, _config: &crate::config::schema::UiConfig, focused: bool) {
        self.render_area = Some(area);
        let colors = SidebarColors::dark();

        let border_color = if focused { colors.accent } else { colors.border };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(colors.bg));
        Widget::render(block, area, buf);

        let title = Line::from(vec![
            Span::styled("◈ CATEGORIES", Style::default()
                .fg(colors.text_dim)
                .add_modifier(Modifier::BOLD)),
        ]);
        
        Paragraph::new(title)
            .style(Style::default().bg(colors.bg))
            .render(Rect::new(area.x + 1, area.y, area.width - 2, 1), buf);

        let list_area = Rect::new(area.x + 1, area.y + 2, area.width - 2, area.height - 3);

        for (i, item) in self.items.iter().enumerate() {
            let y = list_area.y + i as u16;
            if y >= list_area.y + list_area.height {
                break;
            }

            let is_selected = self.state.selected() == Some(i);
            let is_hovered = self.hovered_idx == Some(i);
            
            let (fg_color, bg_color) = if is_selected {
                (colors.selected, colors.selected_bg)
            } else if is_hovered {
                (colors.accent, Color::Rgb(40, 40, 60))
            } else if item.is_wizard {
                (Color::Rgb(255, 200, 80), colors.bg)
            } else {
                (colors.text, colors.bg)
            };

            let marker = if is_selected { "▸" } else { " " };
            let icon = if is_selected { &item.icon_filled } else { &item.icon_empty };
            
            buf.set_string(list_area.x, y, &format!(" {} {} ", marker, icon), Style::default()
                .fg(fg_color)
                .bg(bg_color));

            buf.set_string(list_area.x + 5, y, &item.label, Style::default()
                .fg(fg_color)
                .bg(bg_color));

            if item.count > 0 {
                let count_str = format!("[{}]", item.count);
                let count_x = list_area.x + list_area.width.saturating_sub(count_str.len() as u16 + 1);
                buf.set_string(count_x, y, &count_str, Style::default()
                    .fg(Color::Rgb(30, 30, 46))
                    .bg(colors.count_bg)
                    .add_modifier(Modifier::BOLD));
            }

            if item.is_wizard {
                let badge = "[W]";
                let badge_x = list_area.x + list_area.width.saturating_sub(badge.len() as u16 + 1);
                buf.set_string(badge_x, y, badge, Style::default()
                    .fg(Color::Rgb(30, 30, 46))
                    .bg(Color::Rgb(255, 200, 80))
                    .add_modifier(Modifier::BOLD));
            }
        }
    }
}