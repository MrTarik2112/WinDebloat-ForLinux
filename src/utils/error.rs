use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Permission denied: {0}. Try running with sudo or check file permissions.")]
    Permission(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Backup error: {0}")]
    Backup(String),

    #[error("Module error: {0}")]
    Module(String),

    #[error("Clean error: {0}")]
    Clean(String),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Not supported: {0}")]
    NotSupported(String),

    #[error("{0}")]
    Other(String),
}

impl AppError {
    pub fn user_message(&self) -> String {
        match self {
            AppError::Permission(_) => "Permission denied. Some operations require elevated privileges.".to_string(),
            AppError::Config(ref s) => format!("Configuration issue: {}. Run 'windebloat config reset' to reset.", s),
            AppError::Backup(ref s) => format!("Backup failed: {}. Your files are safe.", s),
            AppError::Clean(ref s) => format!("Cleaning error: {}. Check logs for details.", s),
            AppError::Module(ref s) => format!("Module error: {}. Try scanning a different category.", s),
            AppError::Io(ref e) if e.kind() == std::io::ErrorKind::NotFound => "File or directory not found.".to_string(),
            AppError::Io(ref e) => format!("IO error: {}", e),
            AppError::NotSupported(ref s) => format!("Feature not supported: {}", s),
            AppError::Other(ref s) => s.clone(),
            other => other.to_string(),
        }
    }

    pub fn is_critical(&self) -> bool {
        matches!(self, AppError::Permission(_) | AppError::Backup(_))
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
