# ☀️🌙 Solard — Демон автоматического переключения светлой/темной темы

**Solard** — это легковесный, быстрый и модульный демон на **Rust** для **Arch Linux (GNOME Wayland)**, синхронно переключающий оформление системы по положению солнца (восход/закат по координатам) или по заданному расписанию.

---

## 🚀 Возможности

1. **Точный астрономический расчет (NOAA Solar Algorithm)**:
   - Вычисляет точное время восхода и заката для заданных координат (широта и долгота).
   - Поддерживает официальный восход/закат (зенит 90.83°) или гражданские сумерки (96.0°).
   - Поддерживает смещение по времени (например, переключить за 15 минут до заката).
   - Корректно обрабатывает полярный день и полярную ночь.
   - Поддерживает альтернативный режим фиксированного времени (например, день с 07:30, ночь с 20:30).

2. **GNOME & Сессия Wayland**:
   - Мгновенное переключение `org.gnome.desktop.interface color-scheme` (`prefer-light` / `prefer-dark`).
   - Переключение `gtk-theme` для классических приложений GTK 3/4 (например, Adwaita / Adwaita-dark).
   - Автоматическая смена дневных и ночных обоев (`picture-uri` и `picture-uri-dark`).
   - Интеграция со встроенным **GNOME Night Light** (аппаратный фильтр синего цвета под Wayland).

3. **Браузеры (Firefox, Chrome, Chromium)**:
   - В среде GNOME Wayland браузеры слушают FreeDesktop Appearance Portal (`xdg-desktop-portal`). Переключение системной `color-scheme` автоматически переключает темы сайтов (`@media (prefers-color-scheme: dark/light)`) и интерфейс браузера без необходимости сторонних расширений.

4. **Терминал Kitty**:
   - Атомарное обновление символической ссылки `~/.config/kitty/current-theme.conf`.
   - Мгновенная перезагрузка палитры во всех открытых окнах Kitty без перезапуска терминала с помощью сигнала `SIGUSR1` (`pkill -SIGUSR1 kitty`).
   - Опциональная поддержка Kitty Remote Control (`kitty @ set-colors`).

5. **Qt (Kvantum & qt6ct)**:
   - Синхронное переключение тем Kvantum (`kvantummanager --set <theme>` или `kvconfig`).
   - Синхронизация цветовой палитры в `qt6ct.conf`.
   - Нативные Qt приложения на GNOME Wayland (через `qgnomeplatform` / `QT_QPA_PLATFORMTHEME=gnome`) также нативно подхватывают тему через portal.

6. **Gammastep / Redshift**:
   - Опциональное управление температурой экрана (`gammastep -O <temp>` или управление через systemd-юнит `gammastep.service`).

7. **Пользовательские хуки (Hooks)**:
   - Запуск произвольных скриптов/команд при переходе в режим дня и ночи с передачей переменных окружения `$SOLARD_MODE`, `$SOLARD_LATITUDE`, `$SOLARD_LONGITUDE`.

---

## 📦 Установка на Arch Linux

### 1. Установка зависимостей
```bash
sudo pacman -S rust cargo kitty libnotify
```

### 2. Сборка и установка бинарника
```bash
git clone https://github.com/Sunfalll-alt/solard.git
cd solard
cargo build --release
cargo install --path .
```
Бинарник установится в `~/.cargo/bin/solard`. Убедитесь, что `~/.cargo/bin` добавлен в вашу переменную `$PATH` (обычно в `~/.bashrc` или `~/.zshrc`):
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

---

## ⚙️ Настройка

### 1. Создание конфигурационного файла
Выполните команду для создания файла конфигурации по умолчанию:
```bash
solard init-config
```
Конфигурация сохранится в `~/.config/solard/config.toml`.

### 2. Редактирование `~/.config/solard/config.toml`
Пример файла конфигурации:

