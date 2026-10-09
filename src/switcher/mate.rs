use crate::config::{expand_tilde, GnomeConfig};
use crate::theme::ThemeMode;
use anyhow::Result;
use std::process::Command;

pub struct MateSwitcher {
    config: GnomeConfig,
}

impl MateSwitcher {
    pub fn new(config: GnomeConfig) -> Self {
        Self { config }
    }

    pub fn apply(&self, mode: ThemeMode) -> Result<()> {
        log::info!("Applying MATE theme for: {}", mode);

        let gtk_theme = match mode {
            ThemeMode::Dark => self.config.gtk_theme_dark.as_deref().unwrap_or("Yaru-MATE-dark"),
            ThemeMode::Light => self.config.gtk_theme_light.as_deref().unwrap_or("Yaru-MATE-light"),
        };

        let _ = Command::new("gsettings")
            .args(["set", "org.mate.interface", "gtk-theme", gtk_theme])
            .output();

        let wallpaper = match mode {
            ThemeMode::Dark => self.config.wallpaper_dark.as_deref(),
            ThemeMode::Light => self.config.wallpaper_light.as_deref(),
        };
        if let Some(wp) = wallpaper {
            let expanded = expand_tilde(wp);
            let _ = Command::new("gsettings")
                .args(["set", "org.mate.background", "picture-filename", expanded.to_str().unwrap_or("")])
                .output();
        }

        Ok(())
    }
}
