use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub location: LocationConfig,
    pub schedule: ScheduleConfig,
    pub gnome: GnomeConfig,
    pub kitty: KittyConfig,
    pub qt: QtConfig,
    pub gammastep: GammastepConfig,
    pub hooks: HooksConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LocationConfig {
    /// Latitude in decimal degrees (e.g. 59.9343 for Saint Petersburg, 55.7558 for Moscow)
    pub latitude: f64,
    /// Longitude in decimal degrees (e.g. 30.3351 for Saint Petersburg, 37.6173 for Moscow)
    pub longitude: f64,
    /// Zenith in degrees. 90.8333 is official sunrise/sunset, 96.0 is civil twilight (dawn/dusk)
    pub zenith: f64,
    /// Minutes to shift sunrise (negative = switch earlier, positive = switch later)
    pub sunrise_offset_minutes: i64,
    /// Minutes to shift sunset
    pub sunset_offset_minutes: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScheduleMode {
    Solar,
    Fixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScheduleConfig {
    pub mode: ScheduleMode,
    /// Fixed time for light theme (HH:MM format in local time, e.g. "07:30")
    pub fixed_light_time: String,
    /// Fixed time for dark theme (HH:MM format in local time, e.g. "20:00")
    pub fixed_dark_time: String,
    /// Check interval in seconds for daemon polling fallback
    pub check_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GnomeConfig {
    pub enabled: bool,
    /// Switch org.gnome.desktop.interface color-scheme ('prefer-light' / 'prefer-dark')
    pub set_color_scheme: bool,
    /// GTK theme for legacy GTK3/4 apps
    pub gtk_theme_light: Option<String>,
    pub gtk_theme_dark: Option<String>,
    /// Icon theme
    pub icon_theme_light: Option<String>,
    pub icon_theme_dark: Option<String>,
    /// Wallpapers
    pub wallpaper_light: Option<String>,
    pub wallpaper_dark: Option<String>,
    /// Control GNOME built-in Night Light (hardware Wayland blue-light filter)
    pub control_gnome_night_light: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KittyConfig {
    pub enabled: bool,
    /// Path to light theme file (e.g. "~/.config/kitty/themes/latte.conf")
    pub theme_light: Option<String>,
    /// Path to dark theme file (e.g. "~/.config/kitty/themes/mocha.conf")
    pub theme_dark: Option<String>,
    /// Symlink target for kitty theme inclusion (default: "~/.config/kitty/current-theme.conf")
    pub symlink_path: String,
    /// Send SIGUSR1 to all kitty processes to hot-reload config
    pub send_sigusr1: bool,
    /// Run `kitty @ set-colors` if remote control socket is active
    pub use_remote_control: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct QtConfig {
    pub enabled: bool,
    /// Kvantum theme names
    pub kvantum_theme_light: Option<String>,
    pub kvantum_theme_dark: Option<String>,
    /// qt5ct / qt6ct color scheme config file path or theme name
    pub qt6ct_color_scheme_light: Option<String>,
    pub qt6ct_color_scheme_dark: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GammastepConfig {
    pub enabled: bool,
    /// Day color temperature (Kelvin, default 6500)
    pub temp_day: u16,
    /// Night color temperature (Kelvin, default 3500)
    pub temp_night: u16,
    /// Method: "oneshot" (run `gammastep -O <temp>`), "service" (systemctl --user start/stop gammastep)
    pub method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HooksConfig {
    /// Command or script to execute when switching to light theme
    pub on_light: Option<String>,
    /// Command or script to execute when switching to dark theme
    pub on_dark: Option<String>,
}

impl Default for LocationConfig {
    fn default() -> Self {
        Self {
            latitude: 59.9343, // Saint Petersburg default (user can change in config)
            longitude: 30.3351,
            zenith: 90.83333333333333,
            sunrise_offset_minutes: 0,
            sunset_offset_minutes: 0,
        }
    }
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            mode: ScheduleMode::Solar,
            fixed_light_time: "07:30".to_string(),
            fixed_dark_time: "20:30".to_string(),
            check_interval_secs: 60,
        }
    }
}

impl Default for GnomeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            set_color_scheme: true,
            gtk_theme_light: Some("Adwaita".to_string()),
            gtk_theme_dark: Some("Adwaita-dark".to_string()),
            icon_theme_light: None,
            icon_theme_dark: None,
            wallpaper_light: None,
            wallpaper_dark: None,
            control_gnome_night_light: true,
        }
    }
}

impl Default for KittyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            theme_light: Some("~/.config/kitty/themes/solarized-light.conf".to_string()),
            theme_dark: Some("~/.config/kitty/themes/solarized-dark.conf".to_string()),
            symlink_path: "~/.config/kitty/current-theme.conf".to_string(),
            send_sigusr1: true,
            use_remote_control: false,
        }
    }
}

impl Default for QtConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            kvantum_theme_light: None,
            kvantum_theme_dark: None,
            qt6ct_color_scheme_light: None,
            qt6ct_color_scheme_dark: None,
        }
    }
}

impl Default for GammastepConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Default false because GNOME has built-in Night Light on Wayland
            temp_day: 6500,
            temp_night: 3500,
            method: "oneshot".to_string(),
        }
    }
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            on_light: None,
            on_dark: None,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            location: LocationConfig::default(),
            schedule: ScheduleConfig::default(),
            gnome: GnomeConfig::default(),
            kitty: KittyConfig::default(),
            qt: QtConfig::default(),
            gammastep: GammastepConfig::default(),
            hooks: HooksConfig::default(),
        }
    }
}

impl Config {
    pub fn default_path() -> PathBuf {
        if let Some(base_dirs) = directories::BaseDirs::new() {
            base_dirs.config_dir().join("solard").join("config.toml")
        } else {
            PathBuf::from("config.toml")
        }
    }

    pub fn load_or_default<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            log::info!("Config file not found at {:?}, using default settings", path);
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Helper function to expand ~ into user's home directory
pub fn expand_tilde<P: AsRef<Path>>(path: P) -> PathBuf {
    let p = path.as_ref();
    if let Ok(stripped) = p.strip_prefix("~") {
        if let Some(dirs) = directories::BaseDirs::new() {
            return dirs.home_dir().join(stripped);
        }
    }
    p.to_path_buf()
}
