//! Ginir Rust shell: first slice, an iNiR-styled top bar.
//!
//! The QML shell stays the reference for the full feature set. This binary
//! replaces the bar first: workspaces and active window from niri, clock,
//! weather from iNiR's cache, battery and network status. It renders through
//! GTK4 + wlr-layer-shell with the same palette and font as iNiR.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

use chrono::Local;
use gtk4::prelude::*;
use gtk4::{
    pango, Align, Application, ApplicationWindow, Box, CenterBox, CssProvider, Image, Label,
    Orientation,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use serde_json::Value;

const APP_ID: &str = "org.ginir.Shell.Rs";

struct Theme {
    background: String,
    foreground: String,
    accent: String,
    on_accent: String,
    font: String,
    height: i32,
}

fn main() -> gtk4::glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let theme = Rc::new(load_theme());
    install_css(&theme);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Ginir")
        .build();
    window.init_layer_shell();
    window.set_namespace(Some("ginir-shell-rs"));
    window.set_layer(Layer::Top);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.set_exclusive_zone(theme.height + 8);
    window.set_default_size(1600, theme.height + 8);
    window.set_decorated(false);
    window.set_resizable(false);

    let root = CenterBox::new();
    root.set_widget_name("bar-root");

    let left = Box::new(Orientation::Horizontal, 8);
    let workspaces = Box::new(Orientation::Horizontal, 4);
    workspaces.set_widget_name("workspaces");
    let title = Label::new(None);
    title.set_widget_name("window-title");
    title.set_ellipsize(pango::EllipsizeMode::End);
    title.set_max_width_chars(64);
    left.append(&workspaces);
    left.append(&title);

    let clock = Label::new(None);
    clock.set_widget_name("clock");
    clock.set_halign(Align::Center);

    let right = Box::new(Orientation::Horizontal, 6);
    let weather = Label::new(None);
    weather.set_widget_name("weather");
    let battery_icon = Image::from_icon_name("battery-good-symbolic");
    battery_icon.set_pixel_size(16);
    battery_icon.add_css_class("status-icon");
    let battery = Label::new(None);
    battery.add_css_class("status");
    let network = Image::from_icon_name("network-wireless-symbolic");
    network.set_pixel_size(16);
    network.add_css_class("status-icon");
    right.append(&weather);
    right.append(&battery_icon);
    right.append(&battery);
    right.append(&network);

    root.set_start_widget(Some(&left));
    root.set_center_widget(Some(&clock));
    root.set_end_widget(Some(&right));
    window.set_child(Some(&root));
    window.present();

    let update = {
        let theme = theme.clone();
        let workspaces = workspaces.clone();
        let title = title.clone();
        let clock = clock.clone();
        let weather = weather.clone();
        let battery = battery.clone();
        let battery_icon = battery_icon.clone();
        let network = network.clone();
        move || {
            update_clock(&clock);
            update_workspaces(&workspaces, &theme);
            update_title(&title);
            update_weather(&weather);
            update_battery(&battery, &battery_icon);
            update_network(&network);
        }
    };
    update();
    gtk4::glib::timeout_add_seconds_local(1, move || {
        update();
        gtk4::glib::ControlFlow::Continue
    });
}

