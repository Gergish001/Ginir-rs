//! Rust theming core for iNiR.
//!
//! This crate contains the colour maths and target generators that used to be
//! split between Bash modules and a Python helper. The QML UI itself is not
//! reimplemented here: iNiR's runtime layer (`qs-rs`) already runs the QML.

pub mod color;
pub mod context;
pub mod palette;
pub mod targets;

pub use color::{Color, Mode, Role};
pub use context::{Options, Report, ThemeContext};
pub use palette::Palette;

/// Version reported by `ginir-theme --version`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
