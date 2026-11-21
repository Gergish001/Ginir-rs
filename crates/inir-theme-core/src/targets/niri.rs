//! Niri focus-ring colour from the iNiR accent.

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
        "Darken/saturate the iNiR accent for Niri's focus-ring active-color"
    }

    fn apply(&self, ctx: &ThemeContext) -> Result<Report> {
        let mut report = Report::new(self.id());
        let path = ctx.niri_file.clone().or_else(|| {
            let dir = ctx.home.join(".config/niri/config.d");
            let a = dir.join("20-layout-and-overview.kdl");
            if a.exists() {
                Some(a)
            } else {
                let b = ctx.home.join(".config/niri/config.kdl");
                b.exists().then_some(b)
            }
        });
        let Some(path) = path else {
            report.notes.push("no niri focus-ring file found".into());
            return Ok(report);
        };
        let old = std::fs::read_to_string(&path)?;
        let bg = ctx.palette.background;
        let color = ctx.accent().adjust(
            ctx.options.niri_darken,
            ctx.options.niri_saturate,
            Role::Accent,
            Some(bg),
            Mode::Readable,
        );
        let (new, changed) = update_focus_ring(&old, color.to_hex().as_str());
        if !changed {
            report
                .notes
                .push("focus-ring active-color not found".into());
            return Ok(report);
        }
        if ctx.write(&path, &new, &mut report)? {
            report
                .notes
                .push(format!("focus-ring -> {}", color.to_hex()));
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

    #[test]
    fn rewrites_first_focus_ring() {
        let src = "layout {\n    focus-ring {\n        width 2\n        active-color   \"#111111\"\n    }\n    focus-ring {\n        active-color \"#222222\"\n    }\n}\n";
        let (out, changed) = update_focus_ring(src, "#abcdef");
        assert!(changed);
        assert!(out.contains("active-color   \"#abcdef\""));
        assert!(out.contains("active-color \"#222222\""));
    }
}