fn install_css(theme: &Theme) {
    let provider = CssProvider::new();
    provider.load_from_data(&css(theme));
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn css(theme: &Theme) -> String {
    let Theme {
        background,
        foreground,
        accent,
        on_accent,
        font,
        height,
    } = theme;
    format!(
        r#"
window {{ background: transparent; }}
#bar-root {{
  background-color: alpha({background}, 0.88);
  border-radius: 14px;
  margin: 4px 8px;
  padding: 0 14px;
  min-height: {height}px;
  color: {foreground};
  font-family: "{font}";
  font-size: 14px;
}}
#workspaces {{ margin-right: 4px; }}
.workspace {{
  min-width: 22px;
  min-height: 22px;
  padding: 1px 6px;
  border-radius: 999px;
  background-color: alpha({foreground}, 0.08);
  color: alpha({foreground}, 0.72);
  font-size: 13px;
}}
.workspace.active {{
  background-color: {accent};
  color: {on_accent};
  font-weight: 600;
}}
.workspace.urgent {{ background-color: #fb4934; color: #1d2021; }}
#window-title {{ font-size: 14px; color: alpha({foreground}, 0.86); }}
#clock {{ font-size: 16px; font-weight: 600; color: {foreground}; }}
.status {{ font-size: 13px; color: alpha({foreground}, 0.8); }}
.status-icon {{ color: alpha({foreground}, 0.8); }}
#weather {{ color: alpha({foreground}, 0.9); margin-right: 4px; }}
"#
    )
}

fn load_theme() -> Theme {
    let home = home();
    let colors = load_colors(&home);
    let config = load_config(&home);
    let pick = |key: &str, fallback: &str| -> String {
        colors
            .as_ref()
            .and_then(|colors| colors.get(key))
            .and_then(Value::as_str)
            .filter(|raw| is_hex(raw))
            .unwrap_or(fallback)
            .to_string()
    };
    let font = config
        .as_ref()
        .and_then(|config| config.pointer("/appearance/typography/mainFont"))
        .and_then(Value::as_str)
        .unwrap_or("JetBrainsMono Nerd Font")
        .to_string();
    let height = config
        .as_ref()
        .and_then(|config| config.pointer("/material/bar/height"))
        .and_then(Value::as_i64)
        .unwrap_or(40)
        .clamp(28, 80) as i32;
    Theme {
        background: pick("surface", "#1c1e19"),
        foreground: pick("on_surface", "#c6c6be"),
        accent: pick("primary", "#84a955"),
        on_accent: pick("on_primary", "#12190a"),
        font,
        height,
    }
}

fn load_colors(home: &Path) -> Option<Value> {
    for path in [
        home.join(".local/state/quickshell/user/generated/colors.json"),
        home.join(".local/state/quickshell/user/generated/app-palette.json"),
        home.join(".local/state/ginir/quickshell/user/generated/colors.json"),
    ] {
        if let Some(value) = std::fs::read(path)
            .ok()
            .and_then(|raw| serde_json::from_slice::<Value>(&raw).ok())
        {
            return Some(value);
        }
    }
    None
}

fn load_config(home: &Path) -> Option<Value> {
    for path in [
        home.join(".config/illogical-impulse/config.json"),
        home.join(".config/ginir/illogical-impulse/config.json"),
    ] {
        if let Some(value) = std::fs::read(path)
            .ok()
            .and_then(|raw| serde_json::from_slice::<Value>(&raw).ok())
        {
            return Some(value);
        }
    }
    None
}

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn update_clock(clock: &Label) {
    clock.set_text(&Local::now().format("%a, %d/%m  %H:%M").to_string());
}

fn update_workspaces(workspaces: &Box, _theme: &Theme) {
    while let Some(child) = workspaces.first_child() {
        workspaces.remove(&child);
    }
    let Some(value) = niri_json(&["workspaces"]) else {
        return;
    };
    let Some(list) = value.as_array() else {
        return;
    };
    let mut items: Vec<&Value> = list.iter().collect();
    items.sort_by_key(|item| item.get("idx").and_then(Value::as_i64).unwrap_or(0));
    for item in items {
        let Some(idx) = item.get("idx").and_then(Value::as_i64) else {
            continue;
        };
        let label = Label::new(Some(&idx.to_string()));
        label.add_css_class("workspace");
        label.set_size_request(22, 22);
        label.set_halign(Align::Center);
        label.set_valign(Align::Center);
        let active = item
            .get("is_active")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            || item
                .get("is_focused")
                .and_then(Value::as_bool)
                .unwrap_or(false);
        if active {
            label.add_css_class("active");
        }
        if item
            .get("is_urgent")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            label.add_css_class("urgent");
        }
        workspaces.append(&label);
    }
}

fn update_title(title: &Label) {
    let Some(value) = niri_json(&["focused-window"]) else {
        title.set_text("");
        return;
    };
    let app_id = value.get("app_id").and_then(Value::as_str).unwrap_or("");
    let window_title = value.get("title").and_then(Value::as_str).unwrap_or("");
    if window_title.is_empty() {
        title.set_text("");
    } else if app_id.is_empty() {
        title.set_text(window_title);
    } else {
        title.set_text(&format!("{app_id} · {window_title}"));
    }
}

fn update_weather(weather: &Label) {
    let path = home().join(".local/state/quickshell/user/weather-cache.json");
    let data = std::fs::read(path)
        .ok()
        .and_then(|raw| serde_json::from_slice::<Value>(&raw).ok())
        .and_then(|value| value.get("data").cloned());
    let Some(data) = data else {
        weather.set_visible(false);
        return;
    };
    let temp = data.get("temp").and_then(Value::as_str).unwrap_or("");
    if temp.is_empty() {
        weather.set_visible(false);
        return;
    }
    weather.set_text(&format!("☁ {temp}"));
    weather.set_visible(true);
}

fn update_battery(label: &Label, icon: &Image) {
    let capacity = std::fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
        .ok()
        .and_then(|raw| raw.trim().parse::<u8>().ok());
    let status = std::fs::read_to_string("/sys/class/power_supply/BAT0/status").unwrap_or_default();
    let Some(capacity) = capacity else {
        label.set_visible(false);
        icon.set_visible(false);
        return;
    };
    label.set_visible(true);
    icon.set_visible(true);
    label.set_text(&format!("{capacity}%"));
    let icon_name = if status.trim() == "Charging" {
        "battery-charging-symbolic"
    } else if capacity >= 90 {
        "battery-full-symbolic"
    } else if capacity >= 50 {
        "battery-good-symbolic"
    } else if capacity >= 20 {
        "battery-low-symbolic"
    } else {
        "battery-caution-symbolic"
    };
    icon.set_icon_name(Some(icon_name));
}

fn update_network(network: &Image) {
    let wireless = std::fs::read_dir("/sys/class/net")
        .ok()
        .map(|entries| {
            entries
                .flatten()
                .any(|entry| entry.file_name().to_string_lossy().starts_with("wl"))
        })
        .unwrap_or(false);
    network.set_icon_name(Some(if wireless {
        "network-wireless-symbolic"
    } else {
        "network-wired-symbolic"
    }));
}

fn niri_json(args: &[&str]) -> Option<Value> {
    let output = Command::new("niri")
        .arg("msg")
        .arg("--json")
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    serde_json::from_slice(&output.stdout).ok()
}

fn is_hex(raw: &str) -> bool {
    raw.len() == 7 && raw.starts_with('#') && raw[1..].chars().all(|c| c.is_ascii_hexdigit())
}
