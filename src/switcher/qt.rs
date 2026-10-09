use crate::config::{expand_tilde, QtConfig};
use crate::theme::ThemeMode;
use anyhow::Result;
use std::fs;
use std::process::Command;

pub struct QtSwitcher {
    config: QtConfig,
}

impl QtSwitcher {
    pub fn new(config: QtConfig) -> Self {
        Self { config }
    }

    pub fn apply(&self, mode: ThemeMode) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // 1. Kvantum Theme
        let kvantum_theme = match mode {
            ThemeMode::Light => self.config.kvantum_theme_light.as_deref(),
            ThemeMode::Dark => self.config.kvantum_theme_dark.as_deref(),
        };

        if let Some(theme) = kvantum_theme {
            // Try kvantummanager CLI first
            let res = Command::new("kvantummanager")
                .arg("--set")
                .arg(theme)
                .output();

            match res {
                Ok(out) if out.status.success() => {
                    log::info!("Kvantum theme set to '{}' via kvantummanager", theme);
                }
                _ => {
                    // Fallback to directly editing ~/.config/Kvantum/kvantum.kvconfig
                    let kvconfig_path = expand_tilde("~/.config/Kvantum/kvantum.kvconfig");
                    if let Some(parent) = kvconfig_path.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    let content = format!("[General]\ntheme={}\n", theme);
                    let _ = fs::write(&kvconfig_path, content);
                    log::info!("Kvantum theme written to {:?}", kvconfig_path);
                }
            }
        }

        // 2. qt6ct color scheme
        let qt6_color = match mode {
            ThemeMode::Light => self.config.qt6ct_color_scheme_light.as_deref(),
            ThemeMode::Dark => self.config.qt6ct_color_scheme_dark.as_deref(),
        };

        if let Some(color_scheme) = qt6_color {
            let conf_path = expand_tilde("~/.config/qt6ct/qt6ct.conf");
            if conf_path.exists() {
                if let Ok(mut text) = fs::read_to_string(&conf_path) {
                    if text.contains("color_scheme_path=") {
                        let new_lines: Vec<String> = text
                            .lines()
                            .map(|line| {
                                if line.starts_with("color_scheme_path=") {
                                    format!("color_scheme_path={}", color_scheme)
                                } else {
                                    line.to_string()
                                }
                            })
                            .collect();
                        text = new_lines.join("\n") + "\n";
                        let _ = fs::write(&conf_path, text);
                        log::info!("qt6ct color scheme updated to '{}'", color_scheme);
                    }
                }
            }
        }

        Ok(())
    }
}
