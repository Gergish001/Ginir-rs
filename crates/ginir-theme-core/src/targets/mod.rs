//! Target registry.

pub mod chrome;
pub mod fetch;
pub mod firefox;
pub mod herdr;
pub mod launchers;
pub mod limusic;
pub mod niri;
pub mod opencode;
pub mod umbriel;

use anyhow::Result;

use crate::context::{Report, ThemeContext};

pub trait Target: Send + Sync {
    fn id(&self) -> &'static str;
    fn label(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn apply(&self, ctx: &ThemeContext) -> Result<Report>;
}

pub fn all() -> Vec<Box<dyn Target>> {
    vec![
        Box::new(herdr::HerdrTarget),
        Box::new(opencode::OpencodeTarget),
        Box::new(niri::NiriTarget),
        Box::new(umbriel::UmbrielTarget),
        Box::new(firefox::FirefoxTarget),
        Box::new(chrome::ChromeTarget),
        Box::new(limusic::LimusicTarget),
        Box::new(fetch::FetchTarget),
        Box::new(launchers::LaunchersTarget),
    ]
}

pub fn find(id: &str) -> Option<Box<dyn Target>> {
    all().into_iter().find(|t| t.id() == id)
}
