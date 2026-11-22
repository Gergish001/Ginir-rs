//! Firefox family: write the palette as `ui.*` system colours into `user.js`.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde_json::Value;

use crate::context::{Report, ThemeContext};
use crate::targets::Target;

const BLOCK_BEGIN: &str = "// iNiR colours: begin (rewritten by iNiR on every wallpaper change)";
const BLOCK_END: &str = "// iNiR colours: end";

const PREF_KEYS: [&str; 43] = [
    "-moz-headerbar",
    "-moz-headerbartext",
    "-moz-headerbarinactive",
    "-moz-headerbarinactivetext",
    "activecaption",
    "captiontext",
    "inactivecaption",
    "inactivecaptiontext",
    "-moz-dialog",
    "-moz-dialogtext",
    "window",
    "windowtext",
    "-moz-field",
    "-moz-fieldtext",
    "menu",
    "menutext",
    "infobackground",
    "infotext",
    "-moz_menuhover",
    "-moz_menuhovertext",
    "-moz_menubarhovertext",
    "selecteditem",
    "selecteditemtext",
    "-moz-cellhighlight",
    "-moz_cellhighlighttext",
    "highlight",
    "highlighttext",
    "accentcolor",
    "accentcolortext",
    "-moz-hyperlinktext",
    "-moz-activehyperlinktext",
    "-moz-visitedhyperlinktext",
    "-moz-sidebar",
    "-moz-sidebartext",
    "-moz-sidebarborder",
    "buttonface",
    "buttontext",
    "-moz-buttonhoverface",
    "-moz_buttonhovertext",
    "-moz-buttonactiveface",
    "-moz-buttonactivetext",
    "buttonborder",
    "graytext",
];

pub struct FirefoxTarget;

impl Target for FirefoxTarget {
    fn id(&self) -> &'static str {
        "firefox"
    }
    fn label(&self) -> &'static str {
        "Firefox family"
    }
    fn description(&self) -> &'static str {
        "Write the palette as Firefox ui.* system colours into every family profile"
    }

    fn apply(&self, ctx: &ThemeContext) -> Result<Report> {
        let mut report = Report::new(self.id());
        let enabled = firefox_enabled(&ctx.home);
        let app_palette = ctx
            .home
            .join(".local/state/quickshell/user/generated/app-palette.json");
        let palette = if enabled {
            match std::fs::read(&app_palette)
                .ok()
                .and_then(|raw| serde_json::from_slice::<Value>(&raw).ok())
            {
                Some(value) => Some(value),
                None => {
                    report.notes.push("no app palette yet; skipped".into());
                    return Ok(report);
                }
            }
        } else {
            None
        };

        let block = render_block(palette.as_ref());
        let profiles = list_profiles(&ctx.home);
        if profiles.is_empty() {
            report.notes.push("no Firefox-family profiles found".into());
            return Ok(report);
        }

        for profile in profiles {
            let user_js = profile.join("user.js");
            if std::fs::symlink_metadata(&user_js)
                .map(|meta| meta.file_type().is_symlink())
                .unwrap_or(false)
            {
                report
                    .notes
                    .push(format!("left {} alone: it is a link", user_js.display()));
                continue;
            }
            let existing = std::fs::read_to_string(&user_js).unwrap_or_default();
            if !enabled && !existing.contains(BLOCK_BEGIN) {
                continue;
            }
            let new = replace_block(&existing, &block);
            if ctx.write(&user_js, &new, &mut report)? {
                report.notes.push(format!(
                    "{} {}",
                    if enabled { "wrote" } else { "emptied" },
                    user_js.display()
                ));
            }
        }
        Ok(report)
    }
}

fn firefox_enabled(home: &Path) -> bool {
    let path = home.join(".config/illogical-impulse/config.json");
    let Some(value) = std::fs::read(path)
        .ok()
        .and_then(|raw| serde_json::from_slice::<Value>(&raw).ok())
    else {
        return true;
    };
    value
        .pointer("/appearance/wallpaperTheming/enableFirefox")
        .and_then(Value::as_bool)
        .unwrap_or(true)
}

