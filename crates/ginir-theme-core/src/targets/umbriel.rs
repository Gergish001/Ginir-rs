//! Umbriel: window borders, tabs and panels from iNiR's generated colours.

use std::path::Path;

use anyhow::Result;
use serde_json::Value;

use crate::color::Color;
use crate::context::{Report, ThemeContext};
use crate::targets::Target;

pub struct UmbrielTarget;

impl Target for UmbrielTarget {
    fn id(&self) -> &'static str {
        "umbriel"
    }
    fn label(&self) -> &'static str {
        "Umbriel"
    }
    fn description(&self) -> &'static str {
        "Window borders, tabs and panels in the theme colours; adds iRiS compositor-glass blur"
    }

    fn apply(&self, ctx: &ThemeContext) -> Result<Report> {
        let mut report = Report::new(self.id());
        let config = ctx.home.join(".config/umbriel/config.toml");
        let marker = "# iNiR — Umbriel configuration";
        if !std::fs::read_to_string(&config)
            .map(|text| text.lines().any(|line| line.starts_with(marker)))
            .unwrap_or(false)
        {
            report
                .notes
                .push("Umbriel config is not iNiR's; skipped".into());
            return Ok(report);
        }

        let colors_path = ctx
            .home
            .join(".local/state/quickshell/user/generated/colors.json");
        let colors = match std::fs::read(&colors_path)
            .ok()
            .and_then(|raw| serde_json::from_slice::<Value>(&raw).ok())
        {
            Some(colors) => colors,
            None => {
                report
                    .notes
                    .push(format!("no colours at {}", colors_path.display()));
                return Ok(report);
            }
        };

        let shell = shell_options(&ctx.home.join(".config/illogical-impulse/config.json"));
        let content = render(&colors, &shell.family, &shell.glass);
        let out = ctx
            .home
            .join(".local/state/quickshell/user/generated/umbriel-colors.toml");
        if ctx.write(&out, &content, &mut report)? {
            report.notes.push("wrote Umbriel colours".into());
        }
        Ok(report)
    }
}

struct ShellOptions {
    family: String,
    glass: String,
}

fn shell_options(path: &Path) -> ShellOptions {
    let value = std::fs::read(path)
        .ok()
        .and_then(|raw| serde_json::from_slice::<Value>(&raw).ok());
    let family = value
        .as_ref()
        .and_then(|v| v.get("panelFamily"))
        .and_then(Value::as_str)
        .unwrap_or("ii")
        .to_string();
    let glass = value
        .as_ref()
        .and_then(|v| v.pointer("/iris/appearance/glass/mode"))
        .and_then(Value::as_str)
        .unwrap_or("wallpaper")
        .to_string();
    ShellOptions { family, glass }
}

