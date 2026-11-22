//! LiMusic: write the shadcn token sheet the app re-reads on change.

use std::path::Path;

use anyhow::Result;
use serde_json::Value;

use crate::context::{Report, ThemeContext};
use crate::targets::Target;

pub struct LimusicTarget;

impl Target for LimusicTarget {
    fn id(&self) -> &'static str {
        "limusic"
    }
    fn label(&self) -> &'static str {
        "LiMusic"
    }
    fn description(&self) -> &'static str {
        "Write the palette to LiMusic's live matugen.css shadcn tokens"
    }

    fn apply(&self, ctx: &ThemeContext) -> Result<Report> {
        let mut report = Report::new(self.id());
        let css_path = ctx.home.join(".config/limusic/matugen.css");
        if !limusic_enabled(&ctx.home) {
            remove_theme(&css_path, &mut report);
            return Ok(report);
        }
        if !limusic_installed(&ctx.home) {
            report.notes.push("LiMusic not installed".into());
            return Ok(report);
        }
        let Some(colors) = palette(&ctx.home) else {
            report
                .notes
                .push("no generated palette yet; skipped".into());
            return Ok(report);
        };
        let css = render(&colors, theme_mode(&ctx.home));
        if ctx.write(&css_path, &css, &mut report)? {
            report.notes.push(format!("wrote {}", css_path.display()));
        }
        // shortcut: the pre-#351 WebKit/GTK_MODULES launcher patch stays in Bash;
        // current LiMusic reads this file directly. Upgrade when an old release must be themed.
        Ok(report)
    }
}

fn limusic_enabled(home: &Path) -> bool {
    let path = home.join(".config/illogical-impulse/config.json");
    let Some(value) = std::fs::read(path)
        .ok()
        .and_then(|raw| serde_json::from_slice::<Value>(&raw).ok())
    else {
        return false;
    };
    value
        .pointer("/appearance/wallpaperTheming/enableLimusic")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn limusic_installed(home: &Path) -> bool {
    home.join(".config/com.limusic.desktop").is_dir()
        || home.join(".local/share/com.limusic.desktop").is_dir()
        || home.join(".config/limusic").is_dir()
        || command_exists("limusic")
        || command_exists("limusic-app")
}

fn command_exists(name: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join(name).is_file())
}

fn palette(home: &Path) -> Option<Value> {
    let state = home.join(".local/state/quickshell/user/generated");
    for name in ["app-palette.json", "palette.json", "colors.json"] {
        if let Some(value) = std::fs::read(state.join(name))
            .ok()
            .and_then(|raw| serde_json::from_slice::<Value>(&raw).ok())
        {
            return Some(value);
        }
    }
    None
}

