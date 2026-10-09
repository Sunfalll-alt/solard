use crate::config::{Config, ScheduleMode};
use crate::daemon::Daemon;
use crate::solar::{SolarCalculator, SolarTimes};
use crate::switcher::ThemeManager;
use crate::theme::ThemeMode;
use chrono::{DateTime, Local, Utc};
use std::io::{self, BufRead, Read, Write};
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
                        self.selected_index = 8; // wrap to bottom
                    }
                }
                Key::Down => {
                    if self.selected_index < 8 {
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
                Key::Char('9') => { if self.handle_action(8)? { break; } }
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
        println!("│ Режим:       {:<40} │", format!("{:?}", config.schedule.mode));
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
            ("8", "⚙️   Настройки и конфигурация (Config)", "Изменить координаты, темы, обои, редактор"),
            ("9", "🚪  Выход из меню (Quit)", "Завершить работу меню"),
        ];

        println!("\x1b[1mВыберите действие (стрелки ↑/↓ или клавиши 1-9):\x1b[0m\n");

        for (i, (num, title, desc)) in options.iter().enumerate() {
            if i == self.selected_index {
                // Highlighted row with background color
                println!("  \x1b[1;30;46m ▶ [{}] {:<42} \x1b[0m \x1b[90m({})\x1b[0m", num, title, desc);
            } else {
                println!("    \x1b[1;37m[{}]\x1b[0m {:<44} \x1b[90m({})\x1b[0m", num, title, desc);
            }
        }

        println!("\n\x1b[90m[Enter] Выбрать  │  [q / Esc] Выход  │  [j/k] Навигация\x1b[0m");

        io::stdout().flush()?;
        Ok(())
    }

    fn handle_action(&mut self, index: usize) -> anyhow::Result<bool> {
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
                // Config Editor Submenu
                self.run_config_editor()?;
            }
            8 => {
                return Ok(true); // Exit
            }
            _ => {}
        }

        Ok(false)
    }

    /// Interactive Config Editor Submenu
    fn run_config_editor(&self) -> anyhow::Result<()> {
        loop {
            let mut config = Config::load_or_default(&self.config_path).unwrap_or_default();

            print!("\x1b[2J\x1b[1;1H\x1b[?25h");
            println!("\x1b[1;36m═════════════════════════════════════════════════════════\x1b[0m");
            println!("           \x1b[1;33m⚙️  РЕДАКТОР НАСТРОЕК SOLARD  ⚙️\x1b[0m");
            println!("\x1b[1;36m═════════════════════════════════════════════════════════\x1b[0m\n");

            println!("\x1b[1;37m┌────────────────── ТЕКУЩАЯ КОНФИГУРАЦИЯ ─────────────────┐\x1b[0m");
            println!("│ Режим расписания: {:<37} │", format!("{:?}", config.schedule.mode));
            println!("│ Координаты:       {:<37} │", format!("{:.4}° N, {:.4}° E", config.location.latitude, config.location.longitude));
            println!("│ Фикс. время дня:  {:<37} │", format!("Свет: {} / Тьма: {}", config.schedule.fixed_light_time, config.schedule.fixed_dark_time));
            println!("│ Kitty Light тема: {:<37} │", config.kitty.theme_light.as_deref().unwrap_or("-"));
            println!("│ Kitty Dark тема:  {:<37} │", config.kitty.theme_dark.as_deref().unwrap_or("-"));
            println!("│ Обои (День):      {:<37} │", config.gnome.wallpaper_light.as_deref().unwrap_or("(не заданы)"));
            println!("│ Обои (Ночь):      {:<37} │", config.gnome.wallpaper_dark.as_deref().unwrap_or("(не заданы)"));
            println!("│ Файл на диске:    {:<37} │", self.config_path.display().to_string().chars().take(37).collect::<String>());
            println!("\x1b[1;37m└─────────────────────────────────────────────────────────┘\x1b[0m\n");

            println!("\x1b[1mВыберите, что хотите настроить:\x1b[0m");
            println!("  \x1b[1;36m[1]\x1b[0m 📍 Изменить координаты (Широта / Долгота или быстрые пресеты)");
            println!("  \x1b[1;36m[2]\x1b[0m ⏱️  Переключить режим (По солнцу ⟷ Фиксированное время)");
            println!("  \x1b[1;36m[3]\x1b[0m 🐱 Настроить темы для терминала Kitty");
            println!("  \x1b[1;36m[4]\x1b[0m 🖼️  Настроить обои GNOME (Светлые / Тёмные)");
            println!("  \x1b[1;36m[5]\x1b[0m 📝 Открыть config.toml в текстовом редакторе ($EDITOR)");
            println!("  \x1b[1;36m[6]\x1b[0m 🔄 Сбросить конфиг до значений по умолчанию");
            println!("  \x1b[1;31m[0]\x1b[0m ↩️  Назад в главное меню\n");

            print!("\x1b[1mВаш выбор [0-6]: \x1b[0m");
            io::stdout().flush()?;

            let choice = Self::read_line_input();
            match choice.trim() {
                "1" => {
                    self.edit_location(&mut config)?;
                }
                "2" => {
                    self.edit_schedule_mode(&mut config)?;
                }
                "3" => {
                    self.edit_kitty_themes(&mut config)?;
                }
                "4" => {
                    self.edit_wallpapers(&mut config)?;
                }
                "5" => {
                    self.open_in_editor()?;
                }
                "6" => {
                    print!("\x1b[1;33mВы уверены, что хотите сбросить настройки? (y/N): \x1b[0m");
                    io::stdout().flush()?;
                    if Self::read_line_input().trim().eq_ignore_ascii_case("y") {
                        let def = Config::default();
                        def.save(&self.config_path)?;
                        self.notify_daemon_reload();
                        println!("\x1b[1;32m✔ Конфиг сброшен до значений по умолчанию!\x1b[0m");
                        Self::pause_prompt();
                    }
                }
                "0" | "q" | "" => break,
                _ => {}
            }
        }

        Ok(())
    }

    fn edit_location(&self, config: &mut Config) -> anyhow::Result<()> {
        println!("\n\x1b[1;36m--- Выбор координат ---\x1b[0m");
        println!("Быстрые пресеты городов:");
        println!("  1. Санкт-Петербург (59.9343, 30.3351)");
        println!("  2. Москва           (55.7558, 37.6173)");
        println!("  3. Новосибирск      (55.0084, 82.9357)");
        println!("  4. Екатеринбург     (56.8389, 60.6057)");
        println!("  5. Казань           (55.8304, 49.0661)");
        println!("  6. Ввести координаты вручную");
        print!("\nВыберите вариант [1-6] (Enter для отмены): ");
        io::stdout().flush()?;

        let opt = Self::read_line_input();
        match opt.trim() {
            "1" => { config.location.latitude = 59.9343; config.location.longitude = 30.3351; }
            "2" => { config.location.latitude = 55.7558; config.location.longitude = 37.6173; }
            "3" => { config.location.latitude = 55.0084; config.location.longitude = 82.9357; }
            "4" => { config.location.latitude = 56.8389; config.location.longitude = 60.6057; }
            "5" => { config.location.latitude = 55.8304; config.location.longitude = 49.0661; }
            "6" => {
                print!("Введите широту (Latitude, например 59.9343): ");
                io::stdout().flush()?;
                if let Ok(lat) = Self::read_line_input().trim().parse::<f64>() {
                    config.location.latitude = lat;
                }
                print!("Введите долготу (Longitude, например 30.3351): ");
                io::stdout().flush()?;
                if let Ok(lon) = Self::read_line_input().trim().parse::<f64>() {
                    config.location.longitude = lon;
                }
            }
            _ => return Ok(()),
        }

        config.save(&self.config_path)?;
        self.notify_daemon_reload();
        println!("\x1b[1;32m✔ Координаты обновлены: Lat {:.4}, Lon {:.4}\x1b[0m", config.location.latitude, config.location.longitude);
        Self::pause_prompt();
        Ok(())
    }

    fn edit_schedule_mode(&self, config: &mut Config) -> anyhow::Result<()> {
        println!("\n\x1b[1;36m--- Режим работы ---\x1b[0m");
        println!("1. Solar (автоматически вычислять по положению солнца)");
        println!("2. Fixed (фиксированное время по часам)");
        print!("Выберите режим [1-2]: ");
        io::stdout().flush()?;

        match Self::read_line_input().trim() {
            "1" => {
                config.schedule.mode = ScheduleMode::Solar;
            }
            "2" => {
                config.schedule.mode = ScheduleMode::Fixed;
                print!("Время светлой темы (HH:MM, сейчас: {}): ", config.schedule.fixed_light_time);
                io::stdout().flush()?;
                let lt = Self::read_line_input();
                if !lt.trim().is_empty() {
                    config.schedule.fixed_light_time = lt.trim().to_string();
                }

                print!("Время тёмной темы (HH:MM, сейчас: {}): ", config.schedule.fixed_dark_time);
                io::stdout().flush()?;
                let dt = Self::read_line_input();
                if !dt.trim().is_empty() {
                    config.schedule.fixed_dark_time = dt.trim().to_string();
                }
            }
            _ => return Ok(()),
        }

        config.save(&self.config_path)?;
        self.notify_daemon_reload();
        println!("\x1b[1;32m✔ Режим расписания успешно сохранен!\x1b[0m");
        Self::pause_prompt();
        Ok(())
    }

    fn edit_kitty_themes(&self, config: &mut Config) -> anyhow::Result<()> {
        println!("\n\x1b[1;36m--- Настройка тем Kitty ---\x1b[0m");
        print!("Путь к светлой теме (текущий: {}): ", config.kitty.theme_light.as_deref().unwrap_or("-"));
        io::stdout().flush()?;
        let l = Self::read_line_input();
        if !l.trim().is_empty() {
            config.kitty.theme_light = Some(l.trim().to_string());
        }

        print!("Путь к тёмной теме (текущий: {}): ", config.kitty.theme_dark.as_deref().unwrap_or("-"));
        io::stdout().flush()?;
        let d = Self::read_line_input();
        if !d.trim().is_empty() {
            config.kitty.theme_dark = Some(d.trim().to_string());
        }

        config.save(&self.config_path)?;
        self.notify_daemon_reload();
        println!("\x1b[1;32m✔ Пути к темам Kitty сохранены!\x1b[0m");
        Self::pause_prompt();
        Ok(())
    }

    fn edit_wallpapers(&self, config: &mut Config) -> anyhow::Result<()> {
        println!("\n\x1b[1;36m--- Настройка обоев рабочего стола GNOME ---\x1b[0m");
        print!("Путь к дневным обоям (текущий: {}): ", config.gnome.wallpaper_light.as_deref().unwrap_or("(нет)"));
        io::stdout().flush()?;
        let l = Self::read_line_input();
        if !l.trim().is_empty() {
            config.gnome.wallpaper_light = Some(l.trim().to_string());
        }

        print!("Путь к ночным обоям (текущий: {}): ", config.gnome.wallpaper_dark.as_deref().unwrap_or("(нет)"));
        io::stdout().flush()?;
        let d = Self::read_line_input();
        if !d.trim().is_empty() {
            config.gnome.wallpaper_dark = Some(d.trim().to_string());
        }

        config.save(&self.config_path)?;
        self.notify_daemon_reload();
        println!("\x1b[1;32m✔ Пути к обоям сохранены!\x1b[0m");
        Self::pause_prompt();
        Ok(())
    }

    fn open_in_editor(&self) -> anyhow::Result<()> {
        let editor = std::env::var("EDITOR")
            .or_else(|_| std::env::var("VISUAL"))
            .unwrap_or_else(|_| "nano".to_string());

        println!("\x1b[1;36mЗапуск редактора '{}' для файла {:?}...\x1b[0m", editor, self.config_path);
        let _ = Command::new(&editor)
            .arg(&self.config_path)
            .status();

        self.notify_daemon_reload();
        Ok(())
    }

    fn notify_daemon_reload(&self) {
        // Send SIGHUP to running solard daemon to hot-reload config
        let state_path = if let Some(dirs) = directories::BaseDirs::new() {
            dirs.data_local_dir().join("solard").join("state.json")
        } else {
            PathBuf::from("/tmp/solard.state")
        };

        if state_path.exists() {
            if let Ok(c) = std::fs::read_to_string(&state_path) {
                if let Some(pid_str) = c.split("\"pid\":").nth(1) {
                    if let Ok(pid) = pid_str.trim().trim_end_matches('}').trim().parse::<i32>() {
                        unsafe {
                            let _ = libc::kill(pid, libc::SIGHUP);
                        }
                        log::info!("Sent SIGHUP to solard daemon PID {}", pid);
                    }
                }
            }
        }
    }

    fn read_line_input() -> String {
        let stdin = io::stdin();
        let mut line = String::new();
        let _ = stdin.lock().read_line(&mut line);
        line
    }

    fn pause_prompt() {
        println!("\n\x1b[90mНажмите Enter для продолжения...\x1b[0m");
        let mut buf = String::new();
        let _ = io::stdin().read_line(&mut buf);
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
