use super::schema::Config;
use crate::utils::error::Result;
use std::fs;
use std::path::PathBuf;

pub fn config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
    base.join("windebloat").join("config.toml")
}

pub fn load_config() -> Config {
    let path = config_path();
    let config = if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Warning: config parse error (using defaults): {}", e);
                    Config::default()
                }
            },
            Err(e) => {
                eprintln!("Warning: config file read error (using defaults): {}", e);
                Config::default()
            }
        }
    } else {
        let default = Config::default();
        if let Err(_e) = save_default_config(&path) {
            eprintln!("Warning: could not create default config at {}: run 'windebloat config reset' to reset", path.display());
        }
        default
    };

    // ensure backup location exists
    if !config.backup.location.exists() {
        let _ = fs::create_dir_all(&config.backup.location);
    }

    config
}

fn save_default_config(path: &PathBuf) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(&Config::default())
        .map_err(|e| crate::utils::error::AppError::Config(e.to_string()))?;
    fs::write(path, content)?;
    Ok(())
}
