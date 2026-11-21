//! Colour parsing, HSL transforms and contrast helpers.

use anyhow::{bail, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    Background,
    Text,
    Accent,
    Muted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    /// Keep text/accent readable by enforcing a contrast floor.
    Readable,
    /// Apply the requested transform literally, even if text gets dark.
    Literal,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn from_hex(raw: &str) -> Result<Self> {
        let s = raw.trim().trim_start_matches('#');
        let expanded;
        let s = match s.len() {
            3 => {
                expanded = s.chars().flat_map(|c| [c, c]).collect::<String>();
                expanded.as_str()
            }
            6 => s,
            8 => &s[..6],
            _ => bail!("invalid hex colour: {raw}"),
        };
        let r = u8::from_str_radix(&s[0..2], 16)?;
        let g = u8::from_str_radix(&s[2..4], 16)?;
        let b = u8::from_str_radix(&s[4..6], 16)?;
        Ok(Self { r, g, b })
    }

    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub fn to_hex_bare(self) -> String {
        format!("{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub fn to_sgr(self) -> String {
        format!("38;2;{};{};{}", self.r, self.g, self.b)
    }

    pub fn hsl(self) -> (f64, f64, f64) {
        let r = self.r as f64 / 255.0;
        let g = self.g as f64 / 255.0;
        let b = self.b as f64 / 255.0;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;
        if (max - min).abs() < f64::EPSILON {
            return (0.0, l, 0.0);
        }
        let d = max - min;
        let s = if l > 0.5 {
            d / (2.0 - max - min)
        } else {
            d / (max + min)
        };
        let h = if (max - r).abs() < f64::EPSILON {
            (g - b) / d + if g < b { 6.0 } else { 0.0 }
        } else if (max - g).abs() < f64::EPSILON {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };
        (h / 6.0, l, s)
    }

    pub fn from_hsl(h: f64, l: f64, s: f64) -> Self {
        let h = h.rem_euclid(1.0);
        if s <= f64::EPSILON {
            let v = (l * 255.0).round() as u8;
            return Self::new(v, v, v);
        }
        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;
        let channel = |t: f64| -> u8 {
            let mut t = t;
            if t < 0.0 {
                t += 1.0;
            }
            if t > 1.0 {
                t -= 1.0;
            }
            let v = if t < 1.0 / 6.0 {
                p + (q - p) * 6.0 * t
            } else if t < 0.5 {
                q
            } else if t < 2.0 / 3.0 {
                p + (q - p) * (2.0 / 3.0 - t) * 6.0
            } else {
                p
            };
            (v * 255.0).round().clamp(0.0, 255.0) as u8
        };
        Self::new(channel(h + 1.0 / 3.0), channel(h), channel(h - 1.0 / 3.0))
    }

    pub fn luminance(self) -> f64 {
        fn f(v: u8) -> f64 {
            let v = v as f64 / 255.0;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * f(self.r) + 0.7152 * f(self.g) + 0.0722 * f(self.b)
    }

    pub fn contrast(self, other: Self) -> f64 {
        let a = self.luminance();
        let b = other.luminance();
        let (hi, lo) = if a >= b { (a, b) } else { (b, a) };
        (hi + 0.05) / (lo + 0.05)
    }

    pub fn mix(self, other: Self, t: f64) -> Self {
        let t = t.clamp(0.0, 1.0);
        let lerp = |a: u8, b: u8| -> u8 { (a as f64 * (1.0 - t) + b as f64 * t).round() as u8 };
        Self::new(
            lerp(self.r, other.r),
            lerp(self.g, other.g),
            lerp(self.b, other.b),
        )
    }

    /// Darken by `darken` (0..1) and increase saturation by `saturate` (0..1).
    pub fn adjust(
        self,
        darken: f64,
        saturate: f64,
        role: Role,
        bg: Option<Self>,
        mode: Mode,
    ) -> Self {
        let (h, l, s) = self.hsl();
        let l = (l * (1.0 - darken)).clamp(0.0, 1.0);
        let s = (s * (1.0 + saturate)).clamp(0.0, 1.0);
        let out = Self::from_hsl(h, l, s);
        match (mode, bg) {
            (Mode::Readable, Some(bg)) => {
                let target = match role {
                    Role::Background => 0.0,
                    Role::Text => 4.5,
                    Role::Accent => 3.0,
                    Role::Muted => 2.5,
                };
                ensure_contrast(out, bg, target)
            }
            _ => out,
        }
    }

    pub fn nearest_ansi_name(self) -> &'static str {
        const NAMES: [&str; 8] = [
            "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
        ];
        const RGB: [(u8, u8, u8); 8] = [
            (0, 0, 0),
            (205, 49, 49),
            (13, 188, 121),
            (229, 229, 16),
            (36, 114, 200),
            (188, 63, 188),
            (17, 168, 205),
            (229, 229, 229),
        ];
        let mut best = 0usize;
        let mut best_d = f64::MAX;
        for (i, (r, g, b)) in RGB.iter().enumerate() {
            let d = (self.r as f64 - *r as f64).powi(2)
                + (self.g as f64 - *g as f64).powi(2)
                + (self.b as f64 - *b as f64).powi(2);
            if d < best_d {
                best_d = d;
                best = i;
            }
        }
        NAMES[best]
    }
}

fn ensure_contrast(mut color: Color, bg: Color, target: f64) -> Color {
    if target <= 0.0 || color.contrast(bg) >= target {
        return color;
    }
    let (h, mut l, s) = color.hsl();
    let step = if bg.luminance() < 0.5 { 0.01 } else { -0.01 };
    for _ in 0..100 {
        l = (l + step).clamp(0.0, 1.0);
        color = Color::from_hsl(h, l, s);
        if color.contrast(bg) >= target {
            return color;
        }
        if l <= 0.0 || l >= 1.0 {
            break;
        }
    }
    if bg.luminance() < 0.5 {
        Color::new(255, 255, 255)
    } else {
        Color::new(0, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_formats_hex() {
        let c = Color::from_hex("#b78cda").unwrap();
        assert_eq!(c, Color::new(183, 140, 218));
        assert_eq!(c.to_hex(), "#b78cda");
        assert_eq!(c.to_sgr(), "38;2;183;140;218");
        assert_eq!(Color::from_hex("abc").unwrap(), Color::new(170, 187, 204));
    }

    #[test]
    fn darkens_and_saturates() {
        let c = Color::from_hex("#b78cda").unwrap();
        let out = c.adjust(0.40, 0.20, Role::Accent, None, Mode::Literal);
        assert!(out.hsl().1 < c.hsl().1);
    }

    #[test]
    fn contrast_floor_is_respected() {
        let bg = Color::new(10, 8, 10);
        let text = Color::from_hex("#efebe4").unwrap();
        let out = text.adjust(0.60, 0.50, Role::Text, Some(bg), Mode::Readable);
        assert!(out.contrast(bg) >= 4.49);
    }

    #[test]
    fn nearest_colour_is_sensible() {
        assert_eq!(
            Color::from_hex("#d000d0").unwrap().nearest_ansi_name(),
            "magenta"
        );
    }
}
