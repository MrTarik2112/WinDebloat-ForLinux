use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CleanProfile {
    pub name: String,
    pub safe_only: bool,
    pub skip_categories: Vec<String>,
    pub max_risk: u8,
    pub min_size: u64,
    pub max_size: u64,
    pub no_backup: bool,
    pub no_benchmark: bool,
    pub no_prediction: bool,
}

impl CleanProfile {
    pub fn safe() -> Self {
        CleanProfile {
            name: "safe".to_string(),
            safe_only: true,
            skip_categories: vec!["services".into(), "ssh_keys".into()],
            max_risk: 0,
            min_size: 0,
            max_size: u64::MAX,
            no_backup: false,
            no_benchmark: false,
            no_prediction: false,
        }
    }

    pub fn balanced() -> Self {
        CleanProfile {
            name: "balanced".to_string(),
            safe_only: false,
            skip_categories: vec!["ssh_keys".into()],
            max_risk: 1,
            min_size: 0,
            max_size: u64::MAX,
            no_backup: false,
            no_benchmark: false,
            no_prediction: false,
        }
    }

    pub fn aggressive() -> Self {
        CleanProfile {
            name: "aggressive".to_string(),
            safe_only: false,
            skip_categories: vec![],
            max_risk: 2,
            min_size: 0,
            max_size: u64::MAX,
            no_backup: false,
            no_benchmark: false,
            no_prediction: false,
        }
    }
}

impl Default for CleanProfile {
    fn default() -> Self {
        Self::safe()
    }
}

pub struct ProfileManager {
    pub profiles_dir: PathBuf,
}

impl ProfileManager {
    pub fn new() -> Self {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
        ProfileManager {
            profiles_dir: base.join("windebloat").join("profiles"),
        }
    }

    pub fn get_profile(&self, name: &str) -> CleanProfile {
        match name {
            "safe" => CleanProfile::safe(),
            "balanced" => CleanProfile::balanced(),
            "aggressive" => CleanProfile::aggressive(),
            _ => CleanProfile::safe(),
        }
    }
}
