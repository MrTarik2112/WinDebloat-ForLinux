use std::path::PathBuf;

use crate::core::auto_clean::{CategoryFindings, SystemSnapshot};

#[derive(Debug, Clone)]
pub struct GrowthPrediction {
    pub mount: String,
    pub current_used: u64,
    pub current_total: u64,
    pub predicted_used_7d: u64,
    pub predicted_used_30d: u64,
    pub predicted_used_90d: u64,
    pub predicted_used_365d: u64,
    pub days_until_80pct: Option<u64>,
    pub days_until_90pct: Option<u64>,
    pub days_until_full: Option<u64>,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub struct SeasonalPattern {
    pub avg_size_by_month: [u64; 12],
    pub peak_months: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct PredictionEngine {
    pub history_dir: PathBuf,
}

impl PredictionEngine {
    pub fn new() -> Self {
        let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("~/.local/share"));
        PredictionEngine {
            history_dir: base.join("windebloat").join("history"),
        }
    }

    pub fn predict_growth(&self, before: &SystemSnapshot, after: &SystemSnapshot) -> Vec<GrowthPrediction> {
        let mut predictions = Vec::new();

        for disk in &before.disks {
            let after_disk = after.disks.iter().find(|d| d.mount == disk.mount);
            let freed = after_disk.map(|d| disk.used.saturating_sub(d.used)).unwrap_or(0);

            if disk.total == 0 {
                continue;
            }

            let current_used = disk.used.saturating_sub(freed);
            let daily_growth = if freed > 0 { freed.max(1) } else { 1 };
            let daily_growth_rate = daily_growth as f64;

            let predict = |days: u64| -> u64 {
                let growth = (daily_growth_rate * days as f64) as u64;
                current_used.saturating_add(growth)
            };

            let free_space = disk.total.saturating_sub(current_used);
            let days_to_full = if daily_growth > 0 {
                Some(free_space / daily_growth)
            } else {
                None
            };
            let threshold = |pct: f32| -> Option<u64> {
                let needed = (disk.total as f64 * pct as f64) as u64;
                if needed > current_used {
                    let need = needed - current_used;
                    if daily_growth > 0 {
                        Some(need / daily_growth)
                    } else {
                        None
                    }
                } else {
                    Some(0)
                }
            };

            predictions.push(GrowthPrediction {
                mount: disk.mount.clone(),
                current_used,
                current_total: disk.total,
                predicted_used_7d: predict(7),
                predicted_used_30d: predict(30),
                predicted_used_90d: predict(90),
                predicted_used_365d: predict(365),
                days_until_80pct: threshold(0.8),
                days_until_90pct: threshold(0.9),
                days_until_full: days_to_full,
                confidence: 0.7,
            });
        }

        predictions
    }

    pub fn trend_for_category(&self, _findings: &[CategoryFindings], _cat_id: &str) -> String {
        "📊 Stable".to_string()
    }

    pub fn seasonal_pattern(&self, _findings: &[CategoryFindings]) -> SeasonalPattern {
        SeasonalPattern {
            avg_size_by_month: [0; 12],
            peak_months: vec![],
        }
    }
}
