use crate::config::NotificationsConfig;
use crate::moon::MoonCalculator;
use crate::theme::ThemeMode;
use anyhow::Result;
use chrono::{Local, Utc};
use std::process::Command;

pub struct NotificationSwitcher {
    config: NotificationsConfig,
}

impl NotificationSwitcher {
    pub fn new(config: NotificationsConfig) -> Self {
        Self { config }
    }

    pub fn notify(&self, mode: ThemeMode) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let now_utc = Utc::now();
        let moon = MoonCalculator::calculate_phase(now_utc.date_naive());

        let (title, body, icon) = match mode {
            ThemeMode::Light => (
                "☀️ Светлая тема активирована",
                format!("Наступил день. Фаза Луны: {} {} ({}%)", moon.emoji, moon.phase_name, moon.illumination_pct),
                "weather-clear",
            ),
            ThemeMode::Dark => (
                "🌙 Тёмная тема активирована",
                format!("Наступила ночь. Фаза Луны: {} {} ({}%)", moon.emoji, moon.phase_name, moon.illumination_pct),
                "weather-clear-night",
            ),
        };

        let _ = Command::new("notify-send")
            .args([
                "-a", "Solard",
                "-i", icon,
                "-u", "low",
                title,
                &body,
            ])
            .output();

        Ok(())
    }
}
