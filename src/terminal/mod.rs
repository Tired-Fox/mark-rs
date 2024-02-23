use std::io::{Read, Write};

use lazy_static::lazy_static;

pub mod buffer;
mod command;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum ColorSupport {
    None,
    #[default]
    Standard,
    EightBit,
    TrueColor,
}

impl ColorSupport {
    pub fn new() -> Self {
        let color_term_type = {
            let term_type = std::env::var("TERM")
                .map_or(
                    "dumb".to_string(),
                    |v| v.strip_prefix("xterm-").map_or("dump".to_string(), |v| v.to_string()),
                );
            std::env::var("COLORTERM")
                .unwrap_or(term_type)
        };
        match color_term_type.as_str() {
            "truecolor" | "24bit" => ColorSupport::TrueColor,
            "256color" => ColorSupport::EightBit,
            _ => ColorSupport::default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capabilities {
    pub ansi: bool,
    pub color: ColorSupport,
}

lazy_static! {
    pub static ref CAPABILITIES: Capabilities = Capabilities::default();
}

impl Default for Capabilities {
    fn default() -> Self {
        let term_type = std::env::var("TERM").unwrap_or("dumb".to_string());
        Capabilities {
            ansi: term_type.as_str() != "dumb",
            color: ColorSupport::new(),
        }
    }
}