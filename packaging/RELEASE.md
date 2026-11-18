Screenshot capture and editing directly over the Hyprland desktop.

- Region selection with move and resize handles.
- Pen, line, arrow, shapes, text, and blur.
- Multiple object selection, transforms, undo and redo.
- PNG saving, clipboard copying, and notifications with screenshot previews.
- Keyboard shortcuts and Matugen colors through the Quickshell Atelier palette.

## Install on Arch Linux (x86_64)

Download the `.pkg.tar.zst` asset and install it with pacman.

```sh
sudo pacman -U ./umbrellascreen-0.1.0-1-x86_64.pkg.tar.zst
```

Run `umbrellascreen` inside a Hyprland session. For a Print shortcut, use `/usr/bin/umbrellascreen` in your Hyprland binding.

Screenshots are saved to `~/Pictures/screenshots/`. Press Enter or the checkmark to save, copy, and close. A running notification service is required for notifications.

This package targets current Arch Linux on x86_64 and requires Hyprland. It uses the system GTK4 and gtk4-layer-shell libraries. SHA256 checksums are included in `SHA256SUMS`.
