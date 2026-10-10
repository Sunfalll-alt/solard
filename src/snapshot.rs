use crate::config::{expand_tilde, Config};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemSnapshot {
    // GNOME
    pub gnome_color_scheme: Option<String>,
    pub gnome_gtk_theme: Option<String>,
    pub gnome_icon_theme: Option<String>,
    pub gnome_picture_uri: Option<String>,
    pub gnome_picture_uri_dark: Option<String>,
    pub gnome_night_light: Option<String>,

    // Cinnamon (Linux Mint)
    pub cinnamon_color_scheme: Option<String>,
    pub cinnamon_gtk_theme: Option<String>,
    pub cinnamon_theme: Option<String>,
    pub cinnamon_picture_uri: Option<String>,

    // KDE Plasma
    pub kde_color_scheme: Option<String>,

    // XFCE
    pub xfce_theme: Option<String>,

    // MATE
    pub mate_gtk_theme: Option<String>,

    // Terminals
    pub kitty_symlink_target: Option<String>,
    pub alacritty_symlink_target: Option<String>,
    pub foot_symlink_target: Option<String>,

    // Qt / Kvantum
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

            // 2. Cinnamon settings
            snapshot.cinnamon_color_scheme = Self::get_gsettings("org.cinnamon.desktop.interface", "color-scheme");
            snapshot.cinnamon_gtk_theme = Self::get_gsettings("org.cinnamon.desktop.interface", "gtk-theme");
            snapshot.cinnamon_theme = Self::get_gsettings("org.cinnamon.theme", "name");
            snapshot.cinnamon_picture_uri = Self::get_gsettings("org.cinnamon.desktop.background", "picture-uri");

            // 3. MATE settings
            snapshot.mate_gtk_theme = Self::get_gsettings("org.mate.interface", "gtk-theme");
        }

        // 4. KDE Plasma (read from ~/.config/kdeglobals)
        let kdeglobals = expand_tilde("~/.config/kdeglobals");
        if kdeglobals.exists() {
            if let Ok(content) = fs::read_to_string(&kdeglobals) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("ColorScheme=") {
                        snapshot.kde_color_scheme = Some(trimmed.trim_start_matches("ColorScheme=").trim().to_string());
                        break;
                    }
                }
            }
        }

        // 5. XFCE settings (xfconf-query)
        let xfce_out = Command::new("xfconf-query")
            .args(["-c", "xsettings", "-p", "/Net/ThemeName"])
            .output();
        if let Ok(o) = xfce_out {
            if o.status.success() {
                let theme = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if !theme.is_empty() {
                    snapshot.xfce_theme = Some(theme);
                }
            }
        }

        // 6. Kitty symlink
        if config.kitty.enabled {
            let symlink = expand_tilde(&config.kitty.symlink_path);
            if symlink.is_symlink() {
                if let Ok(target) = fs::read_link(&symlink) {
                    snapshot.kitty_symlink_target = Some(target.to_string_lossy().to_string());
                }
            }
        }

        // 7. Alacritty symlink
        if config.alacritty.enabled {
            let symlink = expand_tilde(&config.alacritty.symlink_path);
            if symlink.is_symlink() {
                if let Ok(target) = fs::read_link(&symlink) {
                    snapshot.alacritty_symlink_target = Some(target.to_string_lossy().to_string());
                }
            }
        }

        // 8. Foot symlink
        if config.foot.enabled {
            let symlink = expand_tilde(&config.foot.symlink_path);
            if symlink.is_symlink() {
                if let Ok(target) = fs::read_link(&symlink) {
                    snapshot.foot_symlink_target = Some(target.to_string_lossy().to_string());
                }
            }
        }

        // 9. Kvantum
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

            // 2. Cinnamon
            if let Some(cs) = &self.cinnamon_color_scheme {
                Self::set_gsettings("org.cinnamon.desktop.interface", "color-scheme", cs);
            }
            if let Some(gt) = &self.cinnamon_gtk_theme {
                Self::set_gsettings("org.cinnamon.desktop.interface", "gtk-theme", gt);
            }
            if let Some(th) = &self.cinnamon_theme {
                Self::set_gsettings("org.cinnamon.theme", "name", th);
            }
            if let Some(pu) = &self.cinnamon_picture_uri {
                Self::set_gsettings("org.cinnamon.desktop.background", "picture-uri", pu);
            }

            // 3. MATE
            if let Some(gt) = &self.mate_gtk_theme {
                Self::set_gsettings("org.mate.interface", "gtk-theme", gt);
            }
        }

        // 4. KDE Plasma
        if let Some(cs) = &self.kde_color_scheme {
            let _ = Command::new("plasma-apply-colorscheme").arg(cs).output();
            log::info!("Restored KDE Plasma color scheme to '{}'", cs);
        }

        // 5. XFCE
        if let Some(th) = &self.xfce_theme {
            let _ = Command::new("xfconf-query")
                .args(["-c", "xsettings", "-p", "/Net/ThemeName", "-s", th])
                .output();
            log::info!("Restored XFCE theme to '{}'", th);
        }

        // 6. Kitty
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

        // 7. Alacritty
        if config.alacritty.enabled {
            if let Some(target_str) = &self.alacritty_symlink_target {
                let target_path = PathBuf::from(target_str);
                let link_path = expand_tilde(&config.alacritty.symlink_path);
                let _ = fs::remove_file(&link_path);
                let _ = std::os::unix::fs::symlink(&target_path, &link_path);
                log::info!("Restored Alacritty symlink to {:?}", target_path);
            }
        }

        // 8. Foot
        if config.foot.enabled {
            if let Some(target_str) = &self.foot_symlink_target {
                let target_path = PathBuf::from(target_str);
                let link_path = expand_tilde(&config.foot.symlink_path);
                let _ = fs::remove_file(&link_path);
                let _ = std::os::unix::fs::symlink(&target_path, &link_path);
                log::info!("Restored Foot symlink to {:?}", target_path);
                let _ = Command::new("pkill").args(["-SIGUSR1", "^foot$"]).output();
            }
        }

        // 9. Kvantum
        if config.qt.enabled {
            if let Some(kv) = &self.kvantum_theme {
                let _ = Command::new("kvantummanager").arg("--set").arg(kv).output();
            }
        }

        // 10. Reset gammastep
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
