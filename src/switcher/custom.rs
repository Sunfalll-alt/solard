use crate::config::{Config, HooksConfig};
use crate::theme::ThemeMode;
use anyhow::{Context, Result};
use std::process::Command;

pub struct CustomSwitcher {
    hooks: HooksConfig,
    lat: f64,
    lon: f64,
}

impl CustomSwitcher {
    pub fn new(config: &Config) -> Self {
        Self {
            hooks: config.hooks.clone(),
            lat: config.location.latitude,
            lon: config.location.longitude,
        }
    }

    pub fn apply(&self, mode: ThemeMode) -> Result<()> {
        let hook_cmd = match mode {
            ThemeMode::Light => self.hooks.on_light.as_deref(),
            ThemeMode::Dark => self.hooks.on_dark.as_deref(),
        };

        if let Some(cmd) = hook_cmd {
            log::info!("Executing custom hook for {}: {}", mode, cmd);
            let status = Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .env("SOLARD_MODE", mode.as_str())
                .env("SOLARD_LATITUDE", self.lat.to_string())
                .env("SOLARD_LONGITUDE", self.lon.to_string())
                .status()
                .with_context(|| format!("Failed to execute custom hook: {}", cmd))?;

            if !status.success() {
                log::warn!("Custom hook '{}' exited with status {:?}", cmd, status.code());
            }
        }

        Ok(())
    }
}
