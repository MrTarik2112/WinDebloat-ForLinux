use super::backup::Backup;
use super::scanner::CleanItem;
use crate::utils::error::Result;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;

#[derive(Debug, Clone)]
pub enum CleanProgress {
    File(String),
    Done(usize),
    Error(String),
    Finished,
}

pub struct Cleaner {
    backup: Backup,
}

impl Cleaner {
    pub fn new(backup: Backup) -> Self {
        Cleaner { backup }
    }

    pub fn execute(
        &self,
        items: &[CleanItem],
        sender: mpsc::Sender<CleanProgress>,
    ) -> Result<()> {
        let selected: Vec<&CleanItem> = items.iter().filter(|i| i.selected).collect();

        if selected.is_empty() {
            return Ok(());
        }

        // Phase 1: Backup
        let paths: Vec<String> = selected.iter().map(|i| i.path.clone()).collect();
        let manifest = self
            .backup
            .create_backup(&paths, &format!("Clean {} items", selected.len()))?;

        // Backup manifest is stored internally, no need to send progress for it

        // Phase 2: Clean
        let mut cleaned = 0;
        for item in &selected {
            let _ = sender.send(CleanProgress::File(item.path.clone()));

            // Try clean command first
            if !item.clean_command.is_empty()
                && (item.clean_command.starts_with("apt")
                    || item.clean_command.starts_with("pacman")
                    || item.clean_command.starts_with("dnf")
                    || item.clean_command.starts_with("zypper"))
            {
                let cmd_parts: Vec<&str> = item.clean_command.split_whitespace().collect();
                if !cmd_parts.is_empty() {
                    if let Ok(output) = Command::new("sudo")
                        .args(&cmd_parts)
                        .output()
                    {
                        if !output.status.success() {
                            // Try without sudo
                            let _ = Command::new(cmd_parts[0])
                                .args(&cmd_parts[1..])
                                .output();
                        }
                    }
                }
            } else {
                // Direct file/directory removal
                let path = Path::new(&item.path);
                if path.exists() {
                    if path.is_dir() {
                        if let Err(e) = fs::remove_dir_all(path) {
                            // Try with sudo
                            if let Err(e2) = Command::new("sudo")
                                .args(["rm", "-rf", &item.path])
                                .output()
                            {
                                let _ = sender.send(CleanProgress::Error(format!(
                                    "Failed to remove {}: {} / {}",
                                    item.path, e, e2
                                )));
                                continue;
                            }
                        }
                    } else {
                        if let Err(e) = fs::remove_file(path) {
                            if let Err(e2) = Command::new("sudo")
                                .args(["rm", "-f", &item.path])
                                .output()
                            {
                                let _ = sender.send(CleanProgress::Error(format!(
                                    "Failed to remove {}: {} / {}",
                                    item.path, e, e2
                                )));
                                continue;
                            }
                        }
                    }
                }
            }
            cleaned += 1;
            let _ = sender.send(CleanProgress::Done(cleaned));
        }

        let _ = sender.send(CleanProgress::Finished);
        Ok(())
    }
}
