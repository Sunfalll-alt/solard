# ☀️🌙 Solard — Демон автоматического переключения тем оформления

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue.svg?style=flat-square)](LICENSE)
[![CI](https://github.com/Sunfalll-alt/solard/actions/workflows/ci.yml/badge.svg?style=flat-square)](https://github.com/Sunfalll-alt/solard/actions)
[![Arch Linux](https://img.shields.io/badge/Arch%20Linux-AUR-1793d1.svg?style=flat-square&logo=arch-linux)](https://aur.archlinux.org/packages/solard)
[![Wayland / X11](https://img.shields.io/badge/display-Wayland%20%7C%20X11-purple.svg?style=flat-square)]()

**Solard** — это легковесный, высокопроизводительный демон на **Rust** для **любых Linux-дистрибутивов** (Arch Linux, Ubuntu, Debian, Linux Mint, Fedora и др.), синхронно переключающий оформление всей системы по положению солнца (восход/закат по координатам) или по расписанию.

Автоматически определяет ваше рабочее окружение (**GNOME**, **Cinnamon / Linux Mint**, **KDE Plasma**, **XFCE**, **MATE**), тип сессии (**Wayland** или **X11**), терминалы (**Kitty**, **Alacritty**, **Foot**) и редакторы кода (**VS Code**, **Neovim**).

---

## 🖥 Интерфейс и демонстрация

```text
  ███████╗ ██████╗ ██╗      █████╗ ██████╗ ██████╗ 
  ██╔════╝██╔═══██╗██║     ██╔══██╗██╔══██╗██╔══██╗
  ███████╗██║   ██║██║     ███████║██████╔╝██║  ██║
  ╚════██║██║   ██║██║     ██╔══██╗██╔══██╗██║  ██║
  ███████║╚██████╔╝███████╗██║  ██║██║  ██║██████╔╝
  ╚══════╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═════╝ 
       ☀️  SOLAR THEME DAEMON FOR GNOME & KITTY  🌙

┌─────────────────── ТЕКУЩИЙ СТАТУС ────────────────────┐
│ Система:     Arch Linux (GNOME)                       │
│ Сессия:      Wayland                                  │
│ Демон:       🟢 Запущен (PID 1420)                     │
│ Пауза:       Не активна                               │
│ Автозапуск:  🟢 Включен (solard.service)              │
│ Тема сейчас: 🌙 Темная (Night)                         │
│ Солнце:      ☀️ 06:49:09  /  🌙 17:43:01              │
│ Луна:        🌑 Новолуние (1%), Moonset at 16:59      │
│ Следующее:   ☀️ Восход -> Светлая                     │
│ Таймер:      07h 12m 45s                              │
└───────────────────────────────────────────────────────┘

┌───────────────── ДЕЙСТВИЯ И УПРАВЛЕНИЕ ───────────────┐
│ > [1] Включить светлую тему вручную (Light)           │
│   [2] Включить темную тему вручную (Dark)             │
│   [3] Инвертировать тему прямо сейчас (Toggle)        │
│   [4] ⏸️  Приостановить авто-смену на 2 часа           │
│   [5] Запустить фоновый демон solard                  │
│   [6] Остановить фоновый демон                        │
│   [7] Включить автозапуск при входе (Systemd)         │
│   [8] 🔍 Автоопределение координат по IP              │
│   [9] Редактировать конфигурацию (~/.config/solard)   │
│   [0] 📅 Астрономический календарь (7 дней)            │
│   [Q] Выход                                           │
└───────────────────────────────────────────────────────┘
```

---

## 🚀 Возможности

1. **Точный астрономический расчет (NOAA Solar Algorithm)**:
   - Вычисляет точное время восхода и заката для заданных координат (широта и долгота).
   - Поддерживает официальный восход/закат (зенит 90.83°) или гражданские сумерки (96.0°).
   - Смещение по времени (например, переключить за 15 минут до заката).
   - Корректная обработка полярного дня и полярной ночи.
   - Альтернативный режим фиксированного времени (например, день с 07:30, ночь с 20:30).
   - Расчет фаз Луны (новолуние, четверти, полнолуние с % освещенности) и точного времени её захода.

2. **Автоопределение координат по IP**:
   - Быстрое определение ваших координат, города, страны и часового пояса без ручного ввода (`solard locate` или в меню).

3. **Режим паузы / ингибирования (Pause / Inhibit)**:
   - Приостановка автоматической смены тем на время презентаций, просмотра фильмов или игр (`solard pause 2h`, `solard resume` или через меню).

4. **Системные уведомления**:
   - Нативные уведомления через `notify-send` при переключении темы с информацией о фазе Луны, проценте освещенности и времени дня/ночи.

5. **Поддержка рабочих окружений (DE) и сессий Wayland / X11**:
   - **GNOME** (Ubuntu, Debian, Fedora, Arch): переключение `color-scheme` (Portal / libadwaita), `gtk-theme`, обоев и аппаратного Wayland Night Light.
   - **Cinnamon** (Linux Mint): переключение `org.cinnamon.desktop.interface` (GTK, темы оболочки Cinnamon, обоев и color-scheme).
   - **KDE Plasma** (Kubuntu, Debian, Arch): вызовы `plasma-apply-colorscheme`, `plasma-apply-lookandfeel`, обоев.
   - **XFCE** (Xubuntu, Mint XFCE): темы через `xfconf-query` (xsettings и xfce4-desktop).
   - **MATE** (Ubuntu MATE, Mint MATE): `org.mate.interface` и фон рабочего стола.

6. **Терминалы (Kitty, Alacritty, Foot)**:
   - **Kitty**: атомарное обновление симлинка `current-theme.conf` и мгновенный `SIGUSR1` релоад без перезапуска терминала.
   - **Alacritty**: атомарное переключение файла конфигурации.
   - **Foot**: атомарное переключение темы и сигнал `SIGUSR1` для мгновенного обновления.

7. **Редакторы кода (VS Code, Neovim)**:
   - **VS Code / VSCodium / Code OSS**: автоматическая замена `workbench.colorTheme` в `settings.json`.
   - **Neovim**: передача команды `set background=light/dark` на лету через RPC UNIX-сокеты во все запущенные сессии `nvim`.

8. **Браузеры (Firefox, Chrome, Chromium)**:
   - Через FreeDesktop Appearance Portal (`xdg-desktop-portal`) браузеры на лету адаптируют тему сайтов (`@media (prefers-color-scheme: dark/light)`).

9. **Qt (Kvantum & qt6ct)**:
   - Синхронное переключение тем Kvantum (`kvantummanager --set <theme>`).
   - Синхронизация цветовой палитры в `qt6ct.conf`.

10. **Gammastep / Redshift**:
    - Управление температурой экрана (`gammastep -O <temp>` или через `gammastep.service`).

11. **Слепок настроек и безопасный откат (Snapshot & Restore)**:
    - При первом старте сохраняет исходный профиль системы, а при `solard stop` автоматически восстанавливает все параметры до единого.

---

## 📦 Установка

### 1. Arch Linux (AUR)
Пакет доступен в Arch User Repository:
```bash
# С помощью yay
yay -S solard

# Или с помощью paru
paru -S solard
```

Либо ручная сборка из `PKGBUILD`:
```bash
git clone https://github.com/Sunfalll-alt/solard.git
cd solard
makepkg -si
```

### 2. Сборка из исходников (Ubuntu, Debian, Mint, Fedora, Arch)

#### Установка зависимостей:
- **Arch / Manjaro**:
  ```bash
  sudo pacman -S --needed base-devel rust cargo curl libnotify
  ```
- **Ubuntu / Debian / Linux Mint**:
  ```bash
  sudo apt update && sudo apt install -y build-essential cargo rustc curl libnotify-bin
  ```
- **Fedora**:
  ```bash
  sudo dnf install -y gcc cargo rust curl libnotify
  ```

#### Сборка и установка:
```bash
git clone https://github.com/Sunfalll-alt/solard.git
cd solard
cargo build --release
cargo install --path .
```
Убедитесь, что `~/.cargo/bin` добавлен в ваш `$PATH`:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

---

## 🛠 Команды управления (CLI)

```bash
# Интерактивное TUI-меню со стрелочками и редактором конфигурации
solard
solard menu

# Автоматическое определение координат по IP и запись в конфиг
solard locate

# Приостановить авто-смену тем (по умолчанию 2 часа, можно 30m, 4h, 1d)
solard pause 2h
solard resume      # Возобновить смену тем

# Текущий статус, расписание солнца, фазы луны и таймер
solard status

# Астрономическая таблица восходов и заходов луны на 7 дней
solard calc

# Переключение тем вручную
solard set light   # Включить день
solard set dark    # Включить ночь
solard toggle      # Инвертировать текущую тему

# Управление автозапуском Systemd
solard enable      # Включить службу при входе в систему
solard disable     # Отключить службу

# Остановка демона с возвратом исходных настроек системы
solard stop
```

---

## ⚙️ Конфигурация (`~/.config/solard/config.toml`)

Инициализация файла настроек по умолчанию:
```bash
solard init-config
```

### Пример конфигурации:
```toml
[location]
latitude = 59.9343
longitude = 30.3351
zenith = 90.8333
sunrise_offset_minutes = 0
sunset_offset_minutes = 0

[schedule]
mode = "solar" # "solar" или "fixed"
fixed_light_time = "07:30"
fixed_dark_time = "20:30"
check_interval_secs = 60

[gnome]
enabled = true
set_color_scheme = true
gtk_theme_light = "Adwaita"
gtk_theme_dark = "Adwaita-dark"
# wallpaper_light = "~/Pictures/day.jpg"
# wallpaper_dark = "~/Pictures/night.jpg"
control_gnome_night_light = true

[kitty]
enabled = true
theme_light = "~/.config/kitty/themes/solarized-light.conf"
theme_dark = "~/.config/kitty/themes/solarized-dark.conf"
symlink_path = "~/.config/kitty/current-theme.conf"
send_sigusr1 = true

[alacritty]
enabled = false
theme_light = "~/.config/alacritty/themes/light.toml"
theme_dark = "~/.config/alacritty/themes/dark.toml"
symlink_path = "~/.config/alacritty/theme.toml"

[foot]
enabled = false
theme_light = "~/.config/foot/themes/light.ini"
theme_dark = "~/.config/foot/themes/dark.ini"
symlink_path = "~/.config/foot/theme.ini"

[vscode]
enabled = false
theme_light = "Default Light Modern"
theme_dark = "Default Dark Modern"

[neovim]
enabled = false

[notifications]
enabled = true

[qt]
enabled = true
# kvantum_theme_light = "KvFlatLight"
# kvantum_theme_dark = "KvFlatDark"

[gammastep]
enabled = false
temp_day = 6500
temp_night = 3500
method = "oneshot"

[hooks]
# on_light = "notify-send 'Solard' 'Day mode activated'"
# on_dark = "notify-send 'Solard' 'Night mode activated'"
```

---

## 🔄 Системная служба Systemd (Autostart)

Для автозапуска при входе в систему достаточно выполнить:
```bash
solard enable
```
Команда автоматически создаст и активирует пользовательский юнит `solard.service`.

Проверка статуса:
```bash
systemctl --user status solard.service
journalctl --user -u solard.service -f
```

---

## 💡 Сигналы процесса

- `SIGUSR1` — мгновенное переключение текущей темы (Light ⟷ Dark).
- `SIGHUP` — перезагрузка файла конфигурации `config.toml` без перезапуска демона.
- `SIGINT` / `SIGTERM` — корректная остановка и восстановление системного снапшота.

---

## 📄 Лицензия

MIT License (c) 2026 Sunfalll-alt
