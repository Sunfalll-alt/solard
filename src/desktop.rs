use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DesktopEnvironment {
    Auto,
    Gnome,
    Cinnamon,
    Kde,
    Xfce,
    Mate,
    Generic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Wayland,
    X11,
    Unknown,
}

pub struct SystemInfo {
    pub distro_name: String,
    pub desktop: DesktopEnvironment,
    pub session_type: SessionType,
}

impl DesktopEnvironment {
    pub fn detect() -> Self {
        let xdg_current = env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_uppercase();
        let desktop_session = env::var("DESKTOP_SESSION").unwrap_or_default().to_uppercase();

        if xdg_current.contains("CINNAMON") || desktop_session.contains("CINNAMON") {
            Self::Cinnamon
        } else if xdg_current.contains("KDE") || xdg_current.contains("PLASMA") || desktop_session.contains("PLASMA") {
            Self::Kde
        } else if xdg_current.contains("XFCE") || desktop_session.contains("XFCE") {
            Self::Xfce
        } else if xdg_current.contains("MATE") || desktop_session.contains("MATE") {
            Self::Mate
        } else if xdg_current.contains("GNOME") || xdg_current.contains("UBUNTU") || desktop_session.contains("GNOME") {
            Self::Gnome
        } else {
            Self::Generic
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto-detect",
            Self::Gnome => "GNOME",
            Self::Cinnamon => "Cinnamon (Linux Mint)",
            Self::Kde => "KDE Plasma",
            Self::Xfce => "XFCE",
            Self::Mate => "MATE",
            Self::Generic => "Generic FreeDesktop",
        }
    }
}

impl SessionType {
    pub fn detect() -> Self {
        match env::var("XDG_SESSION_TYPE").unwrap_or_default().to_lowercase().as_str() {
            "wayland" => Self::Wayland,
            "x11" => Self::X11,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Wayland => "Wayland",
            Self::X11 => "X11",
            Self::Unknown => "Unknown",
        }
    }
}

impl SystemInfo {
    pub fn detect() -> Self {
        let distro_name = Self::detect_distro();
        let desktop = DesktopEnvironment::detect();
        let session_type = SessionType::detect();

        Self {
            distro_name,
            desktop,
            session_type,
        }
    }

    fn detect_distro() -> String {
        if let Ok(content) = fs::read_to_string("/etc/os-release") {
            for line in content.lines() {
                if let Some(stripped) = line.strip_prefix("PRETTY_NAME=") {
                    return stripped.trim_matches('"').trim().to_string();
                }
            }
            for line in content.lines() {
                if let Some(stripped) = line.strip_prefix("NAME=") {
                    return stripped.trim_matches('"').trim().to_string();
                }
            }
        }
        "Linux".to_string()
    }
}
