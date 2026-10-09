use crate::config::{expand_tilde, GnomeConfig};
use crate::theme::ThemeMode;
use anyhow::Result;
use std::process::Command;

pub struct KdeSwitcher {
    config: GnomeConfig,
}

impl KdeSwitcher {
    pub fn new(config: GnomeConfig) -> Self {
        Self { config }
    }

    pub fn apply(&self, mode: ThemeMode) -> Result<()> {
        log::info!("Applying KDE Plasma theme for: {}", mode);

        let (color_scheme, look_and_feel) = match mode {
            ThemeMode::Dark => ("BreezeDark", "org.kde.breezedark.desktop"),
            ThemeMode::Light => ("BreezeLight", "org.kde.breeze.desktop"),
        };

        // 1. Try plasma-apply-colorscheme (Plasma 5.25+)
        let _ = Command::new("plasma-apply-colorscheme")
            .arg(color_scheme)
            .output();

        // 2. Try plasma-apply-lookandfeel
        let _ = Command::new("plasma-apply-lookandfeel")
            .args(["-a", look_and_feel])
            .output();

        // 3. Wallpaper
        let wallpaper = match mode {
            ThemeMode::Dark => self.config.wallpaper_dark.as_deref(),
            ThemeMode::Light => self.config.wallpaper_light.as_deref(),
        };
        if let Some(wp) = wallpaper {
            let expanded = expand_tilde(wp);
            let _ = Command::new("plasma-apply-wallpaperimage")
                .arg(expanded)
                .output();
        }

        Ok(())
    }
}
