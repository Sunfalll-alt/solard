use crate::config::GnomeConfig;
use crate::theme::ThemeMode;
use anyhow::{Context, Result};
use std::process::Command;

pub struct GnomeSwitcher {
    config: GnomeConfig,
}

impl GnomeSwitcher {
    pub fn new(config: GnomeConfig) -> Self {
        Self { config }
    }

    pub fn apply(&self, mode: ThemeMode) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // 1. Color Scheme (FreeDesktop portal / libadwaita / Firefox / Chrome)
        if self.config.set_color_scheme {
            let scheme_val = match mode {
                ThemeMode::Dark => "prefer-dark",
                ThemeMode::Light => "prefer-light",
            };
            self.run_gsettings(&["set", "org.gnome.desktop.interface", "color-scheme", scheme_val])
                .context("Failed to set org.gnome.desktop.interface color-scheme")?;
            log::info!("GNOME color-scheme set to '{}'", scheme_val);
        }

        // 2. GTK Theme (for legacy GTK 3/4 applications)
        let gtk_theme = match mode {
            ThemeMode::Dark => self.config.gtk_theme_dark.as_deref(),
            ThemeMode::Light => self.config.gtk_theme_light.as_deref(),
        };
        if let Some(theme) = gtk_theme {
            self.run_gsettings(&["set", "org.gnome.desktop.interface", "gtk-theme", theme])
                .context("Failed to set org.gnome.desktop.interface gtk-theme")?;
            log::info!("GNOME gtk-theme set to '{}'", theme);
        }

        // 3. Icon Theme
        let icon_theme = match mode {
            ThemeMode::Dark => self.config.icon_theme_dark.as_deref(),
            ThemeMode::Light => self.config.icon_theme_light.as_deref(),
        };
        if let Some(icons) = icon_theme {
            self.run_gsettings(&["set", "org.gnome.desktop.interface", "icon-theme", icons])
                .context("Failed to set org.gnome.desktop.interface icon-theme")?;
            log::info!("GNOME icon-theme set to '{}'", icons);
        }

        // 4. Wallpaper
        let wallpaper = match mode {
            ThemeMode::Dark => self.config.wallpaper_dark.as_deref(),
            ThemeMode::Light => self.config.wallpaper_light.as_deref(),
        };
        if let Some(wp) = wallpaper {
            let uri = if wp.starts_with("file://") {
                wp.to_string()
            } else {
                let expanded = crate::config::expand_tilde(wp);
                format!("file://{}", expanded.display())
            };

            // GNOME uses picture-uri for light and picture-uri-dark for dark
            match mode {
                ThemeMode::Dark => {
                    let _ = self.run_gsettings(&["set", "org.gnome.desktop.background", "picture-uri-dark", &uri]);
                }
                ThemeMode::Light => {
                    let _ = self.run_gsettings(&["set", "org.gnome.desktop.background", "picture-uri", &uri]);
                }
            }
            // Also ensure primary picture-uri is synchronized for older apps
            let _ = self.run_gsettings(&["set", "org.gnome.desktop.background", "picture-uri", &uri]);
            log::info!("GNOME wallpaper updated to '{}'", uri);
        }

        // 5. GNOME built-in Night Light
        if self.config.control_gnome_night_light {
            let night_light_state = match mode {
                ThemeMode::Dark => "true",
                ThemeMode::Light => "false",
            };
            let _ = self.run_gsettings(&[
                "set",
                "org.gnome.settings-daemon.plugins.color",
                "night-light-enabled",
                night_light_state,
            ]);
            log::info!("GNOME night-light-enabled set to {}", night_light_state);
        }

        Ok(())
    }

    fn run_gsettings(&self, args: &[&str]) -> Result<()> {
        let output = Command::new("gsettings")
            .args(args)
            .output()
            .with_context(|| format!("Failed to execute 'gsettings {}'", args.join(" ")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::warn!("gsettings {} returned non-zero exit code: {}", args.join(" "), stderr);
        }

        Ok(())
    }
}
