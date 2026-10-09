use crate::config::GammastepConfig;
use crate::theme::ThemeMode;
use anyhow::Result;
use std::process::Command;

pub struct GammastepSwitcher {
    config: GammastepConfig,
}

impl GammastepSwitcher {
    pub fn new(config: GammastepConfig) -> Self {
        Self { config }
    }

    pub fn apply(&self, mode: ThemeMode) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        match self.config.method.as_str() {
            "service" => {
                let action = match mode {
                    ThemeMode::Dark => "start",
                    ThemeMode::Light => "stop",
                };
                let _ = Command::new("systemctl")
                    .args(["--user", action, "gammastep.service"])
                    .output();
                log::info!("gammastep systemd service '{}' executed", action);
            }
            _ => {
                // "oneshot"
                let binary = if Self::command_exists("gammastep") {
                    "gammastep"
                } else if Self::command_exists("redshift") {
                    "redshift"
                } else {
                    log::warn!("Neither gammastep nor redshift found in PATH");
                    return Ok(());
                };

                match mode {
                    ThemeMode::Light => {
                        // Reset gamma / temperature
                        let _ = Command::new(binary).arg("-x").output();
                        log::info!("Reset {} to default daytime temperature", binary);
                    }
                    ThemeMode::Dark => {
                        let temp_str = self.config.temp_night.to_string();
                        let _ = Command::new(binary).args(["-O", &temp_str]).output();
                        log::info!("Set {} temperature to {}K", binary, temp_str);
                    }
                }
            }
        }

        Ok(())
    }

    fn command_exists(cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