fn render_block(palette: Option<&Value>) -> String {
    let mut lines = vec![BLOCK_BEGIN.to_string()];
    let values = palette.map(pref_values).unwrap_or_default();
    for (index, key) in PREF_KEYS.iter().enumerate() {
        let value = values.get(index).map(String::as_str).unwrap_or("");
        lines.push(format!("user_pref(\"ui.{key}\", \"{value}\");"));
    }
    lines.push(BLOCK_END.to_string());
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

fn pref_values(palette: &Value) -> Vec<String> {
    let pick = |keys: &[&str]| -> String {
        keys.iter()
            .find_map(|key| palette.get(*key).and_then(Value::as_str))
            .filter(|raw| is_hex(raw))
            .unwrap_or("#808080")
            .to_string()
    };
    let frame = pick(&["app_headerbar_bg", "surface"]);
    let raised = pick(&["app_surface_popup", "surface_container_high"]);
    let (frame, raised) = if luma(&frame) <= luma(&raised) {
        (frame, raised)
    } else {
        (raised, frame)
    };
    let text = pick(&["app_foreground", "on_surface"]);
    let subtext = pick(&["app_subtext", "on_surface_variant"]);
    let on_raised = pick(&["app_on_surface_popup", "on_surface"]);
    let selection = pick(&["app_selection", "secondary_container"]);
    let on_selection = pick(&["app_on_selection", "on_secondary_container"]);
    let accent = pick(&["app_accent", "primary"]);
    let on_accent = pick(&["app_on_accent", "on_primary"]);
    let border = pick(&["app_border_subtle", "outline_variant"]);
    let elevated = pick(&["app_surface_elevated"]);
    let elevated_hover = pick(&["app_surface_elevated_hover"]);
    let elevated_active = pick(&["app_surface_elevated_active"]);
    vec![
        frame.clone(),
        text.clone(),
        frame.clone(),
        subtext.clone(),
        frame.clone(),
        text.clone(),
        frame.clone(),
        subtext.clone(),
        under_white(&raised, 0.15),
        on_raised.clone(),
        frame.clone(),
        text.clone(),
        raised.clone(),
        on_raised.clone(),
        raised.clone(),
        on_raised.clone(),
        raised.clone(),
        on_raised.clone(),
        selection.clone(),
        on_selection.clone(),
        on_selection.clone(),
        selection.clone(),
        on_selection.clone(),
        selection.clone(),
        on_selection.clone(),
        selection.clone(),
        on_selection.clone(),
        accent.clone(),
        on_accent,
        accent.clone(),
        accent.clone(),
        accent,
        frame,
        text.clone(),
        border.clone(),
        elevated.clone(),
        text.clone(),
        elevated_hover.clone(),
        text.clone(),
        elevated_active,
        text,
        border,
        subtext,
    ]
}

fn list_profiles(home: &Path) -> Vec<PathBuf> {
    let mut profiles = BTreeSet::new();
    for root in profile_roots(home) {
        let ini = root.join("profiles.ini");
        let Ok(text) = std::fs::read_to_string(ini) else {
            continue;
        };
        for line in text.lines() {
            let line = line.trim_end_matches('\r');
            let Some(path) = line.strip_prefix("Path=") else {
                continue;
            };
            let path = if Path::new(path).is_absolute() {
                PathBuf::from(path)
            } else {
                root.join(path)
            };
            if path.is_dir() {
                profiles.insert(path);
            }
        }
    }
    profiles.into_iter().collect()
}

fn profile_roots(home: &Path) -> Vec<PathBuf> {
    let config = home.join(".config");
    vec![
        config.join("mozilla/firefox"),
        home.join(".mozilla/firefox"),
        home.join(".var/app/org.mozilla.firefox/config/mozilla/firefox"),
        home.join(".var/app/org.mozilla.firefox/.mozilla/firefox"),
        home.join(".librewolf"),
        config.join("librewolf/librewolf"),
        home.join(".var/app/io.gitlab.librewolf-community/.librewolf"),
        home.join(".floorp"),
        home.join(".var/app/one.ablaze.floorp/.floorp"),
        home.join(".waterfox"),
        home.join(".var/app/net.waterfox.waterfox/.waterfox"),
        home.join(".zen"),
        config.join("zen"),
        home.join(".var/app/app.zen_browser.zen/.zen"),
    ]
}

fn replace_block(existing: &str, block: &str) -> String {
    let mut kept = Vec::new();
    let mut skip = false;
    for line in existing.lines() {
        if line == BLOCK_BEGIN {
            skip = true;
            continue;
        }
        if line == BLOCK_END {
            skip = false;
            continue;
        }
        if !skip {
            kept.push(line);
        }
    }
    let mut out = kept.join("\n");
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(block);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn is_hex(raw: &str) -> bool {
    raw.len() == 7 && raw.starts_with('#') && raw[1..].chars().all(|c| c.is_ascii_hexdigit())
}

fn rgb(raw: &str) -> Option<[u8; 3]> {
    if !is_hex(raw) {
        return None;
    }
    let raw = raw.trim_start_matches('#');
    Some([
        u8::from_str_radix(&raw[0..2], 16).ok()?,
        u8::from_str_radix(&raw[2..4], 16).ok()?,
        u8::from_str_radix(&raw[4..6], 16).ok()?,
    ])
}

fn luma(raw: &str) -> f64 {
    let [r, g, b] = rgb(raw).unwrap_or([128, 128, 128]);
    f64::from(r) * 0.2126 + f64::from(g) * 0.7152 + f64::from(b) * 0.0722
}

/// Firefox paints the toolbar as `color-mix(-moz-dialog 85%, white)`. Solve the
/// dialog colour that lands it on `raw`, lifting very dark tints first.
fn under_white(raw: &str, share: f64) -> String {
    let [r, g, b] = rgb(raw).unwrap_or([128, 128, 128]);
    let floor = (255.0 * share).ceil() as i32;
    let low = i32::from(*[r, g, b].iter().min().unwrap());
    let lift = (floor - low).max(0);
    let mut out = [0u8; 3];
    for (slot, channel) in out.iter_mut().zip([r, g, b]) {
        let lifted = i32::from(channel) + lift;
        *slot = (((f64::from(lifted) - 255.0 * share) / (1.0 - share)).round() as i32).clamp(0, 255)
            as u8;
    }
    format!("#{:02X}{:02X}{:02X}", out[0], out[1], out[2])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::Palette;

    fn setup(dir: &Path, enabled: bool) -> ThemeContext {
        let app = dir.join(".local/state/quickshell/user/generated/app-palette.json");
        std::fs::create_dir_all(app.parent().unwrap()).unwrap();
        std::fs::write(
            &app,
            serde_json::json!({
                "app_headerbar_bg": "#222222",
                "surface": "#111111",
                "app_surface_popup": "#333333",
                "surface_container_high": "#444444",
                "app_foreground": "#eeeeee",
                "on_surface": "#dddddd",
                "app_subtext": "#bbbbbb",
                "on_surface_variant": "#aaaaaa",
                "app_selection": "#445566",
                "secondary_container": "#334455",
                "app_on_selection": "#ffffff",
                "on_secondary_container": "#eeeeee",
                "app_accent": "#84a955",
                "primary": "#84a955",
                "app_on_accent": "#000000",
                "app_border_subtle": "#555555",
                "outline_variant": "#666666",
                "app_surface_elevated": "#2a2a2a",
                "app_surface_elevated_hover": "#2b2b2b",
                "app_surface_elevated_active": "#2c2c2c",
                "app_on_surface_popup": "#eeeeee",
                "secondary_container_high": "#444444"
            })
            .to_string(),
        )
        .unwrap();

        let config = dir.join(".config/illogical-impulse/config.json");
        std::fs::create_dir_all(config.parent().unwrap()).unwrap();
        std::fs::write(
            &config,
            serde_json::json!({
                "appearance": {"wallpaperTheming": {"enableFirefox": enabled}}
            })
            .to_string(),
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

    fn add_profile(dir: &Path, name: &str, user_js: Option<&str>) -> PathBuf {
        let root = dir.join(".mozilla/firefox");
        std::fs::create_dir_all(&root).unwrap();
        let profile = root.join(name);
        std::fs::create_dir_all(&profile).unwrap();
        let ini = root.join("profiles.ini");
        let mut text = std::fs::read_to_string(&ini).unwrap_or_default();
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str(&format!("Path={name}\n"));
        std::fs::write(ini, text).unwrap();
        if let Some(user_js) = user_js {
            std::fs::write(profile.join("user.js"), user_js).unwrap();
        }
        profile
    }

    #[test]
    fn writes_managed_block() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = setup(dir.path(), true);
        let profile = add_profile(dir.path(), "abc.default", Some("user_pref(\"keep\", 1);\n"));
        FirefoxTarget.apply(&ctx).unwrap();
        let out = std::fs::read_to_string(profile.join("user.js")).unwrap();
        assert!(out.contains("user_pref(\"keep\", 1);"));
        assert!(out.contains(BLOCK_BEGIN));
        assert!(out.contains("user_pref(\"ui.-moz-headerbar\", \"#222222\");"));
        assert!(out.contains("user_pref(\"ui.accentcolor\", \"#84a955\");"));
        assert!(out.contains(BLOCK_END));
    }

    #[test]
    fn off_empties_existing_block_only() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = setup(dir.path(), false);
        let colored = add_profile(
            dir.path(),
            "colored.default",
            Some(&format!(
                "{BLOCK_BEGIN}\nuser_pref(\"ui.-moz-headerbar\", \"#222222\");\n{BLOCK_END}\n"
            )),
        );
        let untouched = add_profile(dir.path(), "other.default", Some("keep\n"));
        FirefoxTarget.apply(&ctx).unwrap();
        let out = std::fs::read_to_string(colored.join("user.js")).unwrap();
        assert!(out.contains("user_pref(\"ui.-moz-headerbar\", \"\");"));
        assert_eq!(
            std::fs::read_to_string(untouched.join("user.js")).unwrap(),
            "keep\n"
        );
    }

    #[test]
    fn skips_symlinked_user_js() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = setup(dir.path(), true);
        let profile = add_profile(dir.path(), "linked.default", None);
        let target = dir.path().join("real-user.js");
        std::fs::write(&target, "untouched\n").unwrap();
        std::os::unix::fs::symlink(&target, profile.join("user.js")).unwrap();
        FirefoxTarget.apply(&ctx).unwrap();
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "untouched\n");
    }

    #[test]
    fn under_white_matches_firefox_mix() {
        assert_eq!(under_white("#333333", 0.15), "#0F0F0F");
        assert_eq!(under_white("#101010", 0.15), "#010101");
    }
}
