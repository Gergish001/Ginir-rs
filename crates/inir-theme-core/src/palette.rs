//! Palette loading from iNiR's generated JSON or kitty's theme.conf.

use std::path::Path;

use anyhow::{bail, Context, Result};
use serde_json::Value;

use crate::color::Color;

#[derive(Clone, Debug)]
pub struct Palette {
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub on_primary: Color,
    pub selection_background: Color,
    pub surface_container: Color,
    pub surface_container_high: Color,
    pub on_surface_variant: Color,
    pub outline: Color,
    pub terminal: [Color; 16],
}

impl Palette {
    /// Load iNiR's generated palette, falling back to the kitty theme.
    pub fn load(
        home: &Path,
        app_palette: Option<&Path>,
        terminal_json: Option<&Path>,
        kitty: Option<&Path>,
    ) -> Result<Self> {
        let app = app_palette.map(Path::to_path_buf).unwrap_or_else(|| {
            home.join(".local/state/quickshell/user/generated/app-palette.json")
        });
        let term = terminal_json
            .map(Path::to_path_buf)
            .unwrap_or_else(|| home.join(".local/state/quickshell/user/generated/terminal.json"));
        if app.exists() && term.exists() {
            return Self::from_inir(&app, &term);
        }
        let kitty = kitty
            .map(Path::to_path_buf)
            .unwrap_or_else(|| home.join(".config/kitty/theme.conf"));
        if kitty.exists() {
            return Self::from_kitty(&kitty);
        }
        bail!(
            "no palette found: {} / {} / {}",
            app.display(),
            term.display(),
            kitty.display()
        )
    }

    pub fn from_inir(app_palette: &Path, terminal_json: &Path) -> Result<Self> {
        let app: Value = serde_json::from_slice(
            &std::fs::read(app_palette)
                .with_context(|| format!("read {}", app_palette.display()))?,
        )
        .with_context(|| format!("parse {}", app_palette.display()))?;
        let term: Value = serde_json::from_slice(
            &std::fs::read(terminal_json)
                .with_context(|| format!("read {}", terminal_json.display()))?,
        )
        .with_context(|| format!("parse {}", terminal_json.display()))?;

        let background = pick(&app, &["surface", "background", "app_background"])
            .unwrap_or(Color::new(0x13, 0x10, 0x14));
        let foreground = pick(&app, &["on_surface", "foreground", "app_foreground"])
            .unwrap_or(Color::new(0xef, 0xeb, 0xe4));
        let accent = pick(&app, &["app_accent", "primary"]).unwrap_or(Color::new(0xb7, 0x8c, 0xda));
        let on_primary =
            pick(&app, &["app_on_accent", "on_primary", "on_accent"]).unwrap_or(background);
        let selection_background = pick(
            &app,
            &["primary_container", "surface_variant", "app_selection"],
        )
        .unwrap_or(background);
        let surface_container =
            pick(&app, &["surface_container", "app_card_bg", "app_surface"]).unwrap_or(background);
        let surface_container_high = pick(
            &app,
            &[
                "surface_container_high",
                "surface_container",
                "app_surface_elevated",
            ],
        )
        .unwrap_or(surface_container);
        let on_surface_variant =
            pick(&app, &["on_surface_variant", "app_subtext"]).unwrap_or(foreground);
        let outline =
            pick(&app, &["outline", "outline_variant"]).unwrap_or(Color::new(0x84, 0x7c, 0x84));

        let mut terminal = default_terminal();
        for (i, slot) in terminal.iter_mut().enumerate() {
            if let Some(c) = pick(&term, &[&format!("term{i}")]) {
                *slot = c;
            }
        }

        Ok(Self {
            background,
            foreground,
            accent,
            on_primary,
            selection_background,
            surface_container,
            surface_container_high,
            on_surface_variant,
            outline,
            terminal,
        })
    }

