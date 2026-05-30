use ratatui::prelude::*;

pub struct ProgressWidget {
    pub fraction: f32,
    pub width: u16,
    pub label: String,
    pub color: Color,
}

impl ProgressWidget {
    pub fn new(fraction: f32, width: u16) -> Self {
        ProgressWidget {
            fraction: fraction.clamp(0.0, 1.0),
            width,
            label: String::new(),
            color: Color::Rgb(100, 220, 180),
        }
    }

    pub fn with_label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn render(&self, buf: &mut Buffer, x: u16, y: u16) {
        let filled = (self.fraction * self.width as f32) as usize;
        let empty = (self.width as usize).saturating_sub(filled);

        let fill_str: String = std::iter::repeat('█').take(filled).collect();
        let empty_str: String = std::iter::repeat('░').take(empty).collect();

        buf.set_string(x, y, &fill_str, Style::default().fg(self.color));
        buf.set_string(x + filled as u16, y, &empty_str, Style::default().fg(Color::Rgb(60, 60, 90)));

        if !self.label.is_empty() {
            let pct = (self.fraction * 100.0) as u64;
            let label = format!(" {}% ", pct);
            let label_x = x + (self.width / 2).saturating_sub(label.len() as u16 / 2);
            buf.set_string(label_x, y, &label, Style::default().fg(Color::Rgb(30, 30, 46)).bg(self.color));
        }
    }
}

pub struct TableWidget {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub widths: Vec<u16>,
    pub selected: Option<usize>,
    pub highlight_color: Color,
}

impl TableWidget {
    pub fn new(headers: Vec<String>) -> Self {
        TableWidget {
            headers,
            rows: Vec::new(),
            widths: Vec::new(),
            selected: None,
            highlight_color: Color::Rgb(50, 50, 80),
        }
    }

    pub fn add_row(&mut self, row: Vec<String>) {
        if self.widths.is_empty() {
            self.widths = row.iter().map(|c| c.len() as u16).collect();
        } else {
            for (i, w) in row.iter().enumerate() {
                if i < self.widths.len() {
                    self.widths[i] = self.widths[i].max(w.len() as u16);
                }
            }
        }
        self.rows.push(row);
    }

    pub fn render_header(&self, buf: &mut Buffer, x: u16, y: u16, color: Color) {
        let mut cx = x;
        for (i, h) in self.headers.iter().enumerate() {
            let w = self.widths.get(i).copied().unwrap_or(10);
            buf.set_string(cx, y, &format!(" {:<w$} ", h, w = w as usize), Style::default().fg(color).add_modifier(Modifier::BOLD));
            cx += w + 2;
        }
    }

    pub fn render_row(&self, buf: &mut Buffer, x: u16, y: u16, index: usize) {
        if index >= self.rows.len() { return; }
        let row = &self.rows[index];
        let is_selected = self.selected == Some(index);
        let bg = if is_selected { self.highlight_color } else { Color::Reset };
        let fg = if is_selected { Color::Rgb(255, 220, 120) } else { Color::Rgb(210, 210, 230) };

        let marker = if is_selected { "▸" } else { " " };
        buf.set_string(x, y, marker, Style::default().fg(fg).bg(bg));

        let mut cx = x + 1;
        for (i, cell) in row.iter().enumerate() {
            let w = self.widths.get(i).copied().unwrap_or(10);
            buf.set_string(cx, y, &format!(" {:<w$} ", cell, w = w as usize), Style::default().fg(fg).bg(bg));
            cx += w + 2;
        }
    }
}
