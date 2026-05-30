use crate::core::auto_clean::{AutoCleanReport, CategoryReport, CleanError, DiskComparison, ReportSummary, Suggestion};
use crate::utils::error::Result;

pub enum ReportFormat {
    Text,
    Json,
    Markdown,
}

impl AutoCleanReport {
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out.push_str("════════════════════════════════════════════════════\n");
        out.push_str("  ✅ ULTRA MEGA AUTO CLEAN — COMPLETE!\n");
        out.push_str("════════════════════════════════════════════════════\n\n");

        out.push_str(&format!("  Date: {}\n", &self.timestamp[..19]));
        out.push_str(&format!("  Duration: {:.1} seconds\n", self.duration_secs));
        out.push_str(&format!("  Scanned: {} items\n", self.summary.total_scanned));
        out.push_str(&format!("  Cleaned: {} items\n", self.summary.total_cleaned));
        out.push_str(&format!("  Space Recovered: {} / {}\n",
            format_size(self.summary.total_cleaned_size),
            format_size(self.summary.total_cleanable_size)));
        out.push_str(&format!("  Hata: {}\n", self.summary.total_failed));
        if let Some(ref bid) = self.backup_id {
            out.push_str(&format!("  Backup ID: {}\n", bid));
        }
        out.push('\n');

        out.push_str("  ── BY CATEGORY ──\n");
        for cat in &self.categories {
            let pct = if cat.found_items > 0 { cat.cleaned_items as f32 / cat.found_items as f32 * 100.0 } else { 0.0 };
            out.push_str(&format!("  {} {}: {}/{} (%{:.0}) {}\n",
                cat.icon, cat.name, cat.cleaned_items, cat.found_items, pct, format_size(cat.cleaned_size)));
        }
        out.push('\n');

        out.push_str("  ── DISK COMPARISON ──\n");
        for d in &self.before_after {
            out.push_str(&format!("  {}: {:.0}% → {:.0}% (gain: {})\n",
                d.mount, d.before_pct, d.after_pct, format_size(d.freed)));
        }
        out.push('\n');

        if !self.errors.is_empty() {
            out.push_str("  ── HATALAR ──\n");
            for e in &self.errors {
                out.push_str(&format!("  ❌ {}: {}\n", e.item_path, e.message));
            }
            out.push('\n');
        }

        if !self.suggestions.is_empty() {
            out.push_str("  ── SUGGESTIONS ──\n");
            for s in &self.suggestions {
                out.push_str(&format!("  💡 {}: {} ({})\n", s.category_id, s.message, format_size(s.impact_size)));
            }
            out.push('\n');
        }

        out
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str("# ✅ Ultra Mega Auto Clean Report\n\n");
        out.push_str(&format!("- **Tarih:** {}\n", &self.timestamp[..19]));
        out.push_str(&format!("- **Duration:** {:.1}s\n", self.duration_secs));
        out.push_str(&format!("- **Cleaned:** {} items\n", self.summary.total_cleaned));
        out.push_str(&format!("- **Space Recovered:** {}\n", format_size(self.summary.total_cleaned_size)));
        out.push_str(&format!("- **Hata:** {}\n\n", self.summary.total_failed));

        out.push_str("## Categories\n\n");
        out.push_str("| Category | Items | Size | Success |\n");
        out.push_str("|---|---|---|---|\n");
        for cat in &self.categories {
            let pct = if cat.found_items > 0 { cat.cleaned_items as f32 / cat.found_items as f32 * 100.0 } else { 0.0 };
            out.push_str(&format!("| {} {} | {}/{} | {} | %{:.0} |\n",
                cat.icon, cat.name, cat.cleaned_items, cat.found_items,
                format_size(cat.cleaned_size), pct));
        }

        out.push_str("\n## Disk Comparison\n\n");
        out.push_str("| Mount Point | Before | After | Gain |\n");
        out.push_str("|---|---|---|---|\n");
        for d in &self.before_after {
            out.push_str(&format!("| {} | %{:.0} | %{:.0} | {} |\n",
                d.mount, d.before_pct, d.after_pct, format_size(d.freed)));
        }

        out
    }
}

pub fn save_report(report: &AutoCleanReport, path: &str, format: ReportFormat) -> Result<()> {
    let content = match format {
        ReportFormat::Text => report.to_text(),
        ReportFormat::Json => report.to_json(),
        ReportFormat::Markdown => report.to_markdown(),
    };
    std::fs::write(path, &content)?;
    Ok(())
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
