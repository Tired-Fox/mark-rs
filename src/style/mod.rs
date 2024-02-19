use std::fmt::{Debug, Display, Formatter};
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

pub struct Hyperlink<S>(pub S);

impl Default for Hyperlink<&'static str> {
    fn default() -> Self {
        Hyperlink("")
    }
}

impl<S: Debug> Debug for Hyperlink<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hyperlink")
            .field("link", &self.0)
            .finish()
    }
}

impl<S: Copy> Copy for Hyperlink<S> {}

impl<S: Clone> Clone for Hyperlink<S> {
    fn clone(&self) -> Self {
        Hyperlink(self.0.clone())
    }
}

impl<D: Display> Display for Hyperlink<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.sign_minus() {
            write!(f, "{}", self.reset_sequence())
        } else {
            write!(f, "{}", self.sequence())
        }
    }
}

impl<D> From<D> for Hyperlink<D> {
    fn from(d: D) -> Self {
        Hyperlink(d)
    }
}

impl<D: Display> AnsiSequence for Hyperlink<D> {
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

/// System color representation.
#[derive(Debug, Clone, Copy)]
pub enum SystemColor {
    BLACK,
    RED,
    GREEN,
    YELLOW,
    BLUE,
    MAGENTA,
    CYAN,
    WHITE,
}

impl AnsiSequence for SystemColor {
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
        }
    }

    fn reset_ansi(&self) -> String {
        "9".to_string()
    }
}

impl Display for SystemColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let prefix = if f.alternate() { "4" } else { "3" };
        if f.sign_minus() {
            write!(f, "\x1b[{prefix}{}m", self.reset_ansi())
        } else {
            write!(f, "\x1b[{prefix}{}m", self.ansi())
        }
    }
}

/// Terminal color representation.
///
/// Supports named system colors, XTerm/Ansi colors (0-255), and RGB colors (0-255,0-255,0-255).
#[derive(Debug, Clone, Copy)]
pub enum Color {
    System(SystemColor),
    /// 0<=value<=255
    Ansi(u8),
    /// 0<=R<=255, 0<=G<=255, 0<=B<=255
    RGB { r: u8, g: u8, b: u8 },
    /// 0<=H<360, 0<=S<=1, 0<=L<=1
    HSL { h: u16, s: f32, l: f32 },
    HSV { h: u16, s: f32, v: f32 },
    CYMK { c: f32, y: f32, m: f32, k: f32 },
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
            Color::System(sys) => sys.ansi(),
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
                let c = (1.0 - ((2.0 * l) - 1.0).abs()) * s;
                let h = *h as f32 / 60.0;
                let x = c * (1.0 - ((h % 2.0) - 1.0));
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
    let (r, g, b) = if h > 0.0 && h < 1.0 { (c, x, 0.0) }
    else if h >= 1.0 && h <= 2.0 { (x, c, 0.0) }
    else if h >= 2.0 && h <= 3.0 { (0.0, c, x) }
    else if h >= 3.0 && h <= 4.0 { (0.0, x, c) }
    else if h >= 4.0 && h <= 5.0 { (x, 0.0, c) }
    else if h >= 5.0 && h <= 6.0 { (c, 0.0, x) }
    else {
        panic!("Hue in hsl is greater than or equal to 360 which is out of bounds")
    };

    format!(
        "8;2;{};{};{}",
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8
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
            paste::paste!($crate::style::Color::System($crate::style::SystemColor::[<$color:upper>]))
        };
    }

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct _Placeholder;

impl Display for _Placeholder {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

pub struct Style<D = _Placeholder> {
    pub flags: StyleFlag,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub link: Option<Hyperlink<D>>,
}

impl<D: Display + Debug> Debug for Style<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Style")
            .field("flags", &self.flags)
            .field("fg", &self.fg)
            .field("bg", &self.bg)
            .field("link", &self.link)
            .finish()
    }
}

impl<D: Display + Copy> Copy for Style<D> {}

impl<D: Display + Clone> Clone for Style<D> {
    fn clone(&self) -> Self {
        Style {
            flags: self.flags,
            fg: self.fg,
            bg: self.bg,
            link: self.link.clone(),
        }
    }
}

impl Style<&'static str> {
    pub fn builder() -> Style<&'static str> {
        Style {
            flags: StyleFlag::default(),
            fg: None,
            bg: None,
            link: None::<Hyperlink<&'static str>>,
        }
    }
}

impl<D: Display> Style<D> {
    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn link<L: Display>(self, link: L) -> Style<L> {
        Style {
            flags: self.flags,
            fg: self.fg,
            bg: self.bg,
            link: Some(Hyperlink(link)),
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

impl<D: Display> Display for Style<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.sign_minus() {
            write!(f, "{}", self.reset_sequence())
        } else {
            write!(f, "{}", self.sequence())
        }
    }
}

impl<D: Display> AnsiSequence for Style<D> {
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
        format!("{}\x1b[{}m", link, self.ansi())
    }

    fn reset_sequence(&self) -> String {
        let link = match &self.link {
            Some(link) => {
                link.reset_sequence()
            }
            None => String::new()
        };
        format!("\x1b[{}m{}", self.reset_ansi(), link)
    }
}
