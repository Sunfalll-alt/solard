use crate::config::{expand_tilde, GnomeConfig};
use crate::theme::ThemeMode;
use anyhow::Result;
use std::process::Command;

pub struct XfceSwitcher {
    config: GnomeConfig,
}

impl XfceSwitcher {
    pub fn new(config: GnomeConfig) -> Self {
        Self { config }
    }

    pub fn apply(&self, mode: ThemeMode) -> Result<()> {
        log::info!("Applying XFCE theme for: {}", mode);

        let gtk_theme = match mode {
            ThemeMode::Dark => self.config.gtk_theme_dark.as_deref().unwrap_or("Adwaita-dark"),
            ThemeMode::Light => self.config.gtk_theme_light.as_deref().unwrap_or("Adwaita"),
        };

        // 1. Set GTK theme via xfconf-query
        let _ = Command::new("xfconf-query")
            .args(["-c", "xsettings", "-p", "/Net/ThemeName", "-s", gtk_theme])
            .output();

        // 2. Wallpaper
        let wallpaper = match mode {
            ThemeMode::Dark => self.config.wallpaper_dark.as_deref(),
            ThemeMode::Light => self.config.wallpaper_light.as_deref(),
        };
        if let Some(wp) = wallpaper {
            let expanded = expand_tilde(wp);
            let _ = Command::new("xfconf-query")
                .args([
                    "-c",
                    "xfce4-desktop",
                    "-p",
                    "/backdrop/screen0/monitor0/workspace0/last-image",
                    "-s",
                    expanded.to_str().unwrap_or(""),
                ])
                .output();
        }

        Ok(())
    }
}
