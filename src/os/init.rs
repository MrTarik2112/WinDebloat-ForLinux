use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InitSystem {
    Systemd,
    OpenRC,
    Runit,
    S6,
    Other(String),
}

pub fn detect_init() -> InitSystem {
    if Command::new("systemctl")
        .arg("--version")
        .output()
        .is_ok()
    {
        InitSystem::Systemd
    } else if Command::new("rc-status")
        .output()
        .is_ok()
        || std::path::Path::new("/etc/init.d/").exists()
    {
        InitSystem::OpenRC
    } else if std::path::Path::new("/etc/runit/").exists() {
        InitSystem::Runit
    } else if std::path::Path::new("/etc/s6/").exists() {
        InitSystem::S6
    } else {
        InitSystem::Other("unknown".into())
    }
}
