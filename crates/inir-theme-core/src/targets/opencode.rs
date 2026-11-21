//! OpenCode theme generator: port of the iNiR `inir` theme with a dark/saturated variant.

use std::collections::HashMap;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::color::{Color, Mode, Role};
use crate::context::{Report, ThemeContext};
use crate::targets::Target;

const EMBEDDED_TEMPLATE: &str = include_str!("../../assets/opencode-inir.json");

pub struct OpencodeTarget;

impl Target for OpencodeTarget {
    fn id(&self) -> &'static str {
        "opencode"
    }
    fn label(&self) -> &'static str {
        "OpenCode"
    }
    fn description(&self) -> &'static str {
        "OpenCode theme: iNiR structure, 60% darker and +50% saturation"
    }

    fn apply(&self, ctx: &ThemeContext) -> Result<Report> {
        let mut report = Report::new(self.id());
        let template_path = ctx.home.join(".config/opencode/themes/inir.json");
        let template = if template_path.exists() {
            std::fs::read_to_string(&template_path)
                .with_context(|| format!("read {}", template_path.display()))?
        } else {
            EMBEDDED_TEMPLATE.to_string()
        };
        let mut theme: Value =
            serde_json::from_str(&template).context("parse opencode template")?;
        let out = ctx.home.join(".config/opencode/themes/inir-dark60.json");
        let palette = build(ctx);
        let bg = palette["m3Surface"];
        apply_defs(&mut theme, &palette, ctx, bg);
        apply_theme_hex(&mut theme, ctx, bg);
        let text = serde_json::to_string_pretty(&theme)? + "\n";
        if ctx.write(&out, &text, &mut report)? {
            report.notes.push(if ctx.dry_run {
                "would write dark60 variant".into()
            } else {
                "wrote dark60 variant".into()
            });
        }
        Ok(report)
    }
}

fn build(ctx: &ThemeContext) -> HashMap<&'static str, Color> {
    let p = &ctx.palette;
    let bg = p
        .background
        .adjust(0.60, 0.50, Role::Background, None, Mode::Readable);
    let fg = p.foreground;
    let accent = ctx.accent();
    let primary = p.terminal(6);
    let secondary = p.terminal(4);
    let error = p.terminal(1);
    let success = p.terminal(2);
    let outline = p.outline;
    let variant = p.on_surface_variant;
    let container = p.surface_container;

    let mut m = HashMap::new();
    m.insert("m3Surface", bg);
    m.insert("m3OnSurface", fg);
    m.insert("m3Primary", primary);
    m.insert("m3Secondary", secondary);
    m.insert("m3Tertiary", accent);
    m.insert("m3Error", error);
    m.insert("m3Success", success);
    m.insert("m3Outline", outline);
    m.insert("m3OnSurfaceVariant", variant);
    m.insert("m3SurfaceContainer", container);
    m.insert("m3SurfaceContainerHigh", bg.mix(fg, 0.10));
    m.insert("m3SurfaceContainerHighest", bg.mix(fg, 0.14));
    m.insert("m3SurfaceContainerLow", bg.mix(fg, 0.04));
    m.insert(
        "m3SurfaceDim",
        p.background
            .adjust(0.15, 0.0, Role::Background, None, Mode::Literal),
    );
    m.insert("m3SurfaceBright", bg.mix(fg, 0.06));
    m.insert("m3PrimaryContainer", primary.mix(bg, 0.80));
    m.insert("m3OnPrimary", bg);
    m.insert("m3OnPrimaryContainer", p.terminal(10));
    m.insert("m3SecondaryContainer", secondary.mix(bg, 0.80));
    m.insert("m3TertiaryContainer", accent.mix(bg, 0.80));
    m.insert("m3ErrorContainer", error.mix(bg, 0.80));
    m.insert("m3SuccessContainer", success.mix(bg, 0.80));
    m.insert("m3OutlineVariant", p.outline);
    m.insert("m3InverseSurface", fg);
    m.insert("m3InverseOnSurface", p.background);
    m.insert("m3InversePrimary", accent);
    m.insert("m3Shadow", Color::new(0, 0, 0));

    let ansi = [
        ("ansiBlue", 4),
        ("ansiBrightBlue", 12),
        ("ansiBrightCyan", 14),
        ("ansiBrightGreen", 10),
        ("ansiBrightMagenta", 13),
        ("ansiBrightRed", 9),
        ("ansiBrightYellow", 11),
        ("ansiCyan", 6),
        ("ansiGreen", 2),
        ("ansiMagenta", 5),
        ("ansiRed", 1),
        ("ansiYellow", 3),
    ];
    for (name, idx) in ansi {
        m.insert(name, p.terminal(idx));
    }
    m
}

