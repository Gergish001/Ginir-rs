//! Shared context and reporting for target generators.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};

use crate::color::Color;
use crate::palette::Palette;

#[derive(Clone, Debug)]
pub struct Options {
    pub herdr_darken: f64,
    pub herdr_saturate: f64,
    pub opencode_darken: f64,
    pub opencode_saturate: f64,
    pub niri_darken: f64,
    pub niri_saturate: f64,
    pub fetch_darken: f64,
    pub accent_exact: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            herdr_darken: 0.40,
            herdr_saturate: 0.20,
            opencode_darken: 0.60,
            opencode_saturate: 0.50,
            niri_darken: 0.20,
            niri_saturate: 0.34,
            fetch_darken: 0.10,
            accent_exact: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ThemeContext {
    pub palette: Palette,
    pub accent: Color,
    pub home: PathBuf,
    pub dry_run: bool,
    pub reload: bool,
    pub backup_dir: PathBuf,
    pub niri_file: Option<PathBuf>,
    pub options: Options,
}

impl ThemeContext {
    pub fn new(palette: Palette) -> Self {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let backup_dir = home.join("gentoo-opt-backups");
        let accent = palette.accent;
        Self {
            palette,
            accent,
            home,
            dry_run: false,
            reload: true,
            backup_dir,
            niri_file: None,
            options: Options::default(),
        }
    }

    pub fn accent(&self) -> Color {
        if self.options.accent_exact {
            self.accent
        } else {
            self.accent.adjust(
                self.options.herdr_darken,
                self.options.herdr_saturate,
                crate::color::Role::Accent,
                Some(self.palette.background),
                crate::color::Mode::Readable,
            )
        }
    }

    pub fn write(&self, path: &Path, content: &str, report: &mut Report) -> Result<bool> {
        if path.exists() {
            let old = std::fs::read_to_string(path).unwrap_or_default();
            if old == content {
                return Ok(false);
            }
        }
        if self.dry_run {
            report.notes.push(format!("would write {}", path.display()));
            report.files.push(path.to_path_buf());
            report.changed = true;
            return Ok(true);
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        if path.exists() {
            self.backup(path)?;
        }
        std::fs::write(path, content).with_context(|| format!("write {}", path.display()))?;
        report.files.push(path.to_path_buf());
        report.changed = true;
        Ok(true)
    }

    pub fn backup(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }
        std::fs::create_dir_all(&self.backup_dir)
            .with_context(|| format!("create {}", self.backup_dir.display()))?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("config");
        let backup = self.backup_dir.join(format!("{name}.{stamp}.bak"));
        std::fs::copy(path, &backup).with_context(|| format!("backup {}", path.display()))?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct Report {
    pub target: String,
    pub changed: bool,
    pub files: Vec<PathBuf>,
    pub notes: Vec<String>,
}

impl Report {
    pub fn new(target: &str) -> Self {
        Self {
            target: target.to_string(),
            ..Default::default()
        }
    }
}
