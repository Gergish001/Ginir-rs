//! Chromium family: BrowserThemeColor policy and the profile colour scheme.

use std::collections::BTreeSet;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde_json::{json, Value};

use crate::context::{Report, ThemeContext};
use crate::targets::Target;

const POLICY_FILE: &str = "ii-theme.json";

struct Browser {
    policy_dir: PathBuf,
    data_dir: PathBuf,
    markers: Vec<String>,
}

pub struct ChromeTarget;

impl Target for ChromeTarget {
    fn id(&self) -> &'static str {
        "chrome"
    }
    fn label(&self) -> &'static str {
        "Chromium browsers"
    }
    fn description(&self) -> &'static str {
        "Write BrowserThemeColor and the profile colour scheme for Chrome, Chromium, Brave and friends"
    }

    fn apply(&self, ctx: &ThemeContext) -> Result<Report> {
        let mut report = Report::new(self.id());
        let installed: Vec<Browser> = browsers(&ctx.home)
            .into_iter()
            .filter(|browser| is_installed(browser, &ctx.home))
            .collect();
        if installed.is_empty() {
            report
                .notes
                .push("no installed Chromium browsers found".into());
            return Ok(report);
        }

        if !chrome_enabled(&ctx.home) {
            for browser in &installed {
                let file = browser.policy_dir.join(POLICY_FILE);
                if !file.exists() {
                    continue;
                }
                match std::fs::remove_file(&file) {
                    Ok(()) => report.notes.push(format!("removed {}", file.display())),
                    Err(err) => report
                        .notes
                        .push(format!("cannot remove {}: {err}", file.display())),
                }
            }
            return Ok(report);
        }

        let Some(colour) = header_colour(&ctx.home) else {
            report.notes.push("no app palette yet; skipped".into());
            return Ok(report);
        };
        let scheme = colour_scheme(&ctx.home);
        let policy = format!(
            "{}\n",
            serde_json::to_string(&json!({
                "BrowserThemeColor": colour,
                "BackgroundModeEnabled": false
            }))?
        );

        let mut dirs = BTreeSet::new();
        for browser in &installed {
            if dirs.insert(browser.policy_dir.clone()) {
                let file = browser.policy_dir.join(POLICY_FILE);
                match ctx.write(&file, &policy, &mut report) {
                    Ok(true) => report
                        .notes
                        .push(format!("wrote {} ({colour})", file.display())),
                    Ok(false) => {}
                    Err(err) => report.notes.push(format!("{}: {err}", file.display())),
                }
            }
            update_preferences(ctx, &browser.data_dir, scheme, &mut report);
        }
        Ok(report)
    }
}

fn chrome_enabled(home: &Path) -> bool {
    let path = home.join(".config/illogical-impulse/config.json");
    let Some(value) = std::fs::read(path)
        .ok()
        .and_then(|raw| serde_json::from_slice::<Value>(&raw).ok())
    else {
        return true;
    };
    value
        .pointer("/appearance/wallpaperTheming/enableChrome")
        .and_then(Value::as_bool)
        .unwrap_or(true)
}

fn header_colour(home: &Path) -> Option<String> {
    let state = home.join(".local/state/quickshell/user/generated");
    for name in ["app-palette.json", "palette.json"] {
        let Some(value) = std::fs::read(state.join(name))
            .ok()
            .and_then(|raw| serde_json::from_slice::<Value>(&raw).ok())
        else {
            continue;
        };
        for key in [
            "app_headerbar_bg",
            "app_surface",
            "surface_container_low",
            "surface",
            "background",
        ] {
            if let Some(raw) = value.get(key).and_then(Value::as_str) {
                if is_hex(raw) {
                    return Some(raw.to_string());
                }
            }
        }
    }
    None
}

fn colour_scheme(home: &Path) -> i64 {
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
        1
    } else {
        2
    }
}

