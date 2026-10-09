use crate::config::{expand_tilde, VsCodeConfig};
use crate::theme::ThemeMode;
use anyhow::Result;
use std::fs;

pub struct VsCodeSwitcher {
    config: VsCodeConfig,
}

impl VsCodeSwitcher {
    pub fn new(config: VsCodeConfig) -> Self {
        Self { config }
    }

    pub fn apply(&self, mode: ThemeMode) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let target_theme = match mode {
            ThemeMode::Light => &self.config.theme_light,
            ThemeMode::Dark => &self.config.theme_dark,
        };

        // Candidate settings.json locations (VS Code, Code - OSS, VSCodium)
        let candidates = [
            "~/.config/Code/User/settings.json",
            "~/.config/Code - OSS/User/settings.json",
            "~/.config/VSCodium/User/settings.json",
        ];

        for c in candidates {
            let path = expand_tilde(c);
            if path.exists() {
                if let Ok(mut text) = fs::read_to_string(&path) {
                    if text.contains("\"workbench.colorTheme\"") {
                        // Replace existing key line
                        let mut new_lines = Vec::new();
                        for line in text.lines() {
                            if line.contains("\"workbench.colorTheme\"") {
                                let indent = line.chars().take_while(|c| c.is_whitespace()).collect::<String>();
                                let comma = if line.trim_end().ends_with(',') { "," } else { "" };
                                new_lines.push(format!("{}\"workbench.colorTheme\": \"{}\"{}", indent, target_theme, comma));
                            } else {
                                new_lines.push(line.to_string());
                            }
                        }
                        text = new_lines.join("\n") + "\n";
                        let _ = fs::write(&path, text);
                        log::info!("Updated VS Code theme to '{}' in {:?}", target_theme, path);
                    }
                }
            }
        }

        Ok(())
    }
}
