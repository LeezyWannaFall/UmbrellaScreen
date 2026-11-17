use gtk::{glib, prelude::*};

fn stylesheet(json: &str) -> Option<String> {
    let palette: serde_json::Value = serde_json::from_str(json).ok()?;
    let mode = palette.get("mode")?.as_str()?;
    let colors = palette.get(mode)?;
    let color = |name: &str| -> Option<&str> {
        let value = colors.get(name)?.as_str()?;
        (value.len() == 7
            && value.starts_with('#')
            && value[1..].bytes().all(|b| b.is_ascii_hexdigit()))
        .then_some(value)
    };
    let background = color("background")?;
    let surface = color("surface")?;
    let text = color("text")?;
    let muted = color("muted")?;
    let accent = color("accent")?;
    let luminance: f64 = [1, 3, 5]
        .into_iter()
        .zip([0.2126, 0.7152, 0.0722])
        .map(|(i, weight)| {
            let channel = u8::from_str_radix(&accent[i..i + 2], 16).unwrap() as f64 / 255.;
            weight
                * if channel <= 0.04045 {
                    channel / 12.92
                } else {
                    ((channel + 0.055) / 1.055).powf(2.4)
                }
        })
        .sum();
    let on_accent = if luminance > 0.179 {
        "#000000"
    } else {
        "#ffffff"
    };
    Some(format!(
        r#"
        @define-color umbrella_handle_surface {surface};
        .umbrella .selection-canvas {{ color: {accent}; }}
        window.umbrella {{ background: {background}; color: {text}; }}
        .umbrella .editor-panel {{ background: alpha({surface}, 0.97); color: {text};
            border: 1px solid alpha({muted}, 0.22); border-radius: 20px;
            padding: 6px; box-shadow: 0 6px 20px alpha(black, 0.22); }}
        .umbrella button {{ color: {text}; background: transparent; background-image: none;
            border: none; box-shadow: none; text-shadow: none; }}
        .umbrella .editor-panel button {{ min-width: 32px; min-height: 32px;
            padding: 0; margin: 0; border-radius: 14px; }}
        .umbrella button:hover {{ background: alpha({muted}, 0.12); }}
        .umbrella button:checked {{ background: alpha({accent}, 0.25); color: {text}; }}
        .umbrella button.suggested-action {{ background: {accent}; color: {on_accent}; }}
        .umbrella button drawingarea {{ color: inherit; }}
        .umbrella popover contents {{ background: {surface}; color: {text};
            border: 1px solid alpha({muted}, 0.22); border-radius: 18px; padding: 10px; }}
        .umbrella entry {{ background: {background}; color: {text}; caret-color: {text}; }}
        .umbrella tooltip {{ background: {surface}; color: {text}; border-radius: 10px; }}
    "#
    ))
}

pub fn install(window: &gtk::ApplicationWindow) {
    window.add_css_class("umbrella");
    let path = glib::user_config_dir().join("quickshell/atelier/colors.json");
    let fallback = r##"{"mode":"dark","dark":{"background":"#131316","surface":"#1f1f22","text":"#e4e1e6","muted":"#c6c5d0","accent":"#717eb2"}}"##;
    let mut previous = std::fs::read_to_string(&path).unwrap_or_default();
    let provider = gtk::CssProvider::new();
    provider
        .load_from_data(&stylesheet(&previous).unwrap_or_else(|| stylesheet(fallback).unwrap()));
    // User gtk.css from Matugen may otherwise override application button surfaces.
    gtk::style_context_add_provider_for_display(
        &WidgetExt::display(window),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_USER + 1,
    );
    let weak = window.downgrade();
    glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
        let Some(window) = weak.upgrade() else {
            return glib::ControlFlow::Break;
        };
        if let Ok(current) = std::fs::read_to_string(&path)
            && current != previous
            && let Some(css) = stylesheet(&current)
        {
            provider.load_from_data(&css);
            previous = current;
            window.queue_draw();
        }
        glib::ControlFlow::Continue
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selects_mode_and_contrasting_icon_color() {
        let json = r##"{"mode":"light","light":{"background":"#ffffff","surface":"#eeeeee","text":"#111111","muted":"#555555","accent":"#106396"}}"##;
        let css = stylesheet(json).unwrap();
        assert!(css.contains("background: #106396; color: #ffffff"));
        assert!(css.contains("color: #111111"));
        assert!(
            stylesheet(&json.replace("#106396", "#eeeeee"))
                .unwrap()
                .contains("background: #eeeeee; color: #000000")
        );
        assert!(stylesheet(&json.replace("#111111", "invalid")).is_none());
        assert!(stylesheet("{").is_none());
    }
}
