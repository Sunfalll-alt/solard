pub mod custom;
pub mod gammastep;
pub mod gnome;
pub mod kitty;
pub mod qt;

use crate::config::Config;
use crate::theme::ThemeMode;
use anyhow::Result;

pub struct ThemeManager {
    gnome: gnome::GnomeSwitcher,
    kitty: kitty::KittySwitcher,
    qt: qt::QtSwitcher,
    gammastep: gammastep::GammastepSwitcher,
    custom: custom::CustomSwitcher,
}

impl ThemeManager {
    pub fn new(config: &Config) -> Self {
        Self {
            gnome: gnome::GnomeSwitcher::new(config.gnome.clone()),
            kitty: kitty::KittySwitcher::new(config.kitty.clone()),
            qt: qt::QtSwitcher::new(config.qt.clone()),
            gammastep: gammastep::GammastepSwitcher::new(config.gammastep.clone()),
            custom: custom::CustomSwitcher::new(config),
        }
    }

    pub fn apply_all(&self, mode: ThemeMode) -> Result<()> {
        log::info!("=== Switching system theme to: {} ===", mode);

        if let Err(e) = self.gnome.apply(mode) {
            log::error!("Error applying GNOME theme: {:#}", e);
        }

        if let Err(e) = self.kitty.apply(mode) {
            log::error!("Error applying Kitty theme: {:#}", e);
        }

        if let Err(e) = self.qt.apply(mode) {
            log::error!("Error applying Qt theme: {:#}", e);
        }

        if let Err(e) = self.gammastep.apply(mode) {
            log::error!("Error applying Gammastep: {:#}", e);
        }

        if let Err(e) = self.custom.apply(mode) {
            log::error!("Error executing custom hooks: {:#}", e);
        }

        log::info!("=== Theme switch completed ===");
        Ok(())
    }
}
