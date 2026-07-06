//! Color themes for the TUI.
//!
//! Three built-in themes: `dark`, `light`, `solarized`.
//! All colors live in one place so the entire UI swaps consistently.

#![allow(dead_code)]

use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeName {
    Dark,
    Light,
    Solarized,
}

impl ThemeName {
    pub fn next(self) -> Self {
        match self {
            ThemeName::Dark => ThemeName::Light,
            ThemeName::Light => ThemeName::Solarized,
            ThemeName::Solarized => ThemeName::Dark,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ThemeName::Dark => "dark",
            ThemeName::Light => "light",
            ThemeName::Solarized => "solarized",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub dim: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub user: Color,
    pub assistant: Color,
    pub tool: Color,
    pub border: Color,
    pub selection: Color,
    pub highlight: Color,
}

impl Theme {
    pub const fn dark() -> Self {
        Self {
            bg: Color::Reset,
            fg: Color::White,
            dim: Color::DarkGray,
            accent: Color::Cyan,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            user: Color::Blue,
            assistant: Color::Magenta,
            tool: Color::Yellow,
            border: Color::DarkGray,
            selection: Color::DarkGray,
            highlight: Color::LightCyan,
        }
    }

    pub const fn light() -> Self {
        Self {
            bg: Color::Reset,
            fg: Color::Black,
            dim: Color::Gray,
            accent: Color::Blue,
            success: Color::Green,
            warning: Color::Rgb(180, 130, 0),
            error: Color::Red,
            user: Color::Blue,
            assistant: Color::Magenta,
            tool: Color::Rgb(160, 90, 0),
            border: Color::Gray,
            selection: Color::LightBlue,
            highlight: Color::Cyan,
        }
    }

    pub const fn solarized() -> Self {
        Self {
            bg: Color::Reset,
            fg: Color::Rgb(147, 161, 161),
            dim: Color::Rgb(88, 110, 117),
            accent: Color::Rgb(38, 139, 210),
            success: Color::Rgb(133, 153, 0),
            warning: Color::Rgb(181, 137, 0),
            error: Color::Rgb(220, 50, 47),
            user: Color::Rgb(38, 139, 210),
            assistant: Color::Rgb(108, 113, 196),
            tool: Color::Rgb(203, 75, 22),
            border: Color::Rgb(88, 110, 117),
            selection: Color::Rgb(7, 54, 66),
            highlight: Color::Rgb(42, 161, 152),
        }
    }

    pub fn by_name(name: ThemeName) -> Self {
        match name {
            ThemeName::Dark => Self::dark(),
            ThemeName::Light => Self::light(),
            ThemeName::Solarized => Self::solarized(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}