fn render(colors: &Value, family: &str, glass: &str) -> String {
    let c = |key: &str, alpha: &str| pick(colors, key, alpha);
    let mut lines = vec![
        "# Written by iNiR from the theme on every colour change; edits here are lost. Your own: config.d/90-user-extra.toml.".into(),
        String::new(),
        "[colors]".into(),
        format!("background = \"{}\"", c("surface_container", "FF")),
        format!("text_primary = \"{}\"", c("on_surface", "FF")),
        format!("text_muted = \"{}\"", c("on_surface_variant", "FF")),
        format!("accent_primary = \"{}\"", c("primary", "FF")),
        format!("accent_secondary = \"{}\"", c("tertiary", "FF")),
        format!("warning = \"{}\"", c("tertiary", "FF")),
        format!("error = \"{}\"", c("error", "FF")),
        format!("insert_hint = \"{}\"", c("primary", "55")),
        "shadow = \"#00000070\"".into(),
        String::new(),
        "[colors.border]".into(),
        format!("focused = \"{}\"", c("primary", "FF")),
        format!("unfocused = \"{}\"", c("outline_variant", "FF")),
        format!("outer = \"{}\"", c("surface", "FF")),
        String::new(),
        "[colors.overview]".into(),
        format!("background_tint = \"{}\"", c("surface", "66")),
        format!(
            "workspace_background = \"{}\"",
            c("surface_container_low", "88")
        ),
        format!("badge = \"{}\"", c("primary", "FF")),
        String::new(),
        "[colors.tab_bar]".into(),
        format!("background = \"{}\"", c("surface_container", "FF")),
        format!("text = \"{}\"", c("on_surface_variant", "FF")),
        format!("active = \"{}\"", c("primary", "FF")),
        format!("active_text = \"{}\"", c("on_primary", "FF")),
        format!(
            "active_unfocused = \"{}\"",
            c("surface_container_highest", "FF")
        ),
        format!("active_unfocused_text = \"{}\"", c("on_surface", "FF")),
        format!("urgent = \"{}\"", c("error", "FF")),
        format!("urgent_text = \"{}\"", c("on_error", "FF")),
        String::new(),
        "[colors.title_bar]".into(),
        format!("background = \"{}\"", c("surface_container_high", "FF")),
        format!(
            "background_unfocused = \"{}\"",
            c("surface_container", "FF")
        ),
        format!("text = \"{}\"", c("on_surface", "FF")),
        format!("text_unfocused = \"{}\"", c("on_surface_variant", "FF")),
        format!("close = \"{}\"", c("error", "FF")),
        format!("close_glyph = \"{}\"", c("on_error", "FF")),
        format!("close_glyph_hover = \"{}\"", c("on_error", "FF")),
        format!("maximize = \"{}\"", c("primary", "FF")),
        format!("maximize_glyph = \"{}\"", c("on_primary", "FF")),
        format!("maximize_glyph_hover = \"{}\"", c("on_primary", "FF")),
        format!("minimize = \"{}\"", c("tertiary", "FF")),
        format!("minimize_glyph = \"{}\"", c("on_tertiary", "FF")),
        format!("minimize_glyph_hover = \"{}\"", c("on_tertiary", "FF")),
    ];

    if family == "iris" && glass == "compositor" {
        lines.extend([
            String::new(),
            "# iRiS asked for compositor glass. Umbriel has no blur region: it blurs every pixel of a matched surface above".into(),
            "# the alpha threshold. So only the surfaces that publish a region under Niri are matched (not Prism, whose".into(),
            "# scrim covers the screen), and the threshold sits over the iRiS shadows (kept under 0.1) and under its glass (0.12+).".into(),
            "# Their popups (tray menus) take the same threshold, so only the menu itself is glass, never its box.".into(),
            "[[layer_rule]]".into(),
            "match.namespace = \"^quickshell:iris-(chassis|sidebar-|notifications|osd|osk|menu|edit|settings)\"".into(),
            "blur = true".into(),
            "blur_popups = true".into(),
            "blur_ignore_alpha = 0.1".into(),
            "blur_optimized = false".into(),
            String::new(),
            "# The Umbriel blur dims and flattens what it blurs (0.9); the Niri one does neither. These are the Niri values".into(),
            "# (3 passes of 3, saturation 1.5), so the glass reads the same on both.".into(),
            "# Settings writes after this file: what a person picks there wins.".into(),
            "[appearance.blur]".into(),
            "passes = 3".into(),
            "radius = 3".into(),
            "brightness = 1.0".into(),
            "contrast = 1.0".into(),
            "saturation = 1.5".into(),
        ]);
    }

    let mut out = lines.join("\n");
    out.push('\n');
    out
}

