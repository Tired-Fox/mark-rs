use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::iter;

pub use crate::_color as color;
use crate::style::flags::{BLINK, BOLD, CROSSED, ITALIC, RESET, REVERSED, StyleFlag, UNDERLINE};

pub mod flags;

pub trait AnsiSequence {
    fn ansi(&self) -> String;
    fn sequence(&self) -> String {
        format!("\x1b[{}m", self.ansi())
    }
    fn reset_ansi(&self) -> String;
    fn reset_sequence(&self) -> String {
        format!("\x1b[{}m", self.reset_ansi())
    }
}

#[derive(Default, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Hyperlink(pub String);
impl Display for Hyperlink {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.sign_minus() {
            write!(f, "{}", self.reset_sequence())
        } else {
            write!(f, "{}", self.sequence())
        }
    }
}

impl From<&str> for Hyperlink {
    fn from(d: &str) -> Self {
        Hyperlink(d.to_string())
    }
}

impl From<String> for Hyperlink {
    fn from(value: String) -> Self {
        Hyperlink(value)
    }
}

impl AnsiSequence for Hyperlink {
    fn ansi(&self) -> String {
        String::new()
    }

    fn reset_ansi(&self) -> String {
        String::new()
    }

    fn sequence(&self) -> String {
        format!("\x1b]8;;{}\x1b\\", self.0)
    }

    fn reset_sequence(&self) -> String {
        "\x1b]8;;\x1b\\".into()
    }
}

/// Terminal color representation.
///
/// Supports named system colors, XTerm/Ansi colors (0-255), and RGB colors (0-255,0-255,0-255).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    BLACK,
    RED,
    GREEN,
    YELLOW,
    BLUE,
    MAGENTA,
    CYAN,
    WHITE,
    /// 0<=value<=255
    Ansi(u8),
    /// 0<=R<=255, 0<=G<=255, 0<=B<=255
    RGB { r: u8, g: u8, b: u8 },
    /// 0<=H<360, 0<=S<=1, 0<=L<=1
    HSL { h: u16, s: f32, l: f32 },
    HSV { h: u16, s: f32, v: f32 },
    CYMK { c: f32, y: f32, m: f32, k: f32 },
}

impl Hash for Color {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::BLACK => "0".hash(state),
            Self::RED => "1".hash(state),
            Self::GREEN => "2".hash(state),
            Self::YELLOW => "3".hash(state),
            Self::BLUE => "4".hash(state),
            Self::MAGENTA => "5".hash(state),
            Self::CYAN => "6".hash(state),
            Self::WHITE => "7".hash(state),
            Color::Ansi(c) => c.hash(state),
            Color::RGB { r, g, b } => {
                r.hash(state);
                g.hash(state);
                b.hash(state);
            }
            Color::HSL { h, s, l } => {
                h.hash(state);
                s.to_bits().hash(state);
                l.to_bits().hash(state);
            }
            Color::HSV { h, s, v } => {
                h.hash(state);
                s.to_bits().hash(state);
                v.to_bits().hash(state);
            }
            Color::CYMK { c, y, m, k } => {
                c.to_bits().hash(state);
                y.to_bits().hash(state);
                m.to_bits().hash(state);
                k.to_bits().hash(state);
            }
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.sign_minus() {
            write!(f, "\x1b[{}m", if f.alternate() { self.reset_bg() } else { self.reset_fg() })
        } else {
            write!(f, "\x1b[{}m", if f.alternate() { self.bg() } else { self.fg() })
        }
    }
}

impl From<u8> for Color {
    fn from(value: u8) -> Self {
        Color::Ansi(value)
    }
}

impl From<String> for Color {
    fn from(value: String) -> Self {
        Color::from(value.as_str())
    }
}

impl From<&str> for Color {
    fn from(value: &str) -> Self {
        let mut value = value.strip_prefix("#").unwrap_or(value).to_string();
        // If shorthand then first extend
        if value.len() == 3 || value.len() == 4 {
            value = value.chars().flat_map(|c| iter::repeat(c).take(2)).collect::<String>()
        }
        let bytes: [u8; 4] = u32::from_str_radix(&value, 16).unwrap().to_be_bytes();
        Color::RGB { r: bytes[1], g: bytes[2], b: bytes[3] }
    }
}

