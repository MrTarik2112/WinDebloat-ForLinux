use std::path::PathBuf;
use crate::utils::error::Result;

pub struct Scheduler {
    pub cron_path: PathBuf,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            cron_path: PathBuf::from("/tmp/windebloat_cron"),
        }
    }

    pub fn install_cron(&self, expression: &str, command: &str) -> Result<()> {
        let cron_line = format!("{} {}", expression, command);
        let existing = std::fs::read_to_string(&self.cron_path).unwrap_or_default();
        let mut lines: Vec<String> = existing.lines()
            .filter(|l| !l.contains(command))
            .map(|l| l.to_string())
            .collect();
        lines.push(cron_line);
        std::fs::write(&self.cron_path, lines.join("\n") + "\n")?;
        Ok(())
    }

    pub fn remove_cron(&self, command: &str) -> Result<()> {
        let existing = std::fs::read_to_string(&self.cron_path).unwrap_or_default();
        let lines: Vec<String> = existing.lines()
            .filter(|l| !l.contains(command))
            .map(|l| l.to_string())
            .collect();
        std::fs::write(&self.cron_path, lines.join("\n") + "\n")?;
        Ok(())
    }

    pub fn is_installed(&self, command: &str) -> bool {
        if let Ok(content) = std::fs::read_to_string(&self.cron_path) {
            content.contains(command)
        } else {
            false
        }
    }
}