fn update_preferences(ctx: &ThemeContext, data_dir: &Path, scheme: i64, report: &mut Report) {
    let prefs = data_dir.join("Default/Preferences");
    if !prefs.is_file() {
        return;
    }
    if std::fs::symlink_metadata(data_dir.join("SingletonLock"))
        .map(|meta| meta.file_type().is_symlink())
        .unwrap_or(false)
    {
        report.notes.push(format!(
            "{}: browser is running; left alone",
            prefs.display()
        ));
        return;
    }
    let mut value: Value = match std::fs::read(&prefs)
        .ok()
        .and_then(|raw| serde_json::from_slice(&raw).ok())
    {
        Some(value) => value,
        None => {
            report
                .notes
                .push(format!("{}: unreadable; left alone", prefs.display()));
            return;
        }
    };
    let Some(root) = value.as_object_mut() else {
        report
            .notes
            .push(format!("{}: not an object; left alone", prefs.display()));
        return;
    };
    let browser = root
        .entry("browser")
        .or_insert_with(|| json!({}))
        .as_object_mut();
    let Some(browser) = browser else {
        report.notes.push(format!(
            "{}: browser is not an object; left alone",
            prefs.display()
        ));
        return;
    };
    let theme = browser
        .entry("theme")
        .or_insert_with(|| json!({}))
        .as_object_mut();
    let Some(theme) = theme else {
        report.notes.push(format!(
            "{}: theme is not an object; left alone",
            prefs.display()
        ));
        return;
    };
    if theme.get("color_scheme").and_then(Value::as_i64) == Some(scheme)
        && theme.get("color_scheme2").and_then(Value::as_i64) == Some(scheme)
    {
        return;
    }
    theme.insert("color_scheme".into(), json!(scheme));
    theme.insert("color_scheme2".into(), json!(scheme));
    let text = match serde_json::to_string(&value) {
        Ok(text) => text + "\n",
        Err(err) => {
            report
                .notes
                .push(format!("{}: cannot encode: {err}", prefs.display()));
            return;
        }
    };
    match ctx.write(&prefs, &text, report) {
        Ok(true) => report.notes.push(format!(
            "wrote {} (colour scheme {scheme})",
            prefs.display()
        )),
        Ok(false) => {}
        Err(err) => report.notes.push(format!("{}: {err}", prefs.display())),
    }
}

fn browsers(home: &Path) -> Vec<Browser> {
    let config = home.join(".config");
    let var = home.join(".var/app");
    let ext = home.join(".local/share/flatpak/extension");
    let arch = std::env::consts::ARCH;
    let chrome = PathBuf::from("/etc/opt/chrome/policies/managed");
    let chromium = PathBuf::from("/etc/chromium/policies/managed");
    let brave = PathBuf::from("/etc/brave/policies/managed");
    vec![
        browser(chrome.clone(), config.join("google-chrome"), &["google-chrome-stable", "google-chrome"]),
        browser(chrome.clone(), config.join("google-chrome-beta"), &["google-chrome-beta"]),
        browser(chrome.clone(), config.join("google-chrome-unstable"), &["google-chrome-unstable"]),
        browser(chromium.clone(), config.join("chromium"), &["chromium", "chromium-browser"]),
        browser(brave.clone(), config.join("BraveSoftware/Brave-Browser"), &["brave", "brave-browser"]),
        browser(chromium.clone(), config.join("net.imput.helium"), &["helium", "helium-browser"]),
        browser(chromium.clone(), config.join("thorium"), &["thorium-browser", "thorium"]),
        browser(
            chrome.clone(),
            var.join("com.google.Chrome/config/google-chrome"),
            &["flatpak:com.google.Chrome"],
        ),
        browser(
            brave.clone(),
            var.join("com.brave.Browser/config/BraveSoftware/Brave-Browser"),
            &["flatpak:com.brave.Browser"],
        ),
        browser(
            ext.join(format!(
                "org.chromium.Chromium.Extension.inir/{arch}/1/policies/managed"
            )),
            var.join("org.chromium.Chromium/config/chromium"),
            &["flatpak:org.chromium.Chromium"],
        ),
        browser(
            ext.join(format!(
                "io.github.ungoogled_software.ungoogled_chromium.Extension.inir/{arch}/1/policies/managed"
            )),
            var.join("io.github.ungoogled_software.ungoogled_chromium/config/chromium"),
            &["flatpak:io.github.ungoogled_software.ungoogled_chromium"],
        ),
    ]
}

fn browser(policy_dir: PathBuf, data_dir: PathBuf, markers: &[&str]) -> Browser {
    Browser {
        policy_dir,
        data_dir,
        markers: markers.iter().map(|marker| marker.to_string()).collect(),
    }
}

