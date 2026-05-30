use sysinfo::{System, Disks};

pub fn collect_system_info() -> (u64, u64, f32) {
    let mut sys = System::new_all();
    sys.refresh_memory();
    sys.refresh_cpu_usage();
    let used = sys.used_memory();
    let total = sys.total_memory();
    let cpu = sys.global_cpu_usage();
    (used, total, cpu)
}

pub fn collect_disk_info() -> Vec<(String, u64, u64, u64, f32)> {
    let disks = Disks::new_with_refreshed_list();
    let mut result = Vec::new();
    for disk in disks.list() {
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total.saturating_sub(available);
        let pct = if total > 0 { used as f32 / total as f32 * 100.0 } else { 0.0 };
        result.push((
            disk.mount_point().to_string_lossy().to_string(),
            total,
            used,
            available,
            pct,
        ));
    }
    result
}

pub fn collect_package_count() -> Option<usize> {
    for cmd in &["dpkg", "pacman", "rpm", "apk"] {
        if let Ok(output) = std::process::Command::new(cmd)
            .args(if *cmd == "dpkg" { &["-l"] as &[_] } else if *cmd == "pacman" { &["-Qq"] } else if *cmd == "rpm" { &["-qa"] } else { &["info"] })
            .output()
        {
            let count = String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter(|l| if *cmd == "dpkg" { l.starts_with("ii") } else { !l.is_empty() })
                .count();
            if count > 0 {
                return Some(count);
            }
        }
    }
    None
}