fn theme_mode(home: &Path) -> &'static str {
    let path = home.join(".local/state/quickshell/user/generated/theme-meta.json");
    let light = std::fs::read(path)
        .ok()
        .and_then(|raw| serde_json::from_slice::<Value>(&raw).ok())
        .and_then(|value| {
            value
                .get("mode")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .is_some_and(|mode| mode == "light");
    if light {
        "light"
    } else {
        "dark"
    }
}

fn render(colors: &Value, mode: &str) -> String {
    let pick = |keys: &[&str], fallback: &str| -> String {
        keys.iter()
            .find_map(|key| colors.get(*key).and_then(Value::as_str))
            .filter(|raw| is_hex(raw))
            .unwrap_or(fallback)
            .to_string()
    };
    let bg = pick(&["app_background", "surface"], "#121318");
    let fg = pick(&["app_foreground", "on_surface"], "#e3e2e9");
    let card = pick(&["app_surface", "surface_container_low"], "#1a1b20");
    let popover = pick(&["app_surface_popup", "surface_container_high"], "#292a2f");
    let raised = pick(&["app_surface_elevated", "surface_container"], "#1e1f25");
    let hover = pick(&["app_surface_hover", "surface_container_high"], "#292a2f");
    let subtext = pick(&["app_subtext", "on_surface_variant"], "#c5c6d0");
    let border = pick(&["app_border_subtle", "outline_variant"], "#44464f");
    let accent = pick(&["app_accent", "primary"], "#b2c5ff");
    let on_accent = pick(&["app_on_accent", "on_primary"], "#182e60");
    let sidebar = pick(&["app_sidebar_bg", "app_background", "surface"], "#121318");
    let error = pick(&["error"], "#ffb4ab");
    let chart2 = pick(&["secondary"], "#c1c6dd");
    let chart3 = pick(&["tertiary"], "#e1bbdc");
    let chart4 = pick(&["app_accent_container", "primary_container"], "#304578");
    let chart5 = pick(&["outline"], "#8e9099");
    format!(
        "/* iNiR — LiMusic. Generated from the current palette; edits are overwritten. */\n\n\
/* Outranks LiMusic's presets (.dark.theme-x is 0,2,0) without a class of its own, so it holds\n\
   whichever preset or mode is selected. */\n\
html:root:root {{\n\
  --foreground: {fg};\n\
  --card-foreground: {fg};\n\
  --popover-foreground: {fg};\n\
  --primary: {accent};\n\
  --primary-foreground: {on_accent};\n\
  --secondary-foreground: {fg};\n\
  --muted-foreground: {subtext};\n\
  --accent: {accent};\n\
  --accent-foreground: {on_accent};\n\
  --destructive: {error};\n\
  --border: {border};\n\
  --input: {border};\n\
  --ring: {accent};\n\
  --chart-1: {accent};\n\
  --chart-2: {chart2};\n\
  --chart-3: {chart3};\n\
  --chart-4: {chart4};\n\
  --chart-5: {chart5};\n\
  --sidebar-foreground: {fg};\n\
  --sidebar-primary: {accent};\n\
  --sidebar-primary-foreground: {on_accent};\n\
  --sidebar-accent-foreground: {fg};\n\
  --sidebar-border: {border};\n\
  --sidebar-ring: {accent};\n\
  color-scheme: {mode};\n\
}}\n\
\n\
/* Fills. \"Adapt colors to artwork\" tints these from the cover, so they step aside while it is on. */\n\
html:root:root:not(.art-tint) {{\n\
  --background: {bg};\n\
  --card: {card};\n\
  --popover: {popover};\n\
  --secondary: {raised};\n\
  --muted: {raised};\n\
  --sidebar: {sidebar};\n\
  --sidebar-accent: {hover};\n\
}}\n"
    )
}

fn remove_theme(path: &Path, report: &mut Report) {
    let Some(first) = std::fs::read_to_string(path)
        .ok()
        .and_then(|text| text.lines().next().map(str::to_string))
    else {
        return;
    };
    if !first.contains("iNiR") {
        return;
    }
    match std::fs::remove_file(path) {
        Ok(()) => report.notes.push(format!("removed {}", path.display())),
        Err(err) => report
            .notes
            .push(format!("cannot remove {}: {err}", path.display())),
    }
}

fn is_hex(raw: &str) -> bool {
    raw.len() == 7 && raw.starts_with('#') && raw[1..].chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::Palette;

    fn setup(dir: &Path, enabled: bool, mode: &str) -> ThemeContext {
        let state = dir.join(".local/state/quickshell/user/generated");
        std::fs::create_dir_all(&state).unwrap();
        std::fs::write(
            state.join("app-palette.json"),
            serde_json::json!({
                "app_background": "#101010",
                "app_foreground": "#eeeeee",
                "app_surface": "#181818",
                "app_surface_popup": "#202020",
                "app_surface_elevated": "#1c1c1c",
                "app_surface_hover": "#222222",
                "app_subtext": "#bbbbbb",
                "app_border_subtle": "#333333",
                "app_accent": "#112233",
                "app_on_accent": "#ffffff",
                "app_sidebar_bg": "#101010",
                "error": "#ff0000",
                "secondary": "#00ff00",
                "tertiary": "#0000ff",
                "app_accent_container": "#223344",
                "outline": "#444444"
            })
            .to_string(),
        )
        .unwrap();
        std::fs::write(
            state.join("theme-meta.json"),
            serde_json::json!({"mode": mode}).to_string(),
        )
        .unwrap();
        let config = dir.join(".config/illogical-impulse/config.json");
        std::fs::create_dir_all(config.parent().unwrap()).unwrap();
        std::fs::write(
            &config,
            serde_json::json!({
                "appearance": {"wallpaperTheming": {"enableLimusic": enabled}}
            })
            .to_string(),
        )
        .unwrap();
        std::fs::create_dir_all(dir.join(".config/limusic")).unwrap();
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
    fn writes_css_when_enabled() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = setup(dir.path(), true, "dark");
        LimusicTarget.apply(&ctx).unwrap();
        let css = std::fs::read_to_string(dir.path().join(".config/limusic/matugen.css")).unwrap();
        assert!(css.contains("--primary: #112233;"));
        assert!(css.contains("--background: #101010;"));
        assert!(css.contains("color-scheme: dark;"));
        assert!(css.contains("--chart-4: #223344;"));
    }

    #[test]
    fn removes_theme_when_disabled() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = setup(dir.path(), false, "dark");
        std::fs::write(
            dir.path().join(".config/limusic/matugen.css"),
            "/* iNiR — LiMusic */\nhtml:root:root {}\n",
        )
        .unwrap();
        LimusicTarget.apply(&ctx).unwrap();
        assert!(!dir.path().join(".config/limusic/matugen.css").exists());
    }

    #[test]
    fn skips_when_not_installed() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = setup(dir.path(), true, "dark");
        std::fs::remove_dir_all(dir.path().join(".config/limusic")).unwrap();
        LimusicTarget.apply(&ctx).unwrap();
        assert!(!dir.path().join(".config/limusic/matugen.css").exists());
    }

    #[test]
    fn writes_light_mode() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = setup(dir.path(), true, "light");
        LimusicTarget.apply(&ctx).unwrap();
        let css = std::fs::read_to_string(dir.path().join(".config/limusic/matugen.css")).unwrap();
        assert!(css.contains("color-scheme: light;"));
    }
}
