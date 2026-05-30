pub fn mean(values: &[f64]) -> f64 {
    if values.is_empty() { return 0.0; }
    values.iter().sum::<f64>() / values.len() as f64
}

pub fn median(values: &mut [f64]) -> f64 {
    if values.is_empty() { return 0.0; }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let len = values.len();
    if len % 2 == 0 {
        (values[len/2 - 1] + values[len/2]) / 2.0
    } else {
        values[len/2]
    }
}

pub fn std_dev(values: &[f64]) -> f64 {
    if values.is_empty() { return 0.0; }
    let m = mean(values);
    let variance = values.iter().map(|v| (v - m).powi(2)).sum::<f64>() / values.len() as f64;
    variance.sqrt()
}

pub fn percentile(values: &[f64], pct: f64) -> f64 {
    if values.is_empty() { return 0.0; }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((sorted.len() as f64 * pct / 100.0).ceil() as usize).min(sorted.len() - 1);
    sorted[idx]
}