impl AnsiSequence for Color {
    fn ansi(&self) -> String {
        match self {
            Self::BLACK => "0".to_string(),
            Self::RED => "1".to_string(),
            Self::GREEN => "2".to_string(),
            Self::YELLOW => "3".to_string(),
            Self::BLUE => "4".to_string(),
            Self::MAGENTA => "5".to_string(),
            Self::CYAN => "6".to_string(),
            Self::WHITE => "7".to_string(),
            Color::Ansi(value) => format!("8;5;{}", value),
            Color::RGB { r, g, b } => format!("8;2;{};{};{}", r, g, b),
            Color::HSV { h, s, v } => {
                // TODO: Convert from hsl to rgb
                let c = v * s;
                let h = *h as f32 / 60.0;
                let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
                let m = v - c;

                format_hs_color(c, h, x, m)
            }
            Color::HSL { h, s, l } => {
                // chroma
                let c = (1.0 - ((2.0 * l) - 1.0).abs()) * s;
                let h = *h as f32 / 60.0;
                let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
                let m = l - (c / 2.0);

                format_hs_color(c, h, x, m)
            }
            Color::CYMK { c, y, m, k } => {
                let kp = 1.0 - k;
                let r = 255.0 * (1.0 - c) * kp;
                let g = 255.0 * (1.0 - y) * kp;
                let b = 255.0 * (1.0 - m) * kp;
                format!("8;2;{};{};{}", r as u8, g as u8, b as u8)
            }
        }
    }

    fn reset_ansi(&self) -> String {
        String::from("9")
    }
}

fn format_hs_color(c: f32, h: f32, x: f32, m: f32) -> String {
    let (r, g, b) = match h {
        0.0..=1.0 => (c, x, 0.0),
        1.0..=2.0 => (x, c, 0.0),
        2.0..=3.0 => (0.0, c, x),
        3.0..=4.0 => (0.0, x, c),
        4.0..=5.0 => (x, 0.0, c),
        5.0..=6.0 => (c, 0.0, x),
        _ => {
            panic!("Hue in hsl is greater than or equal to 360 which is out of bounds")
        }
    };

    format!(
        "8;2;{};{};{}",
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8
    )
}

impl Color {
    pub fn fg(&self) -> String {
        format!("3{}", self.ansi())
    }
    pub fn bg(&self) -> String {
        format!("4{}", self.ansi())
    }
    pub fn reset_fg(&self) -> String {
        format!("3{}", self.reset_ansi())
    }
    pub fn reset_bg(&self) -> String {
        format!("4{}", self.reset_ansi())
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::RGB { r, g, b }
    }

    pub fn hex(hex: String) -> Self {
        Self::from(hex)
    }

    pub fn hsl(h: u16, s: f32, l: f32) -> Result<Self, String> {
        if s > 1.0 || l > 1.0 {
            return Err("Saturation or lightness in hsl is greater than to 1.0 which is out of bounds".into());
        }
        if s < 0.0 || l < 0.0 {
            return Err("Saturation or lightness in hsl is less than 0.0 which is out of bounds".into());
        }
        if h >= 360 {
            return Err("Hue in hsl is greater than or equal to 360 which is out of bounds".into());
        }
        Ok(Self::HSL { h, s, l })
    }

    pub fn hsv(h: u16, s: f32, v: f32) -> Result<Self, String> {
        if s > 1.0 || v > 1.0 {
            return Err("Saturation or Value in hsv is greater than to 1.0 which is out of bounds".into());
        }
        if s < 0.0 || v < 0.0 {
            return Err("Saturation or Value in hsv is less than 0.0 which is out of bounds".into());
        }
        if h >= 360 {
            return Err("Hue in hsv is greater than or equal to 360 which is out of bounds".into());
        }
        Ok(Self::HSV { h, s, v })
    }

    pub fn cymk(c: f32, y: f32, m: f32, k: f32) -> Result<Self, String> {
        if c < 0.0 || y < 0.0 || m < 0.0 || k < 0.0 {
            return Err("CYMK value is less than 0.0 which is out of bounds".into());
        }
        if c > 1.0 || y > 1.0 || m > 1.0 || k > 1.0 {
            return Err("CYMK value is greater than 1.0 which is out of bounds".into());
        }
        Ok(Self::CYMK { c, y, m, k })
    }
}

