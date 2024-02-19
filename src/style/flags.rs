use std::fmt::Display;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

use crate::style::AnsiSequence;

/// Flags representing the style of the text.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StyleFlag(pub u32);

impl AnsiSequence for StyleFlag {
    fn ansi(&self) -> String {
        let mut ansi = Vec::new();
        if *self & BOLD == BOLD {
            ansi.push("1");
        }
        if *self & ITALIC == ITALIC {
            ansi.push("3");
        }
        if *self & UNDERLINE == UNDERLINE {
            ansi.push("4");
        }
        if *self & CROSSED == CROSSED {
            ansi.push("9");
        }
        if *self & BLINK == BLINK {
            ansi.push("5");
        }
        if *self & REVERSED == REVERSED {
            ansi.push("7");
        }
        ansi.join(";")
    }

    fn reset_ansi(&self) -> String {
        let mut ansi = Vec::new();
        if *self & RESET == RESET {
            return "0".to_string();
        }

        if *self & BOLD == BOLD {
            ansi.push("22");
        }
        if *self & ITALIC == ITALIC {
            ansi.push("23");
        }
        if *self & UNDERLINE == UNDERLINE {
            ansi.push("24");
        }
        if *self & CROSSED == CROSSED {
            ansi.push("29");
        }
        if *self & BLINK == BLINK {
            ansi.push("25");
        }
        if *self & REVERSED == REVERSED {
            ansi.push("27");
        }
        ansi.join(";")
    }
}

impl BitOr for StyleFlag {
    type Output = StyleFlag;
    fn bitor(self, rhs: Self) -> Self::Output {
        StyleFlag(self.0 | rhs.0)
    }
}

impl BitAnd for StyleFlag {
    type Output = StyleFlag;
    fn bitand(self, rhs: Self) -> Self::Output {
        StyleFlag(self.0 & rhs.0)
    }
}

impl BitAndAssign for StyleFlag {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl BitOrAssign for StyleFlag {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

/// `\x1b[1m` - `\x1b[22m`
pub const BOLD: StyleFlag = StyleFlag(1u32);

/// `\x1b[3m` - `\x1b[23m`
pub const ITALIC: StyleFlag = StyleFlag(2u32);

/// `\x1b[4m` - `\x1b[24m`
pub const UNDERLINE: StyleFlag = StyleFlag(4u32);

/// `\x1b[9m` - `\x1b[29m`
pub const CROSSED: StyleFlag = StyleFlag(8u32);

/// `\x1b[5m` - `\x1b[25m`
pub const BLINK: StyleFlag = StyleFlag(16u32);

/// `\x1b[7m` - `\x1b[27m`
pub const REVERSED: StyleFlag = StyleFlag(32u32);

/// *ONLY* added the reset sequence.
/// `\x1b[0m`
pub const RESET: StyleFlag = StyleFlag(64u32);

impl Display for StyleFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.sign_minus() {
            write!(f, "{}", self.reset_sequence())
        } else {
            write!(f, "{}", self.sequence())
        }
    }
}