use crate::utils::error::{AppError, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub id: String,
    pub timestamp: String,
    pub items: Vec<BackupEntry>,
    pub total_size: u64,
    pub description: String,
    #[serde(default)]
    pub verified: bool,
    #[serde(default)]
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEntry {
    pub original_path: String,
    pub backup_path: String,
    pub size: u64,
    pub is_dir: bool,
}

pub struct Backup {
    location: PathBuf,
    enabled: bool,
}

impl Backup {
    pub fn new(location: PathBuf, enabled: bool) -> Self {
        Backup { location, enabled }
    }

    pub fn create_backup(
        &self,
        paths: &[String],
        description: &str,
    ) -> Result<BackupManifest> {
        if !self.enabled {
            return Ok(BackupManifest {
                id: String::new(),
                timestamp: Utc::now().to_rfc3339(),
                items: vec![],
                total_size: 0,
                description: description.to_string(),
                verified: false,
                checksum: None,
            });
        }

        let backup_id = Uuid::new_v4().to_string();
        let backup_dir = self.location.join(&backup_id);
        fs::create_dir_all(&backup_dir)?;

        let mut manifest = BackupManifest {
            id: backup_id.clone(),
            timestamp: Utc::now().to_rfc3339(),
            items: vec![],
            total_size: 0,
            description: description.to_string(),
            verified: false,
            checksum: None,
        };

        for path_str in paths {
            let src = Path::new(path_str);
            if !src.exists() {
                continue;
            }

            let relative = if path_str.starts_with('/') {
                path_str.trim_start_matches('/')
            } else {
                path_str
            };
            let dest = backup_dir.join(relative);

            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }

            let metadata = src.metadata()?;
            let is_dir = metadata.is_dir();
            let size = metadata.len();

            if is_dir {
                copy_dir_recursive(src, &dest)?;
                let dir_size = crate::utils::disk::dir_size(src);
                manifest.items.push(BackupEntry {
                    original_path: path_str.clone(),
                    backup_path: dest.to_string_lossy().to_string(),
                    size: dir_size,
                    is_dir: true,
                });
                manifest.total_size += dir_size;
            } else {
                fs::copy(src, &dest)?;
                manifest.items.push(BackupEntry {
                    original_path: path_str.clone(),
                    backup_path: dest.to_string_lossy().to_string(),
                    size,
                    is_dir: false,
                });
                manifest.total_size += size;
            }
        }

        // Write manifest
        let manifest_path = backup_dir.join("manifest.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(&manifest_path, manifest_json)?;

        Ok(manifest)
    }

    pub fn restore_backup(&self, backup_id: &str) -> Result<()> {
        let backup_dir = self.location.join(backup_id);
        if !backup_dir.exists() {
            return Err(AppError::Backup(format!(
                "Backup not found: {}",
                backup_id
            )));
        }

        let manifest_path = backup_dir.join("manifest.json");
        let content = fs::read_to_string(&manifest_path)?;
        let manifest: BackupManifest = serde_json::from_str(&content)?;

        for entry in &manifest.items {
            let src = Path::new(&entry.backup_path);
            let dest = Path::new(&entry.original_path);

            if entry.is_dir && src.exists() {
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                copy_dir_recursive(src, dest)?;
            } else if src.exists() {
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(src, dest)?;
            }
        }

        Ok(())
    }

    pub fn list_backups(&self) -> Result<Vec<BackupManifest>> {
        let mut manifests = Vec::new();
        if !self.location.exists() {
            return Ok(manifests);
        }

        for entry in fs::read_dir(&self.location)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("manifest.json");
                if manifest_path.exists() {
                    if let Ok(content) = fs::read_to_string(&manifest_path) {
                        if let Ok(manifest) = serde_json::from_str::<BackupManifest>(&content) {
                            manifests.push(manifest);
                        }
                    }
                }
            }
        }

        manifests.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(manifests)
    }

    pub fn cleanup_old(&self, retention_days: u64, max_backups: usize) -> Result<()> {
        let backups = self.list_backups()?;

        // Remove based on count
        if backups.len() > max_backups {
            for backup in backups.iter().skip(max_backups) {
                let path = self.location.join(&backup.id);
                if path.exists() {
                    fs::remove_dir_all(&path)?;
                }
            }
        }

        // Remove based on age
        let now = Utc::now();
        for backup in &backups {
            if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(&backup.timestamp) {
                let age = now.signed_duration_since(ts);
                if age.num_days() as u64 > retention_days {
                    let path = self.location.join(&backup.id);
                    if path.exists() {
                        fs::remove_dir_all(&path)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn verify_backup(&self, backup_id: &str) -> Result<BackupVerification> {
        let backup_dir = self.location.join(backup_id);
        if !backup_dir.exists() {
            return Err(AppError::Backup(format!(
                "Backup not found: {}",
                backup_id
            )));
        }

        let manifest_path = backup_dir.join("manifest.json");
        let content = fs::read_to_string(&manifest_path)?;
        let mut manifest: BackupManifest = serde_json::from_str(&content)?;

        let mut verified_items = 0;
        let mut missing_items = 0;
        let mut corrupted_items = 0;

        for entry in &manifest.items {
            let backup_path = Path::new(&entry.backup_path);
            if !backup_path.exists() {
                missing_items += 1;
                continue;
            }

            if entry.is_dir {
                let current_size = crate::utils::disk::dir_size(backup_path);
                if current_size != entry.size {
                    corrupted_items += 1;
                } else {
                    verified_items += 1;
                }
            } else {
                if let Ok(meta) = backup_path.metadata() {
                    if meta.len() == entry.size {
                        verified_items += 1;
                    } else {
                        corrupted_items += 1;
                    }
                } else {
                    corrupted_items += 1;
                }
            }
        }

        manifest.verified = missing_items == 0 && corrupted_items == 0;
        
        // Update manifest
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(&manifest_path, manifest_json)?;

        Ok(BackupVerification {
            backup_id: backup_id.to_string(),
            verified: manifest.verified,
            total_items: manifest.items.len(),
            verified_items,
            missing_items,
            corrupted_items,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupVerification {
    pub backup_id: String,
    pub verified: bool,
    pub total_items: usize,
    pub verified_items: usize,
    pub missing_items: usize,
    pub corrupted_items: usize,
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path)?;
        }
    }
    Ok(())
}