/// Shorthand macro for system colors, rgb, hex, hsl, hsv, xterm, and cymk colors.
///
/// Supported formats:
/// - hsl: (h, s%, l%) | (h s% l%)
/// - hsv: (h, s%, v%) | (h s% v%)
/// - rgb: (r, g, b) | (r g b)
/// - hex: #rrggbb | #rgb
/// - xterm: 0-255
#[macro_export]
macro_rules! _color {
        (hsl $h: literal, $s: literal%, $l: literal%) => {
            $crate::style::Color::hsl($h, $s as f32/100.0, $l as f32/100.0).unwrap()
        };
        (hsl $h: literal $s: literal% $l: literal%) => {
            $crate::style::Color::hsl($h, $s as f32/100.0, $l as f32/100.0).unwrap()
        };
        (hsv $h: literal, $s: literal%, $v: literal%) => {
            $crate::style::Color::hsv($h, $s as f32/100.0, $v as f32/100.0).unwrap()
        };
        (hsv $h: literal $s: literal% $v: literal%) => {
            $crate::style::Color::hsv($h, $s as f32/100.0, $v as f32/100.0).unwrap()
        };
        ($c: literal%, $y: literal%, $m: literal%, $k: literal%) => {
            $crate::style::Color::cymk($c as f32/100.0, $y as f32/100.0, $m as f32/100.0, $k as f32/100.0).unwrap()
        };
        ($c: literal% $y: literal% $m: literal% $k: literal%) => {
            $crate::style::Color::cymk($c as f32/100.0, $y as f32/100.0, $m as f32/100.0, $k as f32/100.0).unwrap()
        };
        ($r: literal, $g: literal, $b: literal) => {
            $crate::style::Color::RGB { r: $r, g: $g, b: $b }
        };
        ($r: literal $g: literal $b: literal) => {
            $crate::style::Color::RGB { r: $r, g: $g, b: $b }
        };
        ($ansi: literal) => {
            $crate::style::Color::from($ansi)
        };
        (#$hex: literal) => {
            $crate::style::Color::from($($hex)*)
        };
        (#$($hex: tt)*) => {
            $crate::style::Color::from(stringify!($($hex)*))
        };
        ($color: ident) => {
            paste::paste!($crate::style::Color::[<$color:upper>])
        };
    }

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct _Placeholder;

impl Display for _Placeholder {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

#[derive(PartialEq, Default, Hash, Clone, Debug)]
pub struct Style {
    pub flags: StyleFlag,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub link: Option<Hyperlink>,
}

impl Style {
    /// Get a hash value for the given style. The hash value is useful for a hash key for both
    /// a map and a reference.
    pub fn hash_key(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub fn builder() -> Style {
        Style::default()
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn link<L: Display>(self, link: L) -> Style {
        Style {
            flags: self.flags,
            fg: self.fg,
            bg: self.bg,
            link: Some(Hyperlink::from(link.to_string())),
        }
    }

    pub fn flags(mut self, flags: StyleFlag) -> Self {
        self.flags |= flags;
        self
    }

    pub fn bold(mut self) -> Self {
        self.flags |= BOLD;
        self
    }

    pub fn italic(mut self) -> Self {
        self.flags |= ITALIC;
        self
    }

    pub fn underline(mut self) -> Self {
        self.flags |= UNDERLINE;
        self
    }

    pub fn crossed(mut self) -> Self {
        self.flags |= CROSSED;
        self
    }

    pub fn blink(mut self) -> Self {
        self.flags |= BLINK;
        self
    }

    pub fn reversed(mut self) -> Self {
        self.flags |= REVERSED;
        self
    }

    pub fn reset(mut self) -> Self {
        self.flags |= RESET;
        self
    }
}

impl Display for Style {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.sign_minus() {
            write!(f, "{}", self.reset_sequence())
        } else {
            write!(f, "{}", self.sequence())
        }
    }
}

impl AnsiSequence for Style {
    fn ansi(&self) -> String {
        let mut ansi = Vec::new();

        if self.flags.0 > 0 {
            ansi.push(self.flags.ansi());
        }

        if let Some(fg) = self.fg {
            ansi.push(fg.fg());
        }

        if let Some(bg) = self.bg {
            ansi.push(bg.bg());
        }

        if ansi.len() > 0 {
            ansi.join(";")
        } else {
            String::new()
        }
    }

    fn reset_ansi(&self) -> String {
        if self.flags & RESET == RESET {
            return "\x1b[0m".to_string();
        }

        let mut ansi = Vec::new();
        if let Some(fg) = self.fg {
            ansi.push(fg.reset_fg());
        }
        if let Some(bg) = self.bg {
            ansi.push(bg.reset_bg());
        }
        if self.flags.0 > 0 {
            ansi.push(self.flags.reset_ansi());
        }

        if ansi.len() > 0 {
            ansi.join(";")
        } else {
            String::new()
        }
    }

    fn sequence(&self) -> String {
        let link = match &self.link {
            Some(link) => {
                link.sequence()
            }
            None => String::new()
        };
        if link.len() == 0 && self.ansi().len() == 0 {
            return String::new();
        }
        format!("{}\x1b[{}m", link, self.ansi())
    }

    fn reset_sequence(&self) -> String {
        let link = match &self.link {
            Some(link) => {
                link.reset_sequence()
            }
            None => String::new()
        };

        if link.len() == 0 && self.reset_ansi().len() == 0 {
            return String::new();
        }
        format!("\x1b[{}m{}", self.reset_ansi(), link)
    }
}
