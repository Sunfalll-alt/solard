use crate::config::{expand_tilde, Config};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemSnapshot {
    pub gnome_color_scheme: Option<String>,
    pub gnome_gtk_theme: Option<String>,
    pub gnome_icon_theme: Option<String>,
    pub gnome_picture_uri: Option<String>,
    pub gnome_picture_uri_dark: Option<String>,
    pub gnome_night_light: Option<String>,
    pub kitty_symlink_target: Option<String>,
    pub kvantum_theme: Option<String>,
}

impl SystemSnapshot {
    pub fn snapshot_file_path() -> PathBuf {
        if let Some(dirs) = directories::BaseDirs::new() {
            dirs.data_local_dir().join("solard").join("initial_snapshot.toml")
        } else {
            PathBuf::from("/tmp/solard_initial_snapshot.toml")
        }
    }

    /// Captures the current system theme and configuration before daemon changes anything
    pub fn capture(config: &Config) -> Self {
        let mut snapshot = Self::default();

        // 1. GNOME settings
        if config.gnome.enabled {
            snapshot.gnome_color_scheme = Self::get_gsettings("org.gnome.desktop.interface", "color-scheme");
            snapshot.gnome_gtk_theme = Self::get_gsettings("org.gnome.desktop.interface", "gtk-theme");
            snapshot.gnome_icon_theme = Self::get_gsettings("org.gnome.desktop.interface", "icon-theme");
            snapshot.gnome_picture_uri = Self::get_gsettings("org.gnome.desktop.background", "picture-uri");
            snapshot.gnome_picture_uri_dark = Self::get_gsettings("org.gnome.desktop.background", "picture-uri-dark");
            snapshot.gnome_night_light = Self::get_gsettings("org.gnome.settings-daemon.plugins.color", "night-light-enabled");
        }

        // 2. Kitty symlink
        if config.kitty.enabled {
            let symlink = expand_tilde(&config.kitty.symlink_path);
            if symlink.is_symlink() {
                if let Ok(target) = fs::read_link(&symlink) {
                    snapshot.kitty_symlink_target = Some(target.to_string_lossy().to_string());
                }
            }
        }

        // 3. Kvantum
        if config.qt.enabled {
            let kvconfig_path = expand_tilde("~/.config/Kvantum/kvantum.kvconfig");
            if kvconfig_path.exists() {
                if let Ok(content) = fs::read_to_string(&kvconfig_path) {
                    for line in content.lines() {
                        if line.starts_with("theme=") {
                            snapshot.kvantum_theme = Some(line.trim_start_matches("theme=").trim().to_string());
                            break;
                        }
                    }
                }
            }
        }

        snapshot
    }

    pub fn save_if_not_exists(&self) -> Result<()> {
        let path = Self::snapshot_file_path();
        if path.exists() {
            log::info!("Initial snapshot already exists at {:?}, keeping original", path);
            return Ok(());
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let toml_str = toml::to_string_pretty(self)?;
        fs::write(&path, toml_str)?;
        log::info!("Initial system theme snapshot saved to {:?}", path);
        Ok(())
    }

    pub fn load() -> Option<Self> {
        let path = Self::snapshot_file_path();
        if !path.exists() {
            return None;
        }

        let content = fs::read_to_string(&path).ok()?;
        toml::from_str(&content).ok()
    }

    /// Restores the captured system state
    pub fn restore(&self, config: &Config) -> Result<()> {
        log::info!("=== Restoring initial system settings ===");

        // 1. GNOME
        if config.gnome.enabled {
            if let Some(cs) = &self.gnome_color_scheme {
                Self::set_gsettings("org.gnome.desktop.interface", "color-scheme", cs);
            }
            if let Some(gt) = &self.gnome_gtk_theme {
                Self::set_gsettings("org.gnome.desktop.interface", "gtk-theme", gt);
            }
            if let Some(it) = &self.gnome_icon_theme {
                Self::set_gsettings("org.gnome.desktop.interface", "icon-theme", it);
            }
            if let Some(pu) = &self.gnome_picture_uri {
                Self::set_gsettings("org.gnome.desktop.background", "picture-uri", pu);
            }
            if let Some(pud) = &self.gnome_picture_uri_dark {
                Self::set_gsettings("org.gnome.desktop.background", "picture-uri-dark", pud);
            }
            if let Some(nl) = &self.gnome_night_light {
                Self::set_gsettings("org.gnome.settings-daemon.plugins.color", "night-light-enabled", nl);
            }
        }

        // 2. Kitty
        if config.kitty.enabled {
            if let Some(target_str) = &self.kitty_symlink_target {
                let target_path = PathBuf::from(target_str);
                let link_path = expand_tilde(&config.kitty.symlink_path);
                let _ = fs::remove_file(&link_path);
                let _ = std::os::unix::fs::symlink(&target_path, &link_path);
                log::info!("Restored Kitty symlink to {:?}", target_path);

                // Reload Kitty
                let _ = Command::new("pkill").arg("-SIGUSR1").arg("^kitty$").output();
            }
        }

        // 3. Kvantum
        if config.qt.enabled {
            if let Some(kv) = &self.kvantum_theme {
                let _ = Command::new("kvantummanager").arg("--set").arg(kv).output();
            }
        }

        // 4. Reset gammastep
        if config.gammastep.enabled {
            let _ = Command::new("gammastep").arg("-x").output();
            let _ = Command::new("systemctl").args(["--user", "stop", "gammastep.service"]).output();
        }

        // Remove snapshot file after successful restore
        let path = Self::snapshot_file_path();
        let _ = fs::remove_file(&path);

        log::info!("=== Initial settings restored successfully ===");
        Ok(())
    }

    fn get_gsettings(schema: &str, key: &str) -> Option<String> {
        let output = Command::new("gsettings")
            .args(["get", schema, key])
            .output()
            .ok()?;
        if output.status.success() {
            let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
            // gsettings get returns string with quotes e.g. 'prefer-dark'
            let cleaned = raw.trim_matches('\'').to_string();
            Some(cleaned)
        } else {
            None
        }
    }

    fn set_gsettings(schema: &str, key: &str, val: &str) {
        let _ = Command::new("gsettings")
            .args(["set", schema, key, val])
            .output();
    }
}