```toml
[location]
# Ваши географические координаты (Широта и Долгота)
# Пример: Санкт-Петербург
latitude = 59.9343
longitude = 30.3351

# 90.8333 = восход/закат, 96.0 = сумерки
zenith = 90.8333

# Смещение (в минутах) относительно заката/восхода
sunrise_offset_minutes = 0
sunset_offset_minutes = 0

[schedule]
# "solar" (по солнцу) или "fixed" (по часам)
mode = "solar"
fixed_light_time = "07:30"
fixed_dark_time = "20:30"
check_interval_secs = 60

[gnome]
enabled = true
set_color_scheme = true
gtk_theme_light = "Adwaita"
gtk_theme_dark = "Adwaita-dark"

# Пути к обоям:
# wallpaper_light = "~/Pictures/day.jpg"
# wallpaper_dark = "~/Pictures/night.jpg"

# Включение аппаратного ночного света GNOME Wayland ночью:
control_gnome_night_light = true

[kitty]
enabled = true
theme_light = "~/.config/kitty/themes/solarized-light.conf"
theme_dark = "~/.config/kitty/themes/solarized-dark.conf"
symlink_path = "~/.config/kitty/current-theme.conf"
send_sigusr1 = true
use_remote_control = false

[qt]
enabled = true
# kvantum_theme_light = "KvFlatLight"
# kvantum_theme_dark = "KvFlatDark"

[gammastep]
# В GNOME Wayland рекомендуется использовать control_gnome_night_light = true
enabled = false
temp_day = 6500
temp_night = 3500
method = "oneshot"

[hooks]
# on_light = "notify-send 'Solard' 'Включена светлая тема'"
# on_dark = "notify-send 'Solard' 'Включена темная тема'"
```

### 3. Настройка Kitty Terminal
Чтобы Kitty подхватывал тему динамически:
1. Создайте папку для тем и скопируйте темы:
   ```bash
   mkdir -p ~/.config/kitty/themes
   cp examples/kitty/solarized-light.conf ~/.config/kitty/themes/
   cp examples/kitty/solarized-dark.conf ~/.config/kitty/themes/
   ```
2. В конце файла `~/.config/kitty/kitty.conf` добавьте строку:
   ```conf
   include current-theme.conf
   ```
   *Демон автоматически создает и обновляет симлинк `~/.config/kitty/current-theme.conf` и отправляет `SIGUSR1`, благодаря чему цвета обновляются прямо на лету!*

---

## 🛠 Команды управления (CLI)

- **Интерактивное консольное меню (TUI в стиле Antigravity CLI)**:
  ```bash
  solard          # или solard menu
  ```
  *(Красочное интерактивное меню с ASCII-артом, живым статусом демона и солнца, навигацией стрелками ↑/↓, клавишами 1-9 и встроенным интерактивным редактором настроек).*

- **Просмотр текущего состояния и расписания солнца**:
  ```bash
  solard status
  ```
- **Таблица восходов и закатов на ближайшие 7 дней**:
  ```bash
  solard calc
  ```
- **Принудительно переключить тему вручную**:
  ```bash
  solard set light   # Включить светлую тему
  solard set dark    # Включить темную тему
  solard toggle      # Инвертировать текущую тему
  ```
  *(Если демон запущен, команда `toggle` мгновенно отправляет ему сигнал `SIGUSR1`).*

- **Запуск демона вручную (в текущем терминале)**:
  ```bash
  solard daemon
  ```
- **Остановка запущенного демона (с восстановлением исходных настроек)**:
  ```bash
  solard stop
  ```
  *(При первом запуске демон автоматически сохраняет слепок ваших исходных настроек системы и терминала, а при вызове `solard stop` или остановке службы автоматически возвращает всё в исходное состояние).*

---

## 🔄 Настройка автозапуска через Systemd

Для запуска демона вместе со входом в сессию GNOME:

1. Скопируйте файл сервиса:
   ```bash
   mkdir -p ~/.config/systemd/user
   cp solard.service ~/.config/systemd/user/
   ```

2. Перезагрузите конфигурацию systemd и включите сервис:
   ```bash
   systemctl --user daemon-reload
   systemctl --user enable --now solard.service
   ```

3. Проверка статуса:
   ```bash
   systemctl --user status solard.service
   journalctl --user -u solard.service -f
   ```

4. Остановка и отключение автозапуска:
   ```bash
   # Временно остановить:
   systemctl --user stop solard.service

   # Отключить автозапуск:
   systemctl --user disable --now solard.service
   ```

---

## 💡 Полезные сигналы процесса

- `SIGUSR1` — мгновенное переключение текущей темы (Light ⟷ Dark).
- `SIGHUP` — перезагрузка файла конфигурации `config.toml` без перезапуска демона.
- `SIGINT` / `SIGTERM` — корректное завершение работы.
