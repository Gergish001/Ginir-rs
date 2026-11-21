//! OpenCode: keep the active iNiR `inir` theme's accent in sync with the system.
//!
//! iNiR generates `~/.config/opencode/themes/inir.json` from the wallpaper. Its
//! `accent` token points at `m3Tertiary`, which is a *tertiary* palette role and
//! does not follow `app_accent`. This target rewrites that token (and its
//! container) to the exact accent used by herdr and the rest of the shell.
//! Everything else in the generated theme is left untouched.

use anyhow::{Context, Result};
use serde_json::Value;

use crate::color::Color;
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
        "Patch the active OpenCode inir theme to use the exact iNiR accent"
    }

    fn apply(&self, ctx: &ThemeContext) -> Result<Report> {
        let mut report = Report::new(self.id());
        let theme_path = ctx.home.join(".config/opencode/themes/inir.json");
        let template = if theme_path.exists() {
            std::fs::read_to_string(&theme_path)
                .with_context(|| format!("read {}", theme_path.display()))?
        } else {
            EMBEDDED_TEMPLATE.to_string()
        };
        let mut theme: Value = serde_json::from_str(&template).context("parse opencode theme")?;
        let accent = ctx.accent();
        let bg = palette_background(&theme).unwrap_or(ctx.palette.background);
        patch_accent(&mut theme, accent, bg)?;
        let text = serde_json::to_string_pretty(&theme)? + "\n";
        if ctx.write(&theme_path, &text, &mut report)? {
            report.notes.push(if ctx.dry_run {
                "would sync the accent".into()
            } else {
                "accent synced".into()
            });
            report
                .notes
                .push(format!("m3Tertiary -> {}", accent.to_hex()));
        }
        Ok(report)
    }
}

/// The generated theme stores its background as `m3Surface`; use it as the
/// backing colour when deriving translucent accents.
fn palette_background(theme: &Value) -> Option<Color> {
    theme
        .get("defs")
        .and_then(|defs| defs.get("m3Surface"))
        .and_then(Value::as_str)
        .and_then(|raw| Color::from_hex(raw).ok())
}

fn patch_accent(theme: &mut Value, accent: Color, bg: Color) -> Result<()> {
    let defs = theme
        .get_mut("defs")
        .and_then(Value::as_object_mut)
        .context("opencode theme has no defs object")?;
    defs.insert("m3Tertiary".into(), Value::String(accent.to_hex()));
    defs.insert(
        "m3TertiaryContainer".into(),
        Value::String(accent.mix(bg, 0.80).to_hex()),
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::Palette;

    #[test]
    fn syncs_accent_and_keeps_the_rest_of_the_theme() {
        let dir = tempfile::tempdir().unwrap();
        let themes = dir.path().join(".config/opencode/themes");
        std::fs::create_dir_all(&themes).unwrap();
        std::fs::write(
            themes.join("inir.json"),
            r##"{
  "defs": {
    "m3Surface": "#191e1f",
    "m3OnSurface": "#c2c7c7",
    "m3Primary": "#60a3e6",
    "m3Tertiary": "#c49f4d",
    "m3TertiaryContainer": "#423000"
  },
  "theme": {
    "accent": { "dark": "m3Tertiary", "light": "#cbb174" },
    "text": { "dark": "m3OnSurface", "light": "#2b2b2b" }
  }
}"##,
        )
        .unwrap();

        let kitty = dir.path().join("theme.conf");
        std::fs::write(
            &kitty,
            "background #1c1e19\nforeground #c6c6be\nactive_border_color #b78cda\n",
        )
        .unwrap();
        let mut ctx = ThemeContext::new(Palette::from_kitty(&kitty).unwrap());
        ctx.home = dir.path().to_path_buf();
        ctx.reload = false;

        OpencodeTarget.apply(&ctx).unwrap();

        let out = std::fs::read_to_string(themes.join("inir.json")).unwrap();
        let theme: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(theme["defs"]["m3Tertiary"], "#b78cda");
        assert_ne!(theme["defs"]["m3TertiaryContainer"], "#423000");
        assert_eq!(theme["defs"]["m3Surface"], "#191e1f");
        assert_eq!(theme["defs"]["m3OnSurface"], "#c2c7c7");
        assert_eq!(theme["theme"]["accent"]["dark"], "m3Tertiary");
    }
}
