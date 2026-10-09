use crate::config::Config;
use crate::daemon::Daemon;
use crate::solar::{SolarCalculator, SolarTimes};
use crate::switcher::ThemeManager;
use crate::theme::ThemeMode;
use chrono::{DateTime, Local, Utc};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::Command;

pub struct InteractiveMenu {
    config_path: PathBuf,
    selected_index: usize,
}

enum Key {
    Up,
    Down,
    Enter,
    Char(char),
    Quit,
    Other,
}

impl InteractiveMenu {
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            selected_index: 0,
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let _raw_guard = RawTerminalGuard::new()?;

        loop {
            self.draw()?;

            match Self::read_key() {
                Key::Up => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    } else {
                        self.selected_index = 7; // wrap to bottom
                    }
                }
                Key::Down => {
                    if self.selected_index < 7 {
                        self.selected_index += 1;
                    } else {
                        self.selected_index = 0; // wrap to top
                    }
                }
                Key::Enter => {
                    if self.handle_action(self.selected_index)? {
                        break;
                    }
                }
                Key::Char('1') => { if self.handle_action(0)? { break; } }
                Key::Char('2') => { if self.handle_action(1)? { break; } }
                Key::Char('3') => { if self.handle_action(2)? { break; } }
                Key::Char('4') => { if self.handle_action(3)? { break; } }
                Key::Char('5') => { if self.handle_action(4)? { break; } }
                Key::Char('6') => { if self.handle_action(5)? { break; } }
                Key::Char('7') => { if self.handle_action(6)? { break; } }
                Key::Char('8') => { if self.handle_action(7)? { break; } }
                Key::Quit | Key::Char('q') => break,
                _ => {}
            }
        }

        // Clear screen and show cursor on exit
        print!("\x1b[2J\x1b[1;1H\x1b[?25h");
        io::stdout().flush()?;
        Ok(())
    }

    fn draw(&self) -> anyhow::Result<()> {
        let config = Config::load_or_default(&self.config_path).unwrap_or_default();
        let daemon_info = self.get_daemon_status();
        let solar_info = self.get_solar_info(&config);

        // Clear screen, move cursor to top-left, hide cursor
        print!("\x1b[2J\x1b[1;1H\x1b[?25l");

        // Banner with ASCII Sun & Moon Art
        println!("\x1b[1;36m");
        println!(r#"  ███████╗ ██████╗ ██╗      █████╗ ██████╗ ██████╗ "#);
        println!(r#"  ██╔════╝██╔═══██╗██║     ██╔══██╗██╔══██╗██╔══██╗"#);
        println!(r#"  ███████╗██║   ██║██║     ███████║██████╔╝██║  ██║"#);
        println!(r#"  ╚════██║██║   ██║██║     ██╔══██║██╔══██╗██║  ██║"#);
        println!(r#"  ███████║╚██████╔╝███████╗██║  ██║██║  ██║██████╔╝"#);
        println!(r#"  ╚══════╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═════╝ "#);
        println!("\x1b[0m");
        println!("       \x1b[1;33m☀️  SOLAR THEME DAEMON FOR GNOME & KITTY  🌙\x1b[0m\n");

        // Live Status Box
        println!("\x1b[1;37m┌─────────────────── ТЕКУЩИЙ СТАТУС ────────────────────┐\x1b[0m");
        println!("│ Демон:       {:<40} │", daemon_info);
        println!("│ Тема сейчас: {:<40} │", solar_info.current_theme);
        println!("│ Солнце:      {:<40} │", solar_info.sun_times);
        println!("│ Следующее:   {:<40} │", solar_info.next_event);
        println!("│ Координаты:  {:<40} │", format!("{:.2}° N, {:.2}° E", config.location.latitude, config.location.longitude));
        println!("\x1b[1;37m└───────────────────────────────────────────────────────┘\x1b[0m\n");

        // Interactive Options
        let options = [
            ("1", "☀️  Включить дневную (светлую) тему", "Применить режим дня для GNOME, Kitty, Qt"),
            ("2", "🌙  Включить ночную (тёмную) тему", "Применить режим ночи для GNOME, Kitty, Qt"),
            ("3", "🔄  Инвертировать тему (Toggle)", "Быстрое переключение светлая <-> тёмная"),
            ("4", "📊  Подробный статус (Status)", "Показать полные данные и состояние"),
            ("5", "📅  Таблица восходов на 7 дней (Calc)", "Астрономический график солнца на неделю"),
            ("6", "🚀  Запустить демон в фоне (Systemd)", "Включить автопереключение по солнцу"),
            ("7", "⏹️   Остановить демон (Restore)", "Остановить и вернуть изначальные настройки"),
            ("8", "🚪  Выход из меню (Quit)", "Завершить работу меню"),
        ];

        println!("\x1b[1mВыберите действие (стрелки ↑/↓ или клавиши 1-8):\x1b[0m\n");

        for (i, (num, title, desc)) in options.iter().enumerate() {
            if i == self.selected_index {
                // Highlighted row with background color
                println!("  \x1b[1;30;46m ▶ [{}] {:<40} \x1b[0m \x1b[90m({})\x1b[0m", num, title, desc);
            } else {
                println!("    \x1b[1;37m[{}]\x1b[0m {:<42} \x1b[90m({})\x1b[0m", num, title, desc);
            }
        }

        println!("\n\x1b[90m[Enter] Выбрать  │  [q / Esc] Выход  │  [j/k] Навигация\x1b[0m");

        io::stdout().flush()?;
        Ok(())
    }

    fn handle_action(&self, index: usize) -> anyhow::Result<bool> {
        // Temporarily leave raw mode for clean command output
        print!("\x1b[2J\x1b[1;1H\x1b[?25h");
        io::stdout().flush()?;

        let config = Config::load_or_default(&self.config_path).unwrap_or_default();
        let manager = ThemeManager::new(&config);

        match index {
            0 => {
                println!("\x1b[1;33m▶ Переключение на светлую тему...\x1b[0m");
                manager.apply_all(ThemeMode::Light)?;
                println!("\x1b[1;32m✔ Светлая тема успешно применена!\x1b[0m");
                Self::pause_prompt();
            }
            1 => {
                println!("\x1b[1;34m▶ Переключение на тёмную тему...\x1b[0m");
                manager.apply_all(ThemeMode::Dark)?;
                println!("\x1b[1;32m✔ Тёмная тема успешно применена!\x1b[0m");
                Self::pause_prompt();
            }
            2 => {
                let daemon = Daemon::new(self.config_path.clone())?;
                let next = daemon.determine_mode().toggle();
                println!("\x1b[1;35m▶ Инвертирование темы на {}...\x1b[0m", next);
                manager.apply_all(next)?;
                println!("\x1b[1;32m✔ Тема изменена на: {}\x1b[0m", next);
                Self::pause_prompt();
            }
            3 => {
                println!("\x1b[1;36m=== Подробный статус Solard ===\x1b[0m\n");
                let _ = Command::new("solard").arg("status").status();
                Self::pause_prompt();
            }
            4 => {
                println!("\x1b[1;36m=== Астрономический график солнца на 7 дней ===\x1b[0m\n");
                let _ = Command::new("solard").arg("calc").status();
                Self::pause_prompt();
            }
            5 => {
                println!("\x1b[1;32m▶ Запуск службы Systemd...\x1b[0m");
                let status = Command::new("systemctl")
                    .args(["--user", "restart", "solard.service"])
                    .status();
                match status {
                    Ok(s) if s.success() => {
                        println!("\x1b[1;32m✔ Служба solard.service успешно запущена в фоне!\x1b[0m");
                    }
                    _ => {
                        println!("\x1b[1;33mЗапуск в текущем сеансе (фоновый процесс)...\x1b[0m");
                        let _ = Command::new("nohup")
                            .args(["solard", "daemon"])
                            .spawn();
                        println!("\x1b[1;32m✔ Демон запущен!\x1b[0m");
                    }
                }
                Self::pause_prompt();
            }
            6 => {
                println!("\x1b[1;31m▶ Остановка демона и восстановление исходных настроек...\x1b[0m");
                let _ = Command::new("solard").arg("stop").status();
                println!("\x1b[1;32m✔ Исходные настройки восстановлены!\x1b[0m");
                Self::pause_prompt();
            }
            7 => {
                return Ok(true); // Exit
            }
            _ => {}
        }

        Ok(false)
    }

    fn pause_prompt() {
        println!("\n\x1b[90mНажмите любую клавишу для возврата в меню...\x1b[0m");
        let mut buf = [0u8; 1];
        let _ = io::stdin().read(&mut buf);
    }

    fn get_daemon_status(&self) -> String {
        let state_path = if let Some(dirs) = directories::BaseDirs::new() {
            dirs.data_local_dir().join("solard").join("state.json")
        } else {
            PathBuf::from("/tmp/solard.state")
        };

        if state_path.exists() {
            if let Ok(c) = std::fs::read_to_string(&state_path) {
                if let Some(pid) = c.split("\"pid\":").nth(1) {
                    let pid_clean = pid.trim().trim_end_matches('}').trim();
                    return format!("\x1b[1;32m● Активен (PID {})\x1b[0m", pid_clean);
                }
            }
            return "\x1b[1;32m● Активен\x1b[0m".to_string();
        }

        // Check systemctl
        let out = Command::new("systemctl")
            .args(["--user", "is-active", "solard.service"])
            .output();
        if let Ok(o) = out {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s == "active" {
                return "\x1b[1;32m● Активен (Systemd)\x1b[0m".to_string();
            }
        }

        "\x1b[1;31m○ Не запущен\x1b[0m".to_string()
    }

    fn get_solar_info(&self, config: &Config) -> SolarDisplayInfo {
        let calc = SolarCalculator::new(config.location.latitude, config.location.longitude)
            .with_zenith(config.location.zenith)
            .with_offsets(config.location.sunrise_offset_minutes, config.location.sunset_offset_minutes);

        let now_utc = Utc::now();
        let is_day = calc.is_daytime(now_utc);

        let current_theme = if is_day {
            "\x1b[1;33m☀️  Светлая (День)\x1b[0m".to_string()
        } else {
            "\x1b[1;34m🌙  Тёмная (Ночь)\x1b[0m".to_string()
        };

        let mut sun_times = String::from("-");
        if let SolarTimes::Normal { sunrise_utc, sunset_utc } = calc.calculate(now_utc.date_naive()) {
            let sr: DateTime<Local> = sunrise_utc.with_timezone(&Local);
            let ss: DateTime<Local> = sunset_utc.with_timezone(&Local);
            sun_times = format!("Восход {} / Закат {}", sr.format("%H:%M"), ss.format("%H:%M"));
        }

        let (next_trans, target_light) = calc.next_transition(now_utc);
        let next_local: DateTime<Local> = next_trans.with_timezone(&Local);
        let next_event = format!(
            "{} в {}",
            if target_light { "Восход ➔ Светлая" } else { "Закат ➔ Тёмная" },
            next_local.format("%H:%M")
        );

        SolarDisplayInfo {
            current_theme,
            sun_times,
            next_event,
        }
    }

    fn read_key() -> Key {
        let mut buf = [0u8; 3];
        let stdin = io::stdin();
        let mut handle = stdin.lock();

        let n = handle.read(&mut buf).unwrap_or(0);
        if n == 0 {
            return Key::Other;
        }

        if n == 1 {
            match buf[0] {
                b'\r' | b'\n' => Key::Enter,
                b'q' | b'Q' => Key::Quit,
                b'k' => Key::Up,
                b'j' => Key::Down,
                3 => Key::Quit, // Ctrl+C
                c => Key::Char(c as char),
            }
        } else if n == 3 && buf[0] == 27 && buf[1] == b'[' {
            // Escape sequence
            match buf[2] {
                b'A' => Key::Up,
                b'B' => Key::Down,
                _ => Key::Other,
            }
        } else if n == 1 && buf[0] == 27 {
            Key::Quit // Esc
        } else {
            Key::Other
        }
    }
}

struct SolarDisplayInfo {
    current_theme: String,
    sun_times: String,
    next_event: String,
}

/// RAII Guard for enabling and restoring terminal raw mode
struct RawTerminalGuard {
    orig: libc::termios,
}

impl RawTerminalGuard {
    fn new() -> anyhow::Result<Self> {
        unsafe {
            let mut orig = std::mem::zeroed();
            if libc::tcgetattr(libc::STDIN_FILENO, &mut orig) != 0 {
                return Err(anyhow::anyhow!("Failed to get terminal attributes"));
            }

            let mut raw = orig;
            // Disable echo and canonical mode (line buffering)
            raw.c_lflag &= !(libc::ECHO | libc::ICANON);
            // Minimum number of characters for non-canonical read
            raw.c_cc[libc::VMIN] = 1;
            raw.c_cc[libc::VTIME] = 0;

            if libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &raw) != 0 {
                return Err(anyhow::anyhow!("Failed to set raw terminal mode"));
            }

            Ok(Self { orig })
        }
    }
}

impl Drop for RawTerminalGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &self.orig);
            // Make cursor visible again
            print!("\x1b[?25h");
            let _ = io::stdout().flush();
        }
    }
}
