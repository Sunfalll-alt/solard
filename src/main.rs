mod config;
mod daemon;
mod menu;
mod snapshot;
mod solar;
mod switcher;
mod theme;

use chrono::{Local, Utc};
use clap::{Parser, Subcommand};
use config::Config;
use daemon::Daemon;
use solar::{SolarCalculator, SolarTimes};
use std::path::PathBuf;
use switcher::ThemeManager;
use theme::ThemeMode;

#[derive(Parser, Debug)]
#[command(
    name = "solard",
    author = "Arch Linux GNOME Theme Daemon",
    version = "0.1.0",
    about = "Dynamic solar and scheduled theme switcher for GNOME Wayland, Kitty, Qt, and Gammastep"
)]
struct Cli {
    /// Path to custom config file
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Open interactive TUI control menu
    Menu,
    /// Run as a background daemon
    Daemon,
    /// Immediately set theme to 'light' or 'dark'
    Set {
        /// Mode: 'light' or 'dark'
        mode: ThemeMode,
    },
    /// Toggle theme between light and dark
    Toggle,
    /// Show current status and solar calculations
    Status,
    /// Display sunrise and sunset calculations
    Calc,
    /// Generate default config file at ~/.config/solard/config.toml
    InitConfig {
        /// Overwrite existing config file
        #[arg(short, long)]
        force: bool,
    },
    /// Stop running solard daemon
    Stop,
    /// Enable systemd user autostart service
    Enable,
    /// Disable systemd user autostart service
    Disable,
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();
    let config_path = cli.config.unwrap_or_else(Config::default_path);

    match cli.command.unwrap_or(Commands::Menu) {
        Commands::Menu => {
            let mut m = menu::InteractiveMenu::new(config_path);
            m.run()?;
        }
        Commands::Daemon => {
            let mut daemon = Daemon::new(config_path)?;
            daemon.run()?;
        }
        Commands::Set { mode } => {
            let config = Config::load_or_default(&config_path)?;
            let manager = ThemeManager::new(&config);
            manager.apply_all(mode)?;
            println!("Theme set to: {}", mode);
        }
        Commands::Toggle => {
            let config = Config::load_or_default(&config_path)?;
            let state_path = if let Some(dirs) = directories::BaseDirs::new() {
                dirs.data_local_dir().join("solard").join("state.json")
            } else {
                PathBuf::from("/tmp/solard.state")
            };

            // If daemon is running, try sending SIGUSR1 to it via libc::kill
            let mut signaled = false;
            if state_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&state_path) {
                    if let Some(pid_str) = content.split("\"pid\":").nth(1) {
                        let pid_num = pid_str.trim().trim_end_matches('}').trim().parse::<i32>();
                        if let Ok(pid) = pid_num {
                            let res = unsafe { libc::kill(pid, libc::SIGUSR1) };
                            if res == 0 {
                                println!("Sent toggle signal (SIGUSR1) to running solard daemon (PID {})", pid);
                                signaled = true;
                            }
                        }
                    }
                }
            }

