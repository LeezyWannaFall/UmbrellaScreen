# UmbrellaScreen

English · [Русский](README.ru.md)

[![CI](https://github.com/LeezyWannaFall/UmbrellaScreen/actions/workflows/ci.yml/badge.svg)](https://github.com/LeezyWannaFall/UmbrellaScreen/actions/workflows/ci.yml)

A screenshot app for **Arch Linux with Hyprland**. Select and edit a region directly over your desktop.

Select a region with the mouse, add annotations, and press **Enter**. The PNG is saved to `~/Pictures/screenshots/`, copied to the clipboard, and the editor closes.

## Installation

Install the dependencies and build the app

```sh
sudo pacman -S --needed base-devel git rust pkgconf gtk4 gtk4-layer-shell grim wl-clipboard

git clone https://github.com/LeezyWannaFall/UmbrellaScreen.git
cd UmbrellaScreen
cargo build --release --locked
```

Copy the executable to your local applications directory

```sh
install -Dm755 target/release/umbrellascreen ~/.local/bin/umbrellascreen
```

## Launch

Run inside a Hyprland session

```sh
~/.local/bin/umbrellascreen
```

To launch with **Print**, add a binding to your active Hyprland configuration.

For `hyprland.conf`

```ini
bind = , Print, exec, ~/.local/bin/umbrellascreen
```

For `hyprland.lua`

```lua
hl.bind("Print", hl.dsp.exec_cmd("~/.local/bin/umbrellascreen"))
```

Apply the changes with `hyprctl reload`.

## Editor

- Pen, line, arrow, rectangle, ellipse, and text.
- Blur the original image within a selected rectangle.
- Color and line width controls.
- Move, resize, and rotate objects.
- Undo and redo.

Hover over a button for a tooltip. To add text, select **T**, click inside the screenshot, and press **Enter** to confirm the text.

| Action | Key or gesture |
| --- | --- |
| Save, copy, and close | **Enter** |
| Save PNG | **Ctrl+S** |
| Copy PNG | **Ctrl+C** |
| Undo / redo | **Ctrl+Z** / **Ctrl+Shift+Z** |
| Delete selected object | **Delete** / **Backspace** |
| Close | **Esc** |
| Move screenshot region | Drag its border |
| Move an object | Choose the selection tool and drag the object |
| Resize an object | Drag a handle on its border |
| Rotate an object | Drag the handle above its border |

Ctrl shortcuts work across keyboard layouts. Standard text-field shortcuts apply while entering text. The checkmark button saves and copies the screenshot, then closes the editor.

## Saving

The `~/Pictures/screenshots/` directory is created automatically. Files have unique names and preserve the selected region's resolution. Saving sends a notification with a thumbnail and the file location. Notifications require a running notification service.

The desktop image is captured when the app launches. Each screenshot region is limited to one monitor. If saving or copying fails, the editor displays an error and stays open.