fn apply_defs(
    theme: &mut Value,
    palette: &HashMap<&'static str, Color>,
    ctx: &ThemeContext,
    bg: Color,
) {
    let Some(defs) = theme.get_mut("defs").and_then(Value::as_object_mut) else {
        return;
    };
    let keys: Vec<String> = defs.keys().cloned().collect();
    for key in keys {
        let Some(value) = defs.get(&key).and_then(Value::as_str) else {
            continue;
        };
        let Some(src) = palette
            .get(key.as_str())
            .copied()
            .or_else(|| Color::from_hex(value).ok())
        else {
            continue;
        };
        let role = classify_def(&key);
        let out = match role {
            DefRole::Keep => src,
            DefRole::Text => src.adjust(
                ctx.options.opencode_darken,
                ctx.options.opencode_saturate,
                Role::Text,
                Some(bg),
                Mode::Readable,
            ),
            DefRole::Background => src.adjust(
                ctx.options.opencode_darken,
                ctx.options.opencode_saturate,
                Role::Background,
                Some(bg),
                Mode::Readable,
            ),
            DefRole::Muted => src.adjust(
                ctx.options.opencode_darken,
                ctx.options.opencode_saturate,
                Role::Muted,
                Some(bg),
                Mode::Readable,
            ),
            DefRole::Accent => src.adjust(
                ctx.options.opencode_darken,
                ctx.options.opencode_saturate,
                Role::Accent,
                Some(bg),
                Mode::Readable,
            ),
        };
        defs.insert(key, Value::String(out.to_hex()));
    }
}

fn apply_theme_hex(theme: &mut Value, ctx: &ThemeContext, bg: Color) {
    let Some(section) = theme.get_mut("theme").and_then(Value::as_object_mut) else {
        return;
    };
    let keys: Vec<String> = section.keys().cloned().collect();
    for key in keys {
        let Some(entry) = section.get_mut(&key).and_then(Value::as_object_mut) else {
            continue;
        };
        let Some(dark) = entry.get("dark").and_then(Value::as_str) else {
            continue;
        };
        let Some(src) = Color::from_hex(dark).ok() else {
            continue;
        };
        let role = classify_theme(&key);
        let out = match role {
            ThemeRole::Skip => continue,
            ThemeRole::Background => src.adjust(
                ctx.options.opencode_darken,
                ctx.options.opencode_saturate,
                Role::Background,
                Some(bg),
                Mode::Readable,
            ),
            ThemeRole::Text => src.adjust(
                ctx.options.opencode_darken,
                ctx.options.opencode_saturate,
                Role::Text,
                Some(bg),
                Mode::Readable,
            ),
            ThemeRole::Muted => src.adjust(
                ctx.options.opencode_darken,
                ctx.options.opencode_saturate,
                Role::Muted,
                Some(bg),
                Mode::Readable,
            ),
            ThemeRole::Accent => src.adjust(
                ctx.options.opencode_darken,
                ctx.options.opencode_saturate,
                Role::Accent,
                Some(bg),
                Mode::Readable,
            ),
        };
        entry.insert("dark".into(), Value::String(out.to_hex()));
    }
}

#[derive(Clone, Copy)]
enum DefRole {
    Keep,
    Text,
    Background,
    Muted,
    Accent,
}

fn classify_def(key: &str) -> DefRole {
    let k = key.to_ascii_lowercase();
    if k.starts_with("m3inverse") || k.starts_with("m3onprimary") {
        DefRole::Keep
    } else if k.contains("onsurface") {
        DefRole::Text
    } else if k.contains("container") || k.contains("surface") || k.contains("shadow") {
        DefRole::Background
    } else if k.contains("outline") {
        DefRole::Muted
    } else {
        DefRole::Accent
    }
}

#[derive(Clone, Copy)]
enum ThemeRole {
    Skip,
    Background,
    Text,
    Muted,
    Accent,
}

fn classify_theme(key: &str) -> ThemeRole {
    let k = key.to_ascii_lowercase();
    if k == "thinkingopacity" {
        ThemeRole::Skip
    } else if k.contains("background")
        || k.contains("panel")
        || k.contains("menu")
        || k.contains("element")
        || k.ends_with("bg")
    {
        ThemeRole::Background
    } else if k == "textmuted" {
        ThemeRole::Muted
    } else if k == "text" || k.ends_with("text") {
        ThemeRole::Text
    } else if k.contains("border")
        || k.contains("outline")
        || k == "diffhunkheader"
        || k == "difflinenumber"
        || k == "syntaxcomment"
        || k == "syntaxoperator"
        || k == "syntaxpunctuation"
    {
        ThemeRole::Muted
    } else {
        ThemeRole::Accent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::Palette;

    #[test]
    fn writes_valid_theme_with_accent() {
        let dir = tempfile::tempdir().unwrap();
        let kitty = dir.path().join("theme.conf");
        std::fs::write(
            &kitty,
            "background #1c1e19\nforeground #c6c6be\nactive_border_color #b78cda\ncolor0 #1c1e19\ncolor1 #ff0000\ncolor2 #00ff00\ncolor3 #ffff00\ncolor4 #0000ff\ncolor5 #ff00ff\ncolor6 #00ffff\ncolor7 #cccccc\ncolor8 #888888\ncolor9 #ff9999\ncolor10 #99ff99\ncolor11 #ffff99\ncolor12 #9999ff\ncolor13 #ff99ff\ncolor14 #99ffff\ncolor15 #ffffff\n",
        )
        .unwrap();
        let palette = Palette::from_kitty(&kitty).unwrap();
        let mut ctx = ThemeContext::new(palette);
        ctx.home = dir.path().to_path_buf();
        ctx.reload = false;
        OpencodeTarget.apply(&ctx).unwrap();
        let out =
            std::fs::read_to_string(dir.path().join(".config/opencode/themes/inir-dark60.json"))
                .unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["theme"]["accent"]["dark"], "m3Tertiary");
        assert!(v["defs"]["m3Tertiary"].as_str().unwrap().starts_with('#'));
        assert!(v["defs"]["m3Surface"].as_str().unwrap().starts_with('#'));
    }
}
