use crate::config::{Config, ScheduleMode};
use crate::solar::SolarCalculator;
use crate::switcher::ThemeManager;
use crate::theme::ThemeMode;
use anyhow::Result;
use chrono::{Local, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

static RUNNING: AtomicBool = AtomicBool::new(true);
static TOGGLE_REQUESTED: AtomicBool = AtomicBool::new(false);
static RELOAD_REQUESTED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonState {
    pub mode: String,
    pub pid: u32,
    pub paused_until: Option<i64>,
}

impl DaemonState {
    pub fn file_path() -> PathBuf {
        if let Some(dirs) = directories::BaseDirs::new() {
            dirs.data_local_dir().join("solard").join("state.json")
        } else {
            PathBuf::from("/tmp/solard.state")
        }
    }

    pub fn load() -> Option<Self> {
        let path = Self::file_path();
        let content = std::fs::read_to_string(path).ok()?;
        let pid = content
            .split("\"pid\":")
            .nth(1)?
            .split(|c| c == ',' || c == '}' || c == '\n')
            .next()?
            .trim()
            .parse::<u32>()
            .ok()?;
        let mode = content
            .split("\"mode\":")
            .nth(1)?
            .split('"')
            .nth(1)?
            .to_string();
        let paused_until = content
            .split("\"paused_until\":")
            .nth(1)
            .and_then(|s| s.split(|c| c == ',' || c == '}' || c == '\n').next())
            .and_then(|s| s.trim().parse::<i64>().ok());
        Some(Self {
            mode,
            pid,
            paused_until,
        })
    }

    pub fn save(&self) {
        let path = Self::file_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let pause_val = match self.paused_until {
            Some(ts) => ts.to_string(),
            None => "null".to_string(),
        };
        let json = format!(
            "{{\"mode\": \"{}\", \"pid\": {}, \"paused_until\": {}}}\n",
            self.mode, self.pid, pause_val
        );
        let _ = std::fs::write(&path, json);
    }

    pub fn is_paused(&self) -> bool {
        if let Some(ts) = self.paused_until {
            Utc::now().timestamp() < ts
        } else {
            false
        }
    }

    pub fn remaining_secs(&self) -> Option<i64> {
        if let Some(ts) = self.paused_until {
            let diff = ts - Utc::now().timestamp();
            if diff > 0 {
                Some(diff)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn parse_duration(s: &str) -> Result<i64> {
        let s = s.trim().to_lowercase();
        if s.ends_with('h') {
            let num: i64 = s[..s.len() - 1].parse()?;
            Ok(num * 3600)
        } else if s.ends_with('m') {
            let num: i64 = s[..s.len() - 1].parse()?;
            Ok(num * 60)
        } else if s.ends_with('s') {
            let num: i64 = s[..s.len() - 1].parse()?;
            Ok(num)
        } else if s.ends_with('d') {
            let num: i64 = s[..s.len() - 1].parse()?;
            Ok(num * 86400)
        } else {
            let num: i64 = s.parse()?;
            Ok(num)
        }
    }
}

extern "C" fn handle_sigint_term(_: libc::c_int) {
    RUNNING.store(false, Ordering::SeqCst);
}

extern "C" fn handle_sigusr1(_: libc::c_int) {
    TOGGLE_REQUESTED.store(true, Ordering::SeqCst);
}

extern "C" fn handle_sighup(_: libc::c_int) {
    RELOAD_REQUESTED.store(true, Ordering::SeqCst);
}

pub struct Daemon {
    config_path: PathBuf,
    config: Config,
    current_mode: Option<ThemeMode>,
    state_file: PathBuf,
}

impl Daemon {
    pub fn new(config_path: PathBuf) -> Result<Self> {
        let config = Config::load_or_default(&config_path)?;
        let state_file = DaemonState::file_path();
        Ok(Self {
            config_path,
            config,
            current_mode: None,
            state_file,
        })
    }

    fn save_state(&self, mode: ThemeMode) {
        let existing_pause = DaemonState::load().and_then(|s| s.paused_until);
        let state = DaemonState {
            mode: mode.as_str().to_string(),
            pid: std::process::id(),
            paused_until: existing_pause,
        };
        state.save();
    }

    pub fn run(&mut self) -> Result<()> {
        log::info!("Starting Solard Daemon (PID: {})", std::process::id());

        // Setup signal handlers via libc
        unsafe {
            libc::signal(libc::SIGINT, handle_sigint_term as *const () as libc::sighandler_t);
            libc::signal(libc::SIGTERM, handle_sigint_term as *const () as libc::sighandler_t);
            libc::signal(libc::SIGUSR1, handle_sigusr1 as *const () as libc::sighandler_t);
            libc::signal(libc::SIGHUP, handle_sighup as *const () as libc::sighandler_t);
        }

        // Capture initial system theme settings before applying any changes
        let snapshot = crate::snapshot::SystemSnapshot::capture(&self.config);
        let _ = snapshot.save_if_not_exists();

        let mut manager = ThemeManager::new(&self.config);

        // Initial check and theme application
        let initial_mode = self.determine_mode();
        log::info!("Initial target theme determined: {}", initial_mode);
        manager.apply_all(initial_mode)?;
        self.current_mode = Some(initial_mode);
        self.save_state(initial_mode);

        while RUNNING.load(Ordering::SeqCst) {
            // Check if reload requested
            if RELOAD_REQUESTED.swap(false, Ordering::SeqCst) {
                log::info!("SIGHUP received: reloading configuration from {:?}", self.config_path);
                if let Ok(new_cfg) = Config::load_or_default(&self.config_path) {
                    self.config = new_cfg;
                    manager = ThemeManager::new(&self.config);
                    log::info!("Configuration reloaded successfully");
                }
            }

            // Check if toggle requested
            if TOGGLE_REQUESTED.swap(false, Ordering::SeqCst) {
                let current = self.current_mode.unwrap_or(ThemeMode::Dark);
                let toggled = current.toggle();
                log::info!("SIGUSR1 received: manual toggle to {}", toggled);
                if let Err(e) = manager.apply_all(toggled) {
                    log::error!("Failed to toggle theme: {:#}", e);
                }
                self.current_mode = Some(toggled);
                self.save_state(toggled);
            }

            // Check pause status
            let is_paused = DaemonState::load()
                .map(|s| s.is_paused())
                .unwrap_or(false);

            if is_paused {
                log::debug!("Theme auto-switching is currently paused/inhibited");
            } else {
                // Determine if theme should switch based on schedule/sun
                let target_mode = self.determine_mode();
                if Some(target_mode) != self.current_mode {
                    log::info!("Schedule event reached: transitioning to {}", target_mode);
                    if let Err(e) = manager.apply_all(target_mode) {
                        log::error!("Failed to apply target theme: {:#}", e);
                    }
                    self.current_mode = Some(target_mode);
                    self.save_state(target_mode);
                }
            }

            // Sleep in small increments (e.g. 5 seconds) to remain responsive to signals and suspend/resume
            for _ in 0..12 {
                if !RUNNING.load(Ordering::SeqCst)
                    || TOGGLE_REQUESTED.load(Ordering::SeqCst)
                    || RELOAD_REQUESTED.load(Ordering::SeqCst)
                {
                    break;
                }
                thread::sleep(Duration::from_secs(5));
            }
        }

        log::info!("Daemon shutting down: restoring original system settings");
        if let Some(snapshot) = crate::snapshot::SystemSnapshot::load() {
            let _ = snapshot.restore(&self.config);
        }
        let _ = std::fs::remove_file(&self.state_file);
        Ok(())
    }

    pub fn determine_mode(&self) -> ThemeMode {
        match self.config.schedule.mode {
            ScheduleMode::Solar => {
                let calc = SolarCalculator::new(
                    self.config.location.latitude,
                    self.config.location.longitude,
                )
                .with_zenith(self.config.location.zenith)
                .with_offsets(
                    self.config.location.sunrise_offset_minutes,
                    self.config.location.sunset_offset_minutes,
                );

                if calc.is_daytime(Utc::now()) {
                    ThemeMode::Light
                } else {
                    ThemeMode::Dark
                }
            }
            ScheduleMode::Fixed => {
                let now_local = Local::now().time();
                let light_time = NaiveTime::parse_from_str(&self.config.schedule.fixed_light_time, "%H:%M")
                    .unwrap_or_else(|_| NaiveTime::from_hms_opt(7, 30, 0).unwrap());
                let dark_time = NaiveTime::parse_from_str(&self.config.schedule.fixed_dark_time, "%H:%M")
                    .unwrap_or_else(|_| NaiveTime::from_hms_opt(20, 30, 0).unwrap());

                if light_time <= dark_time {
                    if now_local >= light_time && now_local < dark_time {
                        ThemeMode::Light
                    } else {
                        ThemeMode::Dark
                    }
                } else {
                    if now_local >= light_time || now_local < dark_time {
                        ThemeMode::Light
                    } else {
                        ThemeMode::Dark
                    }
                }
            }
        }
    }
}
