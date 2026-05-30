use chrono::Utc;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub struct Logger {
    log_path: PathBuf,
    log_file: Option<fs::File>,
}

impl Logger {
    pub fn new() -> Self {
        let base = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("~/.local/share"))
            .join("windebloat")
            .join("logs");

        let _ = fs::create_dir_all(&base);

        let log_path = base.join(format!("windebloat-{}.log", Utc::now().format("%Y%m%d")));
        let log_file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .ok();

        Logger { log_path, log_file }
    }

    pub fn log(&mut self, message: &str) {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let line = format!("[{}] {}\n", timestamp, message);

        if let Some(ref mut file) = self.log_file {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }

        log::info!("{}", message);
    }

    pub fn action(&mut self, action: &str, item: &str, size: u64) {
        self.log(&format!("ACTION: {} | {} | {} bytes", action, item, size));
    }

    pub fn get_logs(&self, count: usize) -> Vec<String> {
        let mut logs = Vec::new();
        if let Ok(content) = fs::read_to_string(&self.log_path) {
            logs = content.lines().rev().take(count).map(String::from).collect();
        }
        logs
    }

    pub fn log_path(&self) -> &PathBuf {
        &self.log_path
    }
}