            if !signaled {
                let daemon = Daemon::new(config_path)?;
                let current = daemon.determine_mode();
                let next = current.toggle();
                let manager = ThemeManager::new(&config);
                manager.apply_all(next)?;
                println!("Theme manually toggled to: {}", next);
            }
        }
        Commands::Status => {
            let config = Config::load_or_default(&config_path)?;
            println!("=== Solard Status ===");
            println!("Config path: {}", config_path.display());
            println!("Schedule mode: {:?}", config.schedule.mode);
            println!("Location: Lat {}, Lon {}", config.location.latitude, config.location.longitude);

            let calc = SolarCalculator::new(config.location.latitude, config.location.longitude)
                .with_zenith(config.location.zenith)
                .with_offsets(config.location.sunrise_offset_minutes, config.location.sunset_offset_minutes);

            let now_utc = Utc::now();
            let now_local = Local::now();
            println!("Current local time: {}", now_local.format("%Y-%m-%d %H:%M:%S %Z"));

            match calc.calculate(now_utc.date_naive()) {
                SolarTimes::Normal { sunrise_utc, sunset_utc } => {
                    let sunrise_local = sunrise_utc.with_timezone(&Local);
                    let sunset_local = sunset_utc.with_timezone(&Local);
                    println!("Sunrise (today): {}", sunrise_local.format("%H:%M:%S"));
                    println!("Sunset  (today): {}", sunset_local.format("%H:%M:%S"));
                }
                SolarTimes::PolarDay => println!("Solar condition: Polar Day (Sun never sets)"),
                SolarTimes::PolarNight => println!("Solar condition: Polar Night (Sun never rises)"),
            }

            // Autostart status
            let autostart_out = std::process::Command::new("systemctl")
                .args(["--user", "is-enabled", "solard.service"])
                .output();
            let is_enabled = autostart_out.map(|o| String::from_utf8_lossy(&o.stdout).trim() == "enabled").unwrap_or(false);
            println!("Autostart (Systemd): {}", if is_enabled { "Enabled" } else { "Disabled" });

            let is_day = calc.is_daytime(now_utc);
            println!("Calculated solar state: {}", if is_day { "Day (Light)" } else { "Night (Dark)" });

            let (next_trans, target_light) = calc.next_transition(now_utc);
            let next_trans_local = next_trans.with_timezone(&Local);
            let duration = next_trans - now_utc;
            let total_seconds = duration.num_seconds().max(0);
            let hours = total_seconds / 3600;
            let minutes = (total_seconds % 3600) / 60;
            let seconds = total_seconds % 60;
            let countdown_str = if hours > 0 {
                format!("{}h {:02}m {:02}s", hours, minutes, seconds)
            } else {
                format!("{}m {:02}s", minutes, seconds)
            };

            println!(
                "Next transition: {} at {} (in {})",
                if target_light { "Sunrise -> Light" } else { "Sunset -> Dark" },
                next_trans_local.format("%H:%M:%S"),
                countdown_str
            );
        }
        Commands::Calc => {
            let config = Config::load_or_default(&config_path)?;
            let calc = SolarCalculator::new(config.location.latitude, config.location.longitude)
                .with_zenith(config.location.zenith)
                .with_offsets(config.location.sunrise_offset_minutes, config.location.sunset_offset_minutes);

            println!("Solar Schedule for Lat: {:.4}, Lon: {:.4}", config.location.latitude, config.location.longitude);
            println!("{:<12} | {:<10} | {:<10} | {:<12}", "Date", "Sunrise", "Sunset", "Day Length");
            println!("------------------------------------------------------------");

            let mut date = Utc::now().date_naive();
            for _ in 0..7 {
                match calc.calculate(date) {
                    SolarTimes::Normal { sunrise_utc, sunset_utc } => {
                        let sr_local = sunrise_utc.with_timezone(&Local);
                        let ss_local = sunset_utc.with_timezone(&Local);
                        let duration = sunset_utc - sunrise_utc;
                        let hours = duration.num_hours();
                        let mins = duration.num_minutes() % 60;
                        println!(
                            "{:<12} | {:<10} | {:<10} | {}h {}m",
                            date.format("%Y-%m-%d"),
                            sr_local.format("%H:%M"),
                            ss_local.format("%H:%M"),
                            hours,
                            mins
                        );
                    }
                    SolarTimes::PolarDay => {
                        println!("{:<12} | {:<10} | {:<10} | 24h 00m (Polar Day)", date.format("%Y-%m-%d"), "-", "-");
                    }
                    SolarTimes::PolarNight => {
                        println!("{:<12} | {:<10} | {:<10} | 00h 00m (Polar Night)", date.format("%Y-%m-%d"), "-", "-");
                    }
                }
                date = date + chrono::Duration::days(1);
            }
        }
        Commands::InitConfig { force } => {
            if config_path.exists() && !force {
                eprintln!("Configuration file already exists at {}. Use --force to overwrite.", config_path.display());
                return Ok(());
            }
            let default_cfg = Config::default();
            default_cfg.save(&config_path)?;
            println!("Configuration initialized successfully at {}", config_path.display());
        }
        Commands::Stop => {
            // 1. Try stopping via systemd user service if active
            let _ = std::process::Command::new("systemctl")
                .args(["--user", "stop", "solard.service"])
                .output();

            // 2. Also check state file PID and send SIGTERM
            let state_path = if let Some(dirs) = directories::BaseDirs::new() {
                dirs.data_local_dir().join("solard").join("state.json")
            } else {
                PathBuf::from("/tmp/solard.state")
            };

            let mut stopped = false;
            if state_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&state_path) {
                    if let Some(pid_str) = content.split("\"pid\":").nth(1) {
                        let pid_num = pid_str.trim().trim_end_matches('}').trim().parse::<i32>();
                        if let Ok(pid) = pid_num {
                            let res = unsafe { libc::kill(pid, libc::SIGTERM) };
                            if res == 0 {
                                println!("Stopped solard daemon (PID {})", pid);
                                stopped = true;
                            }
                        }
                    }
                }
                let _ = std::fs::remove_file(&state_path);
            }

            // 3. Fallback pkill
            let _ = std::process::Command::new("pkill")
                .args(["-SIGTERM", "^solard$"])
                .output();

            // 4. Restore original system theme settings captured at startup
            if let Some(snapshot) = snapshot::SystemSnapshot::load() {
                if let Ok(cfg) = Config::load_or_default(&config_path) {
                    let _ = snapshot.restore(&cfg);
                    println!("Restored original system settings");
                }
            }

            if !stopped {
                println!("Solard daemon stopped");
            }
        }
        Commands::Enable => {
            if let Some(dirs) = directories::BaseDirs::new() {
                let unit_path = dirs.config_dir().join("systemd").join("user").join("solard.service");
                if let Some(parent) = unit_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                const SYSTEMD_UNIT: &str = r#"[Unit]
Description=Solard - Dynamic Solar & Scheduled Theme Daemon
Documentation=https://github.com/Sunfalll-alt/solard
After=graphical-session.target
PartOf=graphical-session.target

[Service]
Type=simple
ExecStart=%h/.cargo/bin/solard daemon
Restart=on-failure
RestartSec=5s

Environment=WAYLAND_DISPLAY=wayland-0
Environment=XDG_CURRENT_DESKTOP=GNOME
Environment=RUST_LOG=info

[Install]
WantedBy=graphical-session.target
"#;
                let _ = std::fs::write(&unit_path, SYSTEMD_UNIT);
            }
            let _ = std::process::Command::new("systemctl").args(["--user", "daemon-reload"]).status();
            let status = std::process::Command::new("systemctl")
                .args(["--user", "enable", "--now", "solard.service"])
                .status();
            match status {
                Ok(s) if s.success() => println!("Autostart enabled and solard.service started!"),
                _ => eprintln!("Failed to enable autostart via systemctl"),
            }
        }
        Commands::Disable => {
            let status = std::process::Command::new("systemctl")
                .args(["--user", "disable", "--now", "solard.service"])
                .status();
            match status {
                Ok(s) if s.success() => println!("Autostart disabled and solard.service stopped."),
                _ => eprintln!("Failed to disable autostart via systemctl"),
            }
        }
    }

    Ok(())
}
