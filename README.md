# UmbrellaScreen

[![CI](https://github.com/LeezyWannaFall/UmbrellaScreen/actions/workflows/ci.yml/badge.svg)](https://github.com/LeezyWannaFall/UmbrellaScreen/actions/workflows/ci.yml)

Скриншоты с редактированием прямо поверх рабочего стола **Arch Linux + Hyprland**.

Выделите область мышью, добавьте пометки и нажмите **Enter**. Снимок сохранится в `~/Pictures/screenshots/`, скопируется в буфер, редактор закроется.

## Установка

```sh
sudo pacman -S --needed base-devel git rust pkgconf gtk4 gtk4-layer-shell grim wl-clipboard

git clone https://github.com/LeezyWannaFall/UmbrellaScreen.git
cd UmbrellaScreen
cargo build --release --locked

install -Dm755 target/release/umbrellascreen ~/.local/bin/umbrellascreen
```

## Запуск

В терминале внутри сессии Hyprland выполните

```sh
~/.local/bin/umbrellascreen
```

Для запуска по **SUPER + ]** добавьте привязку в используемый конфиг Hyprland.

**Lua · `hyprland.lua` или подключённый файл клавиш**

```lua
hl.bind("SUPER + code:35", hl.dsp.exec_cmd("~/.local/bin/umbrellascreen"))
```

**Hyprlang · `hyprland.conf`**

```ini
bind = SUPER, code:35, exec, ~/.local/bin/umbrellascreen
```

Примените конфигурацию командой `hyprctl reload`. Физическая клавиша `code:35` соответствует `]` / `ъ` и работает независимо от раскладки.

## Управление

| Действие | Клавиша или жест |
| --- | --- |
| Сохранить, скопировать и закрыть | **Enter** |
| Сохранить PNG | **Ctrl+S** |
| Копировать PNG | **Ctrl+C** |
| Отменить / повторить | **Ctrl+Z** / **Ctrl+Shift+Z** |
| Удалить выбранный объект | **Delete** / **Backspace** |
| Закрыть без сохранения | **Esc** |
| Переместить область снимка | Потянуть рамку |
| Переместить объект | Инструмент выбора → перетащить объект |
| Изменить размер / повернуть объект | Маркеры по краям / маркер над объектом |

Панель инструментов содержит перо, стрелку, прямоугольник, эллипс, текст и выбор объектов. Цвет и толщина выбираются во всплывающих меню. Наведите курсор на иконку, чтобы увидеть подсказку.

Для текста выберите **T**, нажмите внутри области и введите надпись. Enter в поле подтверждает текст. Следующий Enter сохраняет и копирует снимок. Кнопка с галочкой сохраняет и копирует, оставляя редактор открытым.

## Особенности

- Работает через Wayland layer-shell. Содержимое экрана фиксируется при запуске.
- Выделение доступно в пределах одного монитора. Аннотации перемещаются вместе с рамкой.
- PNG сохраняет разрешение выбранной области. Имена файлов уникальны.
- При ошибке сохранения или копирования редактор остаётся открытым и показывает сообщение.

## Разработка

Стек проекта — Rust, GTK4, Cairo/Pango и gtk4-layer-shell. Захват выполняет `grim`, буфер обмена обслуживает `wl-copy`.

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --release --locked
```

GitHub Actions выполняет эти проверки в контейнере Arch Linux на push в `main` и на pull request. Проверка жестов, фокуса и плавности проводится в живой сессии Hyprland.
