pub fn format_bytes(bytes: u64) -> String {
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

pub fn parse_size(size_str: &str) -> Option<u64> {
    let size_str = size_str.trim().to_uppercase();
    let (num_str, unit) = if let Some(idx) = size_str.find(|c: char| !c.is_ascii_digit() && c != '.') {
        (size_str[..idx].to_string(), size_str[idx..].to_string())
    } else {
        (size_str.clone(), "".to_string())
    };

    let num: f64 = num_str.parse().ok()?;
    let multiplier: u64 = match unit.as_str() {
        "B" => 1,
        "K" | "KB" => 1024,
        "M" | "MB" => 1024 * 1024,
        "G" | "GB" => 1024 * 1024 * 1024,
        "T" | "TB" => 1024 * 1024 * 1024 * 1024,
        "" => 1,
        _ => return None,
    };

    Some((num * multiplier as f64) as u64)
}

pub fn format_count(count: usize) -> String {
    if count > 1000000 {
        format!("{:.1}M", count as f64 / 1000000.0)
    } else if count > 1000 {
        format!("{:.1}K", count as f64 / 1000.0)
    } else {
        count.to_string()
    }
}

pub fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}
