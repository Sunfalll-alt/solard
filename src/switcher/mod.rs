pub mod alacritty;
pub mod cinnamon;
pub mod custom;
pub mod foot;
pub mod gammastep;
pub mod gnome;
pub mod kde;
pub mod kitty;
pub mod mate;
pub mod neovim;
pub mod notifications;
pub mod qt;
pub mod vscode;
pub mod xfce;

use crate::config::Config;
use crate::desktop::DesktopEnvironment;
use crate::theme::ThemeMode;
use anyhow::Result;

pub struct ThemeManager {
    gnome: gnome::GnomeSwitcher,
    cinnamon: cinnamon::CinnamonSwitcher,
    kde: kde::KdeSwitcher,
    xfce: xfce::XfceSwitcher,
    mate: mate::MateSwitcher,
    kitty: kitty::KittySwitcher,
    alacritty: alacritty::AlacrittySwitcher,
    foot: foot::FootSwitcher,
    vscode: vscode::VsCodeSwitcher,
    neovim: neovim::NeovimSwitcher,
    qt: qt::QtSwitcher,
    gammastep: gammastep::GammastepSwitcher,
    notifications: notifications::NotificationSwitcher,
    custom: custom::CustomSwitcher,
}

impl ThemeManager {
    pub fn new(config: &Config) -> Self {
        Self {
            gnome: gnome::GnomeSwitcher::new(config.gnome.clone()),
            cinnamon: cinnamon::CinnamonSwitcher::new(config.gnome.clone()),
            kde: kde::KdeSwitcher::new(config.gnome.clone()),
            xfce: xfce::XfceSwitcher::new(config.gnome.clone()),
            mate: mate::MateSwitcher::new(config.gnome.clone()),
            kitty: kitty::KittySwitcher::new(config.kitty.clone()),
            alacritty: alacritty::AlacrittySwitcher::new(config.alacritty.clone()),
            foot: foot::FootSwitcher::new(config.foot.clone()),
            vscode: vscode::VsCodeSwitcher::new(config.vscode.clone()),
            neovim: neovim::NeovimSwitcher::new(config.neovim.clone()),
            qt: qt::QtSwitcher::new(config.qt.clone()),
            gammastep: gammastep::GammastepSwitcher::new(config.gammastep.clone()),
            notifications: notifications::NotificationSwitcher::new(config.notifications.clone()),
            custom: custom::CustomSwitcher::new(config),
        }
    }

    pub fn apply_all(&self, mode: ThemeMode) -> Result<()> {
        let de = DesktopEnvironment::detect();
        log::info!("=== Switching system theme to: {} (Desktop: {}) ===", mode, de.display_name());

        // 1. Desktop Environment Theme Switcher
        match de {
            DesktopEnvironment::Cinnamon => {
                if let Err(e) = self.cinnamon.apply(mode) {
                    log::error!("Error applying Cinnamon theme: {:#}", e);
                }
            }
            DesktopEnvironment::Kde => {
                if let Err(e) = self.kde.apply(mode) {
                    log::error!("Error applying KDE Plasma theme: {:#}", e);
                }
            }
            DesktopEnvironment::Xfce => {
                if let Err(e) = self.xfce.apply(mode) {
                    log::error!("Error applying XFCE theme: {:#}", e);
                }
            }
            DesktopEnvironment::Mate => {
                if let Err(e) = self.mate.apply(mode) {
                    log::error!("Error applying MATE theme: {:#}", e);
                }
            }
            DesktopEnvironment::Gnome | DesktopEnvironment::Auto | DesktopEnvironment::Generic => {
                if let Err(e) = self.gnome.apply(mode) {
                    log::error!("Error applying GNOME/FreeDesktop theme: {:#}", e);
                }
            }
        }

        // 2. Terminals (Kitty, Alacritty, Foot)
        if let Err(e) = self.kitty.apply(mode) {
            log::error!("Error applying Kitty theme: {:#}", e);
        }
        if let Err(e) = self.alacritty.apply(mode) {
            log::error!("Error applying Alacritty theme: {:#}", e);
        }
        if let Err(e) = self.foot.apply(mode) {
            log::error!("Error applying Foot theme: {:#}", e);
        }

        // 3. Code Editors (VS Code, Neovim)
        if let Err(e) = self.vscode.apply(mode) {
            log::error!("Error applying VS Code theme: {:#}", e);
        }
        if let Err(e) = self.neovim.apply(mode) {
            log::error!("Error applying Neovim theme: {:#}", e);
        }

        // 4. Qt / Kvantum / qt6ct
        if let Err(e) = self.qt.apply(mode) {
            log::error!("Error applying Qt theme: {:#}", e);
        }

        // 5. Gammastep / Redshift
        if let Err(e) = self.gammastep.apply(mode) {
            log::error!("Error applying Gammastep: {:#}", e);
        }

        // 6. Custom user hooks
        if let Err(e) = self.custom.apply(mode) {
            log::error!("Error executing custom hooks: {:#}", e);
        }

        // 7. Desktop Notifications
        if let Err(e) = self.notifications.notify(mode) {
            log::warn!("Desktop notification failed: {:#}", e);
        }

        log::info!("=== Theme switch completed ===");
        Ok(())
    }
}
