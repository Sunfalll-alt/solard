use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    Dark,
}

impl ThemeMode {
    pub fn toggle(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }

    #[allow(dead_code)]
    pub fn is_dark(self) -> bool {
        matches!(self, Self::Dark)
    }

    #[allow(dead_code)]
    pub fn is_light(self) -> bool {
        matches!(self, Self::Light)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

impl fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ThemeMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "light" | "day" => Ok(ThemeMode::Light),
            "dark" | "night" => Ok(ThemeMode::Dark),
            other => Err(format!("Unknown theme mode '{}', expected 'light' or 'dark'", other)),
        }
    }
}
