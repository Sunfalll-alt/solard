use crate::config::{expand_tilde, AlacrittyConfig};
use crate::theme::ThemeMode;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub struct AlacrittySwitcher {
    config: AlacrittyConfig,
}

impl AlacrittySwitcher {
    pub fn new(config: AlacrittyConfig) -> Self {
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

        if let Some(theme_str) = theme_opt {
            let target_path = expand_tilde(theme_str);
            if target_path.exists() {
                let link_path = expand_tilde(&self.config.symlink_path);
                self.update_symlink(&target_path, &link_path)?;
                log::info!("Alacritty symlink updated: {:?} -> {:?}", link_path, target_path);
            }
        }

        Ok(())
    }

    fn update_symlink(&self, target: &Path, link: &Path) -> Result<()> {
        if let Some(parent) = link.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp_link = link.with_extension(format!("tmp.{}", std::process::id()));
        let _ = fs::remove_file(&tmp_link);
        std::os::unix::fs::symlink(target, &tmp_link)
            .with_context(|| format!("Failed to symlink {:?} -> {:?}", tmp_link, target))?;
        fs::rename(&tmp_link, link)
            .with_context(|| format!("Failed to atomically rename symlink to {:?}", link))?;
        Ok(())
    }
}