fn pick(colors: &Value, key: &str, alpha: &str) -> String {
    let raw = colors.get(key).and_then(Value::as_str).unwrap_or("#808080");
    let hex = Color::from_hex(raw)
        .map(|c| c.to_hex().to_uppercase())
        .unwrap_or_else(|_| "#808080".into());
    format!("{hex}{alpha}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::Palette;

    fn setup(dir: &Path, family: &str, glass: &str, marker: bool) -> ThemeContext {
        let colors = serde_json::json!({
            "surface_container": "#2b381b",
            "on_surface": "#c6ee97",
            "on_surface_variant": "#a3a3a3",
            "primary": "#84a955",
            "tertiary": "#7ea4ea",
            "error": "#ff0000",
            "outline_variant": "#555555",
            "surface": "#12190a",
            "surface_container_low": "#1a2110",
            "surface_container_highest": "#333333",
            "surface_container_high": "#222222",
            "on_primary": "#000000",
            "on_error": "#ffffff",
            "on_tertiary": "#000000"
        });
        let state = dir.join(".local/state/quickshell/user/generated");
        std::fs::create_dir_all(&state).unwrap();
        std::fs::write(
            state.join("colors.json"),
            serde_json::to_vec(&colors).unwrap(),
        )
        .unwrap();
        let cfg = dir.join(".config/illogical-impulse/config.json");
        std::fs::create_dir_all(cfg.parent().unwrap()).unwrap();
        std::fs::write(
            &cfg,
            serde_json::json!({
                "panelFamily": family,
                "iris": {"appearance": {"glass": {"mode": glass}}}
            })
            .to_string(),
        )
        .unwrap();
        let umbriel = dir.join(".config/umbriel/config.toml");
        std::fs::create_dir_all(umbriel.parent().unwrap()).unwrap();
        std::fs::write(
            &umbriel,
            if marker {
                "# iNiR — Umbriel configuration\n[colors]\n"
            } else {
                "[colors]\n"
            },
        )
        .unwrap();
        let kitty = dir.join("theme.conf");
        std::fs::write(
            &kitty,
            "background #1c1e19\nforeground #c6c6be\nactive_border_color #35b28f\n",
        )
        .unwrap();
        let mut ctx = ThemeContext::new(Palette::from_kitty(&kitty).unwrap());
        ctx.home = dir.to_path_buf();
        ctx.reload = false;
        ctx
    }

    #[test]
    fn writes_ii_colours_without_blur() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = setup(dir.path(), "ii", "wallpaper", true);
        UmbrielTarget.apply(&ctx).unwrap();
        let out = std::fs::read_to_string(
            dir.path()
                .join(".local/state/quickshell/user/generated/umbriel-colors.toml"),
        )
        .unwrap();
        assert!(out.contains("background = \"#2B381BFF\""));
        assert!(out.contains("accent_primary = \"#84A955FF\""));
        assert!(out.contains("active_unfocused = \"#333333FF\""));
        assert!(!out.contains("[[layer_rule]]"));
    }

    #[test]
    fn adds_compositor_glass_for_iris() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = setup(dir.path(), "iris", "compositor", true);
        UmbrielTarget.apply(&ctx).unwrap();
        let out = std::fs::read_to_string(
            dir.path()
                .join(".local/state/quickshell/user/generated/umbriel-colors.toml"),
        )
        .unwrap();
        assert!(out.contains("[[layer_rule]]"));
        assert!(out.contains("[appearance.blur]"));
        assert!(out.contains("blur_ignore_alpha = 0.1"));
    }

    #[test]
    fn skips_when_config_is_not_inir() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = setup(dir.path(), "ii", "wallpaper", false);
        UmbrielTarget.apply(&ctx).unwrap();
        assert!(!dir
            .path()
            .join(".local/state/quickshell/user/generated/umbriel-colors.toml")
            .exists());
    }

    #[test]
    fn falls_back_uppercase() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = setup(dir.path(), "ii", "wallpaper", true);
        let colors = dir
            .path()
            .join(".local/state/quickshell/user/generated/colors.json");
        std::fs::write(&colors, "{}").unwrap();
        UmbrielTarget.apply(&ctx).unwrap();
        let out = std::fs::read_to_string(
            dir.path()
                .join(".local/state/quickshell/user/generated/umbriel-colors.toml"),
        )
        .unwrap();
        assert!(out.contains("background = \"#808080FF\""));
    }
}