fn is_installed(browser: &Browser, home: &Path) -> bool {
    for marker in &browser.markers {
        if let Some(id) = marker.strip_prefix("flatpak:") {
            let local = home.join(".local/share/flatpak/app").join(id);
            let system = Path::new("/var/lib/flatpak/app").join(id);
            if local.is_dir() || system.is_dir() {
                return true;
            }
        } else if command_exists(marker) {
            return true;
        }
    }
    false
}

fn command_exists(name: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| {
        let path = dir.join(name);
        path.is_file()
            && path
                .metadata()
                .map(|meta| meta.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
    })
}

fn is_hex(raw: &str) -> bool {
    raw.len() == 7 && raw.starts_with('#') && raw[1..].chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::Palette;

    fn setup(dir: &Path, enabled: bool) -> ThemeContext {
        let state = dir.join(".local/state/quickshell/user/generated");
        std::fs::create_dir_all(&state).unwrap();
        std::fs::write(
            state.join("app-palette.json"),
            serde_json::json!({"app_headerbar_bg": "#112233"}).to_string(),
        )
        .unwrap();
        std::fs::write(state.join("theme-meta.json"), r#"{"mode":"dark"}"#).unwrap();
        let config = dir.join(".config/illogical-impulse/config.json");
        std::fs::create_dir_all(config.parent().unwrap()).unwrap();
        std::fs::write(
            &config,
            serde_json::json!({
                "appearance": {"wallpaperTheming": {"enableChrome": enabled}}
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

    fn flatpak_fixture(dir: &Path) -> (PathBuf, PathBuf) {
        let app = dir.join(".local/share/flatpak/app/org.chromium.Chromium");
        std::fs::create_dir_all(&app).unwrap();
        let data = dir.join(".var/app/org.chromium.Chromium/config/chromium");
        std::fs::create_dir_all(data.join("Default")).unwrap();
        std::fs::write(
            data.join("Default/Preferences"),
            r#"{"browser":{"theme":{"color_scheme":1,"color_scheme2":1,"x":2}},"keep":true}"#,
        )
        .unwrap();
        let policy = dir.join(format!(
            ".local/share/flatpak/extension/org.chromium.Chromium.Extension.inir/{}/1/policies/managed",
            std::env::consts::ARCH
        ));
        (policy, data)
    }

    #[test]
    fn writes_flatpak_policy_and_scheme() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = setup(dir.path(), true);
        let (policy, data) = flatpak_fixture(dir.path());
        ChromeTarget.apply(&ctx).unwrap();
        let policy_file = policy.join(POLICY_FILE);
        let text = std::fs::read_to_string(&policy_file).unwrap();
        assert!(text.contains("\"BrowserThemeColor\":\"#112233\""));
        let prefs = std::fs::read_to_string(data.join("Default/Preferences")).unwrap();
        assert!(prefs.contains("\"color_scheme2\":2"));
        assert!(prefs.contains("\"keep\":true"));
    }

    #[test]
    fn disabled_removes_policy() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = setup(dir.path(), false);
        let (policy, _) = flatpak_fixture(dir.path());
        std::fs::create_dir_all(&policy).unwrap();
        std::fs::write(policy.join(POLICY_FILE), "{}\n").unwrap();
        ChromeTarget.apply(&ctx).unwrap();
        assert!(!policy.join(POLICY_FILE).exists());
    }

    #[test]
    fn leaves_running_browser_preferences_alone() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = setup(dir.path(), true);
        let (_policy, data) = flatpak_fixture(dir.path());
        let target = dir.path().join("real-lock");
        std::fs::write(&target, "").unwrap();
        std::os::unix::fs::symlink(&target, data.join("SingletonLock")).unwrap();
        ChromeTarget.apply(&ctx).unwrap();
        let prefs = std::fs::read_to_string(data.join("Default/Preferences")).unwrap();
        assert!(prefs.contains("\"color_scheme2\":1"));
    }

    #[test]
    fn header_colour_prefers_the_app_header() {
        let dir = tempfile::tempdir().unwrap();
        setup(dir.path(), true);
        assert_eq!(header_colour(dir.path()).as_deref(), Some("#112233"));
    }
}
