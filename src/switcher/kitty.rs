use crate::config::{expand_tilde, KittyConfig};
use crate::theme::ThemeMode;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct KittySwitcher {
    config: KittyConfig,
}

impl KittySwitcher {
    pub fn new(config: KittyConfig) -> Self {
        Self { config }
    }

    pub fn apply(&self, mode: ThemeMode) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let theme_opt = match mode {
            ThemeMode::Light => self.config.theme_light.as_deref(),
            ThemeMode::Dark => self.config.theme_dark.as_deref(),
        };

        let theme_str = match theme_opt {
            Some(t) => t,
            None => {
                log::debug!("No Kitty theme specified for mode {}", mode);
                return Ok(());
            }
        };

        let target_theme_path = expand_tilde(theme_str);
        if !target_theme_path.exists() {
            log::warn!(
                "Kitty theme file does not exist at '{}'. Creating default sample theme if needed.",
                target_theme_path.display()
            );
            Self::ensure_sample_theme(&target_theme_path, mode)?;
        }

        let symlink_path = expand_tilde(&self.config.symlink_path);
        self.update_symlink(&target_theme_path, &symlink_path)
            .with_context(|| format!("Failed to update Kitty symlink at {:?}", symlink_path))?;

        log::info!(
            "Kitty symlink updated: {:?} -> {:?}",
            symlink_path,
            target_theme_path
        );

        // 1. Live reload via SIGUSR1 signal (Kitty standard hot reload)
        if self.config.send_sigusr1 {
            let res = Command::new("pkill")
                .arg("-SIGUSR1")
                .arg("^kitty$")
                .output();
            match res {
                Ok(out) => {
                    if out.status.success() {
                        log::info!("Sent SIGUSR1 to Kitty processes for instant theme reload");
                    } else {
                        log::debug!("pkill -SIGUSR1 kitty returned {}", out.status);
                    }
                }
                Err(err) => {
                    log::warn!("Failed to execute pkill: {}", err);
                }
            }
        }

        // 2. Optional: Kitty remote control socket
        if self.config.use_remote_control {
            let _ = Command::new("kitty")
                .args(["@", "set-colors", "--all", target_theme_path.to_str().unwrap_or("")])
                .output();
        }

        Ok(())
    }

    fn update_symlink(&self, target: &Path, link: &Path) -> Result<()> {
        if let Some(parent) = link.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create temporary symlink and rename atomically
        let tmp_link = link.with_extension(format!("tmp.{}", std::process::id()));
        let _ = fs::remove_file(&tmp_link);

        std::os::unix::fs::symlink(target, &tmp_link)
            .with_context(|| format!("Failed to symlink {:?} -> {:?}", tmp_link, target))?;

        fs::rename(&tmp_link, link)
            .with_context(|| format!("Failed to atomically rename symlink to {:?}", link))?;

        Ok(())
    }

    fn ensure_sample_theme(path: &Path, mode: ThemeMode) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let sample = match mode {
            ThemeMode::Light => {
                r#"# Solard Sample Kitty Light Theme (Solarized Light)
background #fdf6e3
foreground #657b83
selection_background #eee8d5
selection_foreground #586e75
cursor #586e75
cursor_text_color #fdf6e3
color0 #073642
color1 #dc322f
color2 #859900
color3 #b58900
color4 #268bd2
color5 #d33682
color6 #2aa198
color7 #eee8d5
color8 #002b36
color9 #cb4b16
color10 #586e75
color11 #657b83
color12 #839496
color13 #6c71c4
color14 #93a1a1
color15 #fdf6e3
"#
            }
            ThemeMode::Dark => {
                r#"# Solard Sample Kitty Dark Theme (Solarized Dark)
background #002b36
foreground #839496
selection_background #073642
selection_foreground #93a1a1
cursor #93a1a1
cursor_text_color #002b36
color0 #073642
color1 #dc322f
color2 #859900
color3 #b58900
color4 #268bd2
color5 #d33682
color6 #2aa198
color7 #eee8d5
color8 #002b36
color9 #cb4b16
color10 #586e75
color11 #657b83
color12 #839496
color13 #6c71c4
color14 #93a1a1
color15 #fdf6e3
"#
            }
        };

        fs::write(path, sample)?;
        log::info!("Created sample theme at {:?}", path);
        Ok(())
    }
}
