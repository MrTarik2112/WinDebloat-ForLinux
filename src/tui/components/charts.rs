use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render_bar_chart(
    buf: &mut Buffer,
    area: Rect,
    title: &str,
    data: &[(&str, u64, Color)],
    max_value: u64,
) {
    let colors = crate::tui::ui::ThemeColors::dark();
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.border));
    Widget::render(block, area, buf);

    let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
    let max_bar_width = (inner.width as usize).saturating_sub(20);
    let max_val = max_value.max(1);

    for (i, (label, value, color)) in data.iter().enumerate() {
        let y = inner.y + i as u16;
        if y >= inner.y + inner.height { break; }

        let bar_len = ((*value as f64 / max_val as f64) * max_bar_width as f64) as usize;
        let bar: String = std::iter::repeat('█').take(bar_len).collect();
        let space: String = std::iter::repeat('░').take(max_bar_width.saturating_sub(bar_len)).collect();

        buf.set_string(inner.x, y, format!(" {}", label), Style::default().fg(*color));
        buf.set_string(inner.x + 14, y, &bar, Style::default().fg(*color));
        buf.set_string(inner.x + 14 + bar_len.min(max_bar_width) as u16, y, &space, Style::default().fg(colors.border));
        let val_str = format_size(*value);
        buf.set_string(inner.x + 14 + max_bar_width as u16 + 1, y, &val_str, Style::default().fg(colors.warning));
    }
}

pub fn render_pie_chart(
    buf: &mut Buffer,
    area: Rect,
    title: &str,
    data: &[(&str, f32, Color)],
) {
    let colors = crate::tui::ui::ThemeColors::dark();
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.border));
    Widget::render(block, area, buf);

    let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
    let total: f32 = data.iter().map(|(_, v, _)| *v).sum::<f32>().max(1.0);

    for (i, (label, value, color)) in data.iter().enumerate() {
        let y = inner.y + i as u16;
        if y >= inner.y + inner.height { break; }

        let pct = *value / total * 100.0;
        let bar_len = (*value / total * (inner.width.saturating_sub(15)) as f32) as usize;
        let bar: String = std::iter::repeat('█').take(bar_len).collect();

        buf.set_string(inner.x, y, format!(" {} ", label), Style::default().fg(*color));
        buf.set_string(inner.x + 12, y, &bar, Style::default().fg(*color));
        buf.set_string(inner.x + 12 + bar_len as u16, y, &format!(" {:.0}%", pct), Style::default().fg(colors.text_dim));
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
    if unit_idx == 0 {
        format!("{} {}", size as u64, UNITS[unit_idx])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}
