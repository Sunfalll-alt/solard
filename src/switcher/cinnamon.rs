use crate::config::{expand_tilde, GnomeConfig};
use crate::theme::ThemeMode;
use anyhow::Result;
use std::process::Command;

pub struct CinnamonSwitcher {
    config: GnomeConfig,
}

impl CinnamonSwitcher {
    pub fn new(config: GnomeConfig) -> Self {
        Self { config }
    }

    pub fn apply(&self, mode: ThemeMode) -> Result<()> {
        log::info!("Applying Cinnamon (Linux Mint) theme for: {}", mode);

        // 1. Cinnamon color-scheme (in newer Cinnamon versions)
        let scheme_val = match mode {
            ThemeMode::Dark => "prefer-dark",
            ThemeMode::Light => "prefer-light",
        };
        let _ = Command::new("gsettings")
            .args(["set", "org.cinnamon.desktop.interface", "color-scheme", scheme_val])
            .output();

        // 2. GTK theme
        let gtk_theme = match mode {
            ThemeMode::Dark => self.config.gtk_theme_dark.as_deref().unwrap_or("Mint-Y-Dark"),
            ThemeMode::Light => self.config.gtk_theme_light.as_deref().unwrap_or("Mint-Y"),
        };
        let _ = Command::new("gsettings")
            .args(["set", "org.cinnamon.desktop.interface", "gtk-theme", gtk_theme])
            .output();

        // 3. Cinnamon Desktop shell theme
        let _ = Command::new("gsettings")
            .args(["set", "org.cinnamon.theme", "name", gtk_theme])
            .output();

        // 4. Wallpaper
        let wallpaper = match mode {
            ThemeMode::Dark => self.config.wallpaper_dark.as_deref(),
            ThemeMode::Light => self.config.wallpaper_light.as_deref(),
        };
        if let Some(wp) = wallpaper {
            let expanded = expand_tilde(wp);
            let uri = format!("file://{}", expanded.display());
            let _ = Command::new("gsettings")
                .args(["set", "org.cinnamon.desktop.background", "picture-uri", &uri])
                .output();
            log::info!("Cinnamon wallpaper set to {}", uri);
        }

        Ok(())
    }
}
