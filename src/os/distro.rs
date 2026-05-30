use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistroFamily {
    Debian,
    Arch,
    Fedora,
    OpenSuse,
    Alpine,
    Void,
    Gentoo,
    Slackware,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OSInfo {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub version_id: Option<String>,
    pub id_like: Vec<String>,
    pub family: DistroFamily,
    pub home_dir: String,
}

impl OSInfo {
    pub fn detect() -> Self {
        let info = os_info::get();
        let (id, name, version_id, id_like) = parse_os_release().unwrap_or_default();

        let family = detect_family(&id, &id_like);

        OSInfo {
            id: id.clone(),
            name: name.unwrap_or_else(|| info.os_type().to_string()),
            version: info.version().to_string().into(),
            version_id,
            id_like,
            family,
            home_dir: dirs::home_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "/root".to_string()),
        }
    }

    pub fn is_debian_based(&self) -> bool {
        self.family == DistroFamily::Debian
    }

    pub fn is_arch_based(&self) -> bool {
        self.family == DistroFamily::Arch
    }

    pub fn is_fedora_based(&self) -> bool {
        self.family == DistroFamily::Fedora
    }

    pub fn is_suse_based(&self) -> bool {
        self.family == DistroFamily::OpenSuse
    }

    pub fn is_alpine(&self) -> bool {
        self.family == DistroFamily::Alpine
    }

    pub fn package_manager(&self) -> &str {
        match self.family {
            DistroFamily::Debian => "apt",
            DistroFamily::Arch => "pacman",
            DistroFamily::Fedora => "dnf",
            DistroFamily::OpenSuse => "zypper",
            DistroFamily::Alpine => "apk",
            DistroFamily::Void => "xbps",
            DistroFamily::Gentoo => "emerge",
            DistroFamily::Slackware => "slackpkg",
            DistroFamily::Other(_) => "unknown",
        }
    }
}

fn parse_os_release() -> Option<(String, Option<String>, Option<String>, Vec<String>)> {
    let paths = vec!["/etc/os-release", "/usr/lib/os-release"];
    for path in &paths {
        if let Ok(content) = fs::read_to_string(path) {
            let mut map = HashMap::new();
            for line in content.lines() {
                if let Some((key, value)) = line.split_once('=') {
                    let val = value.trim_matches('"').to_string();
                    map.insert(key.to_string(), val);
                }
            }
            let id = map.get("ID").cloned().unwrap_or_else(|| "linux".into());
            let name = map.get("NAME").cloned();
            let version_id = map.get("VERSION_ID").cloned();
            let id_like = map
                .get("ID_LIKE")
                .cloned()
                .map(|s| s.split_whitespace().map(String::from).collect())
                .unwrap_or_default();
            return Some((id, name, version_id, id_like));
        }
    }
    let info = os_info::get();
    Some((
        info.os_type().to_string().to_lowercase(),
        Some(info.os_type().to_string()),
        info.version().to_string().into(),
        vec![],
    ))
}

fn detect_family(id: &str, id_like: &[String]) -> DistroFamily {
    let all: Vec<&str> = id_like.iter().map(|s| s.as_str()).chain(std::iter::once(id)).collect();

    for name in &all {
        match *name {
            "debian" | "ubuntu" | "linuxmint" | "pop" | "elementary" | "kali" | "zorin" => {
                return DistroFamily::Debian;
            }
            "arch" | "manjaro" | "endeavouros" | "garuda" | "artix" | "arcolinux" => {
                return DistroFamily::Arch;
            }
            "fedora" | "rhel" | "centos" | "rocky" | "alma" => {
                return DistroFamily::Fedora;
            }
            "suse" | "opensuse" | "sles" => {
                return DistroFamily::OpenSuse;
            }
            "alpine" => {
                return DistroFamily::Alpine;
            }
            "void" => {
                return DistroFamily::Void;
            }
            "gentoo" => {
                return DistroFamily::Gentoo;
            }
            "slackware" => {
                return DistroFamily::Slackware;
            }
            _ => {}
        }
    }
    DistroFamily::Other(id.to_string())
}