    pub fn from_kitty(path: &Path) -> Result<Self> {
        let text =
            std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        let mut map = std::collections::HashMap::new();
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let mut parts = line.split_whitespace();
            let Some(key) = parts.next() else { continue };
            let Some(value) = parts.next() else { continue };
            if value.starts_with('#') {
                map.insert(key.to_string(), value.to_string());
            }
        }
        let get =
            |key: &str| -> Option<Color> { map.get(key).and_then(|s| Color::from_hex(s).ok()) };
        let background = get("background").unwrap_or(Color::new(0x1c, 0x1e, 0x19));
        let foreground = get("foreground").unwrap_or(Color::new(0xc6, 0xc6, 0xbe));
        let accent = get("active_border_color")
            .or_else(|| get("color2"))
            .unwrap_or(Color::new(0x35, 0xb2, 0x8f));
        let on_primary = get("active_tab_foreground").unwrap_or(background);
        let selection_background = get("selection_background").unwrap_or(background);
        let surface_container = get("inactive_tab_background").unwrap_or(background);
        let surface_container_high = get("inactive_tab_background").unwrap_or(background);
        let on_surface_variant = get("color7").unwrap_or(foreground);
        let outline = get("color8").unwrap_or(Color::new(0x92, 0x93, 0x8c));
        let mut terminal = default_terminal();
        for (i, slot) in terminal.iter_mut().enumerate() {
            if let Some(c) = get(&format!("color{i}")) {
                *slot = c;
            }
        }
        Ok(Self {
            background,
            foreground,
            accent,
            on_primary,
            selection_background,
            surface_container,
            surface_container_high,
            on_surface_variant,
            outline,
            terminal,
        })
    }

    pub fn terminal(&self, index: usize) -> Color {
        self.terminal[index % 16]
    }
}

fn pick(value: &Value, keys: &[&str]) -> Option<Color> {
    for key in keys {
        if let Some(s) = value.get(*key).and_then(Value::as_str) {
            if let Ok(c) = Color::from_hex(s) {
                return Some(c);
            }
        }
    }
    None
}

fn default_terminal() -> [Color; 16] {
    [
        Color::new(0x1f, 0x1b, 0x1f),
        Color::new(0xf3, 0xa1, 0xab),
        Color::new(0x8c, 0xc5, 0x99),
        Color::new(0xdb, 0xb2, 0x65),
        Color::new(0x95, 0xba, 0xea),
        Color::new(0xd2, 0xa9, 0xe1),
        Color::new(0x77, 0xc4, 0xc8),
        Color::new(0xb2, 0xab, 0xb0),
        Color::new(0x98, 0x90, 0x95),
        Color::new(0xfb, 0xb2, 0xbd),
        Color::new(0x9f, 0xd1, 0xab),
        Color::new(0xe6, 0xc0, 0x7c),
        Color::new(0xaa, 0xc7, 0xf2),
        Color::new(0xdc, 0xb9, 0xea),
        Color::new(0x90, 0xd0, 0xd3),
        Color::new(0xcb, 0xc3, 0xc9),
    ]
}

pub fn validate_hex(raw: &str) -> Result<Color> {
    match Color::from_hex(raw) {
        Ok(c) => Ok(c),
        Err(_) => bail!("invalid hex colour: {raw}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn loads_inir_json() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("app-palette.json");
        let term = dir.path().join("terminal.json");
        let mut f = std::fs::File::create(&app).unwrap();
        write!(
            f,
            r##"{{"surface":"#131014","on_surface":"#EFEBE4","primary":"#b78cda","primary_container":"#3a2c46","surface_container_high":"#2a262a","on_surface_variant":"#a39ba3","outline":"#847c84"}}"##
        )
        .unwrap();
        let mut f = std::fs::File::create(&term).unwrap();
        write!(f, r##"{{"term0":"#111111","term1":"#ff0000"}}"##).unwrap();
        let p = Palette::from_inir(&app, &term).unwrap();
        assert_eq!(p.background.to_hex(), "#131014");
        assert_eq!(p.accent.to_hex(), "#b78cda");
        assert_eq!(p.terminal(1).to_hex(), "#ff0000");
    }

    #[test]
    fn loads_kitty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("theme.conf");
        std::fs::write(
            &path,
            "background #1c1e19\nforeground #c6c6be\nactive_border_color #35b28f\ncolor1 #ff0000\n",
        )
        .unwrap();
        let p = Palette::from_kitty(&path).unwrap();
        assert_eq!(p.background.to_hex(), "#1c1e19");
        assert_eq!(p.accent.to_hex(), "#35b28f");
    }
}
