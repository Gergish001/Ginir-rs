//! `ginir-theme` CLI.

use std::path::PathBuf;
use std::process::Command;

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};
use ginir_theme_core::color::Color;
use ginir_theme_core::context::{Options, ThemeContext};
use ginir_theme_core::palette::Palette;
use ginir_theme_core::targets;

#[derive(Parser, Debug)]
#[command(
    name = "ginir-theme",
    version,
    about = "Rust theming targets for iNiR (herdr, opencode, niri, umbriel, chrome, firefox, fetch, cmatrix/lavat)"
)]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand, Debug)]
enum CliCommand {
    /// List available targets.
    List,
    /// Apply one target, or all targets.
    Apply(ApplyArgs),
    /// Check the palette and external tools.
    Doctor(DoctorArgs),
}

#[derive(Args, Debug, Clone)]
struct SourceArgs {
    /// Override HOME (useful for tests).
    #[arg(long)]
    home: Option<PathBuf>,
    /// iNiR app-palette.json.
    #[arg(long)]
    app_palette: Option<PathBuf>,
    /// iNiR terminal.json.
    #[arg(long)]
    terminal_json: Option<PathBuf>,
    /// Fallback kitty theme.conf.
    #[arg(long)]
    kitty: Option<PathBuf>,
    /// Override the accent, e.g. #b78cda.
    #[arg(long)]
    accent: Option<String>,
}

#[derive(Args, Debug)]
struct ApplyArgs {
    #[command(flatten)]
    source: SourceArgs,
    /// Target id (herdr, opencode, niri, umbriel, fetch3d, launchers).
    #[arg(long)]
    target: Option<String>,
    /// Apply every target.
    #[arg(long)]
    all: bool,
    /// Show what would change without writing.
    #[arg(long)]
    dry_run: bool,
    /// Do not reload herdr/niri after writing.
    #[arg(long)]
    no_reload: bool,
    /// Niri layout file override.
    #[arg(long)]
    niri_file: Option<PathBuf>,
    /// Herdr darkening (default 0.40).
    #[arg(long)]
    herdr_darken: Option<f64>,
    /// Herdr saturation boost (default 0.20).
    #[arg(long)]
    herdr_sat: Option<f64>,
}

#[derive(Args, Debug)]
struct DoctorArgs {
    #[command(flatten)]
    source: SourceArgs,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        CliCommand::List => list(),
        CliCommand::Apply(args) => apply(args),
        CliCommand::Doctor(args) => doctor(args),
    }
}

fn list() -> Result<()> {
    for target in targets::all() {
        println!(
            "{:<10} {:<20} {}",
            target.id(),
            target.label(),
            target.description()
        );
    }
    Ok(())
}

fn apply(args: ApplyArgs) -> Result<()> {
    let mut ctx = context(&args.source)?;
    ctx.dry_run = args.dry_run;
    ctx.reload = !args.no_reload;
    ctx.niri_file = args.niri_file;
    if let Some(v) = args.herdr_darken {
        ctx.options.herdr_darken = v;
    }
    if let Some(v) = args.herdr_sat {
        ctx.options.herdr_saturate = v;
    }

    let selected: Vec<Box<dyn targets::Target>> = if args.all {
        targets::all()
    } else if let Some(id) = args.target.as_deref() {
        match targets::find(id) {
            Some(t) => vec![t],
            None => bail!("unknown target: {id} (try `ginir-theme list`)"),
        }
    } else {
        bail!("pass --target <id> or --all")
    };

    for target in selected {
        let report = target
            .apply(&ctx)
            .with_context(|| format!("apply {}", target.id()))?;
        let mark = if report.changed { "✓" } else { "•" };
        let state = if report.changed {
            if ctx.dry_run {
                "planned"
            } else {
                "updated"
            }
        } else {
            "no changes"
        };
        println!("{mark} {}: {state}", report.target);
        for file in &report.files {
            println!("  file: {}", file.display());
        }
        for note in &report.notes {
            println!("  note: {note}");
        }
    }
    Ok(())
}

fn doctor(args: DoctorArgs) -> Result<()> {
    let ctx = context(&args.source)?;
    println!("palette:");
    println!("  background {}", ctx.palette.background.to_hex());
    println!("  foreground {}", ctx.palette.foreground.to_hex());
    println!("  accent     {}", ctx.accent.to_hex());
    println!("  terminal   {} colours", ctx.palette.terminal.len());
    println!("targets:");
    for target in targets::all() {
        println!("  {:<10} {}", target.id(), target.label());
    }
    println!("commands:");
    for cmd in ["herdr", "niri", "opencode", "fetch", "cmatrix", "lavat"] {
        let found = Command::new("sh")
            .args(["-c", &format!("command -v {cmd}")])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        println!("  {:<8} {}", cmd, if found { "ok" } else { "missing" });
    }
    Ok(())
}

fn context(source: &SourceArgs) -> Result<ThemeContext> {
    let home = match &source.home {
        Some(p) => p.clone(),
        None => std::env::var_os("HOME")
            .map(PathBuf::from)
            .context("HOME is not set")?,
    };
    let palette = Palette::load(
        &home,
        source.app_palette.as_deref(),
        source.terminal_json.as_deref(),
        source.kitty.as_deref(),
    )?;
    let mut ctx = ThemeContext::new(palette);
    ctx.home = home;
    ctx.backup_dir = ctx.home.join("gentoo-opt-backups");
    if let Some(raw) = &source.accent {
        let accent = Color::from_hex(raw).with_context(|| format!("invalid --accent {raw}"))?;
        ctx.accent = accent;
    }
    ctx.options = Options::default();
    Ok(ctx)
}
