//! Niri focus-ring colour from the iNiR accent.

use std::path::PathBuf;
use std::process::Command;

use anyhow::Result;

use crate::color::{Mode, Role};
use crate::context::{Report, ThemeContext};
use crate::targets::Target;

pub struct NiriTarget;

impl Target for NiriTarget {
    fn id(&self) -> &'static str {
        "niri"
    }
    fn label(&self) -> &'static str {
        "Niri focus ring"
    }
    fn description(&self) -> &'static str {
        "Darken/saturate the iNiR accent for Niri's focus-ring active-color in every config file"
    }

    fn apply(&self, ctx: &ThemeContext) -> Result<Report> {
        let mut report = Report::new(self.id());
        let files = candidate_files(ctx);
        if files.is_empty() {
            report.notes.push("no niri config files found".into());
            return Ok(report);
        }
        let bg = ctx.palette.background;
        let color = ctx.accent().adjust(
            ctx.options.niri_darken,
            ctx.options.niri_saturate,
            Role::Accent,
            Some(bg),
            Mode::Readable,
        );

        let mut updated = 0usize;
        for path in files {
            let old = std::fs::read_to_string(&path)?;
            let (new, changed) = update_focus_ring(&old, color.to_hex().as_str());
            if !changed {
                continue;
            }
            if ctx.write(&path, &new, &mut report)? {
                updated += 1;
                report.notes.push(format!(
                    "{}: focus-ring -> {}",
                    path.display(),
                    color.to_hex()
                ));
            }
        }

        if updated == 0 {
            report
                .notes
                .push("focus-ring active-color not found".into());
            return Ok(report);
        }

        if ctx.reload && !ctx.dry_run {
            let config = ctx.home.join(".config/niri/config.kdl");
            if config.exists() {
                let _ = Command::new("niri")
                    .args(["validate", "-c"])
                    .arg(&config)
                    .output();
            }
            let _ = Command::new("niri")
                .args(["msg", "action", "load-config-file"])
                .output();
        }
        Ok(report)
    }
}

/// `--niri-file` wins; otherwise scan `config.kdl` and `config.d/*.kdl`.
fn candidate_files(ctx: &ThemeContext) -> Vec<PathBuf> {
    if let Some(path) = &ctx.niri_file {
        return if path.exists() {
            vec![path.clone()]
        } else {
            Vec::new()
        };
    }
    let dir = ctx.home.join(".config/niri");
    let mut files = Vec::new();
    let main = dir.join("config.kdl");
    if main.exists() {
        files.push(main);
    }
    if let Ok(entries) = std::fs::read_dir(dir.join("config.d")) {
        let mut extra: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("kdl"))
            .collect();
        extra.sort();
        files.extend(extra);
    }
    files
}

fn update_focus_ring(text: &str, color: &str) -> (String, bool) {
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let mut in_ring = false;
    let mut changed = false;
    for line in &mut lines {
        let trimmed = line.trim_start().to_string();
        if trimmed.starts_with("focus-ring") && trimmed.contains('{') {
            in_ring = true;
            continue;
        }
        if in_ring && !changed && trimmed.starts_with("active-color") {
            let indent = &line[..line.len() - trimmed.len()];
            *line = format!("{indent}active-color   \"{color}\"");
            changed = true;
        }
        if in_ring && trimmed.starts_with('}') {
            in_ring = false;
        }
    }
    let mut out = lines.join("\n");
    out.push('\n');
    (out, changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::Palette;

    #[test]
    fn rewrites_first_focus_ring() {
        let src = "layout {\n    focus-ring {\n        width 2\n        active-color   \"#111111\"\n    }\n    focus-ring {\n        active-color \"#222222\"\n    }\n}\n";
        let (out, changed) = update_focus_ring(src, "#abcdef");
        assert!(changed);
        assert!(out.contains("active-color   \"#abcdef\""));
        assert!(out.contains("active-color \"#222222\""));
    }

    #[test]
    fn updates_every_niri_file() {
        let dir = tempfile::tempdir().unwrap();
        let niri = dir.path().join(".config/niri");
        let config_d = niri.join("config.d");
        std::fs::create_dir_all(&config_d).unwrap();
        std::fs::write(
            niri.join("config.kdl"),
            "layout {\n    focus-ring {\n        active-color \"#111111\"\n    }\n}\n",
        )
        .unwrap();
        std::fs::write(
            config_d.join("20-layout-and-overview.kdl"),
            "layout {\n    focus-ring {\n        active-color \"#222222\"\n    }\n}\n",
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

        NiriTarget.apply(&ctx).unwrap();

        let main = std::fs::read_to_string(niri.join("config.kdl")).unwrap();
        let dropin = std::fs::read_to_string(config_d.join("20-layout-and-overview.kdl")).unwrap();
        let color = main
            .lines()
            .find(|line| line.trim_start().starts_with("active-color"))
            .and_then(|line| line.split('"').nth(1))
            .unwrap()
            .to_string();
        assert_ne!(color, "#111111");
        assert_ne!(color, "#222222");
        assert!(dropin.contains(&color));
    }
}
