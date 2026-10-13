use std::str::FromStr;
use std::sync::atomic::{AtomicU8, Ordering};

use ratatui::style::{Color, Style};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Blank columns kept between a right-aligned action group and the edge.
pub const EDGE_MARGIN: usize = 2;

/// Shared label-column width for the detail/runtime panels so their
/// `Label value` columns line up to the same offset.
pub const DETAIL_LABEL_WIDTH: usize = 13;

/// Right padding for detail/runtime panels. Gives wrapped lines slack so
/// emoji-heavy config names (whose rendered width exceeds `unicode-width`'s
/// estimate) wrap before they collide with the right border.
pub const DETAIL_RIGHT_PAD: u16 = 3;

static ACTIVE_THEME: AtomicU8 = AtomicU8::new(TuiTheme::Default as u8);

#[cfg(test)]
pub static TEST_THEME_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Built-in color themes supported by the TUI.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum TuiTheme {
    #[default]
    Default = 0,
    Ansi = 1,
    Catppuccin = 2,
    Dracula = 3,
    Gruvbox = 4,
    Nord = 5,
    TokyoNight = 6,
    Light = 7,
}

impl TuiTheme {
    pub const CANONICAL_NAMES: &'static [&'static str] = &[
        "default",
        "ansi",
        "catppuccin",
        "dracula",
        "gruvbox",
        "nord",
        "tokyo-night",
        "light",
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Ansi => "ansi",
            Self::Catppuccin => "catppuccin",
            Self::Dracula => "dracula",
            Self::Gruvbox => "gruvbox",
            Self::Nord => "nord",
            Self::TokyoNight => "tokyo-night",
            Self::Light => "light",
        }
    }

    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Ansi,
            2 => Self::Catppuccin,
            3 => Self::Dracula,
            4 => Self::Gruvbox,
            5 => Self::Nord,
            6 => Self::TokyoNight,
            7 => Self::Light,
            _ => Self::Default,
        }
    }

    pub fn palette(self) -> ThemePalette {
        match self {
            Self::Default => ThemePalette {
                chrome: Color::Rgb(237, 225, 205),
                accent: Color::Rgb(247, 181, 99),
                secondary: Color::Rgb(130, 195, 225),
                muted: Color::Rgb(148, 139, 123),
                success: Color::Rgb(97, 198, 128),
                failure: Color::Rgb(229, 96, 84),
                warning: Color::Rgb(204, 170, 80),
                download: Color::Rgb(120, 180, 235),
                upload: Color::Rgb(210, 130, 220),
                selection_bg: Color::Rgb(58, 48, 38),
            },
            Self::Ansi => ThemePalette {
                chrome: Color::Reset,
                accent: Color::Cyan,
                secondary: Color::Blue,
                muted: Color::DarkGray,
                success: Color::Green,
                failure: Color::Red,
                warning: Color::Yellow,
                download: Color::Blue,
                upload: Color::Magenta,
                selection_bg: Color::Indexed(8),
            },
            Self::Catppuccin => ThemePalette {
                chrome: Color::Rgb(205, 214, 244),
                accent: Color::Rgb(137, 180, 250),
                secondary: Color::Rgb(203, 166, 247),
                muted: Color::Rgb(108, 112, 134),
                success: Color::Rgb(166, 227, 161),
                failure: Color::Rgb(243, 139, 168),
                warning: Color::Rgb(249, 226, 175),
                download: Color::Rgb(116, 199, 236),
                upload: Color::Rgb(245, 194, 231),
                selection_bg: Color::Rgb(69, 71, 90),
            },
            Self::Dracula => ThemePalette {
                chrome: Color::Rgb(248, 248, 242),
                accent: Color::Rgb(189, 147, 249),
                secondary: Color::Rgb(139, 233, 253),
                muted: Color::Rgb(98, 114, 164),
                success: Color::Rgb(80, 250, 123),
                failure: Color::Rgb(255, 85, 85),
                warning: Color::Rgb(241, 250, 140),
                download: Color::Rgb(139, 233, 253),
                upload: Color::Rgb(255, 121, 198),
                selection_bg: Color::Rgb(68, 71, 90),
            },
            Self::Gruvbox => ThemePalette {
                chrome: Color::Rgb(235, 219, 178),
                accent: Color::Rgb(254, 128, 25),
                secondary: Color::Rgb(142, 192, 124),
                muted: Color::Rgb(146, 131, 116),
                success: Color::Rgb(184, 187, 38),
                failure: Color::Rgb(251, 73, 52),
                warning: Color::Rgb(250, 189, 47),
                download: Color::Rgb(131, 165, 152),
                upload: Color::Rgb(211, 134, 155),
                selection_bg: Color::Rgb(60, 56, 54),
            },
            Self::Nord => ThemePalette {
                chrome: Color::Rgb(236, 239, 244),
                accent: Color::Rgb(136, 192, 208),
                secondary: Color::Rgb(129, 161, 193),
                muted: Color::Rgb(129, 141, 161),
                success: Color::Rgb(163, 190, 140),
                failure: Color::Rgb(191, 97, 106),
                warning: Color::Rgb(235, 203, 139),
                download: Color::Rgb(143, 188, 187),
                upload: Color::Rgb(180, 142, 173),
                selection_bg: Color::Rgb(59, 66, 82),
            },
            Self::TokyoNight => ThemePalette {
                chrome: Color::Rgb(192, 202, 245),
                accent: Color::Rgb(122, 162, 247),
                secondary: Color::Rgb(187, 154, 247),
                muted: Color::Rgb(86, 95, 137),
                success: Color::Rgb(158, 206, 106),
                failure: Color::Rgb(247, 118, 142),
                warning: Color::Rgb(224, 175, 104),
                download: Color::Rgb(125, 207, 255),
                upload: Color::Rgb(187, 154, 247),
                selection_bg: Color::Rgb(41, 46, 66),
            },
            Self::Light => ThemePalette {
                chrome: Color::Rgb(76, 79, 105),
                accent: Color::Rgb(30, 102, 245),
                secondary: Color::Rgb(136, 57, 239),
                muted: Color::Rgb(140, 143, 161),
                success: Color::Rgb(64, 160, 43),
                failure: Color::Rgb(210, 15, 57),
                warning: Color::Rgb(223, 142, 29),
                download: Color::Rgb(4, 165, 229),
                upload: Color::Rgb(136, 57, 239),
                selection_bg: Color::Rgb(220, 224, 232),
            },
        }
    }
}

impl FromStr for TuiTheme {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "default" | "amber" | "warm" => Ok(Self::Default),
            "ansi" | "terminal" => Ok(Self::Ansi),
            "catppuccin" | "mocha" | "catppuccin-mocha" => Ok(Self::Catppuccin),
            "dracula" => Ok(Self::Dracula),
            "gruvbox" | "gruvbox-dark" => Ok(Self::Gruvbox),
            "nord" => Ok(Self::Nord),
            "tokyo-night" | "tokyonight" => Ok(Self::TokyoNight),
            "light" | "latte" | "solarized-light" => Ok(Self::Light),
            other => Err(format!(
                "unknown theme \"{other}\"; accepted values: {}",
                Self::CANONICAL_NAMES.join(", ")
            )),
        }
    }
}

impl Serialize for TuiTheme {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for TuiTheme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        Self::from_str(&text).map_err(serde::de::Error::custom)
    }
}

/// Color set assigned to TUI UI components.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemePalette {
    pub chrome: Color,
    pub accent: Color,
    pub secondary: Color,
    pub muted: Color,
    pub success: Color,
    pub failure: Color,
    pub warning: Color,
    pub download: Color,
    pub upload: Color,
    pub selection_bg: Color,
}

pub fn active() -> TuiTheme {
    TuiTheme::from_u8(ACTIVE_THEME.load(Ordering::Relaxed))
}

pub fn set_active(theme: TuiTheme) {
    ACTIVE_THEME.store(theme as u8, Ordering::Relaxed);
}

pub fn palette() -> ThemePalette {
    active().palette()
}

pub fn chrome_style() -> Style {
    Style::default().fg(palette().chrome)
}

pub fn accent_style() -> Style {
    Style::default().fg(palette().accent)
}

pub fn secondary_style() -> Style {
    Style::default().fg(palette().secondary)
}

pub fn muted_style() -> Style {
    Style::default().fg(palette().muted)
}

pub fn success_style() -> Style {
    Style::default().fg(palette().success)
}

pub fn failure_style() -> Style {
    Style::default().fg(palette().failure)
}

pub fn warning_style() -> Style {
    Style::default().fg(palette().warning)
}

pub fn selection_style() -> Style {
    Style::default().bg(palette().selection_bg)
}

/// Border style for cards and modals. Focused panels get the primary accent,
/// while unfocused panels stay subtle and muted so content remains readable.
pub fn border_style(focused: bool) -> Style {
    if focused {
        accent_style()
    } else {
        muted_style()
    }
}

/// Style for latency thresholds: green (<150ms), yellow (150..400ms), red (>400ms).
pub fn latency_style(ms: u64) -> Style {
    if ms < 150 {
        success_style()
    } else if ms < 400 {
        warning_style()
    } else {
        failure_style()
    }
}

/// Parse string like "12 ms" or "350ms" and color by latency threshold.
pub fn latency_text_style(text: &str) -> Style {
    let digits: String = text.chars().take_while(|c| c.is_ascii_digit()).collect();
    if let Ok(ms) = digits.parse::<u64>() {
        latency_style(ms)
    } else {
        muted_style()
    }
}

/// Automatic semantic style for detail panel values based on key name and content.
pub fn value_style(label: &str, value: &str) -> Style {
    let trimmed = value.trim();
    if trimmed == "-" || trimmed == "none" {
        return muted_style();
    }
    match label {
        "Enabled" | "Active" => {
            if trimmed == "yes" {
                success_style()
            } else {
                muted_style()
            }
        }
        "Deleted" => {
            if trimmed == "yes" {
                failure_style()
            } else {
                muted_style()
            }
        }
        "Daemon" => {
            if trimmed == "running" {
                success_style()
            } else {
                muted_style()
            }
        }
        "Rotation" => {
            if trimmed.starts_with("on") {
                success_style()
            } else {
                muted_style()
            }
        }
        "Protocol" => secondary_style().add_modifier(ratatui::style::Modifier::BOLD),
        "ICMP" | "TCP" | "Real Delay" => latency_text_style(trimmed),
        "Download" => series_style("download"),
        "Upload" => series_style("upload"),
        "Failure" => failure_style(),
        _ => {
            if trimmed == "yes" || trimmed.contains('✓') {
                success_style()
            } else if trimmed == "no" || trimmed.contains('✗') {
                failure_style()
            } else {
                chrome_style()
            }
        }
    }
}

/// Dynamic style for latency and throughput metrics in graphs and tables.
pub fn series_style(label: &str) -> Style {
    let p = palette();
    let color = match label {
        "icmp" => p.success,
        "tcp" => p.warning,
        "real-delay" => p.accent,
        "download" => p.download,
        "upload" => p.upload,
        _ => p.chrome,
    };
    Style::default().fg(color)
}

/// Shared severity color for app events and proxy engine logs. Maps
/// critical/fatal/panic/error to red, warn/warning to yellow, and
/// info/debug/trace (or any other recognized level) to secondary color. An
/// empty or unknown level falls back to the muted style.
pub fn severity_style(level: &str) -> Style {
    match level.trim().to_ascii_lowercase().as_str() {
        "critical" | "fatal" | "panic" | "error" | "err" => failure_style(),
        "warn" | "warning" => warning_style(),
        "info" | "debug" | "trace" | "notice" => secondary_style(),
        "" => muted_style(),
        _ => secondary_style(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_canonical_names_and_aliases() {
        assert_eq!("default".parse::<TuiTheme>().unwrap(), TuiTheme::Default);
        assert_eq!("amber".parse::<TuiTheme>().unwrap(), TuiTheme::Default);
        assert_eq!("warm".parse::<TuiTheme>().unwrap(), TuiTheme::Default);
        assert_eq!("ansi".parse::<TuiTheme>().unwrap(), TuiTheme::Ansi);
        assert_eq!("terminal".parse::<TuiTheme>().unwrap(), TuiTheme::Ansi);
        assert_eq!(
            "catppuccin".parse::<TuiTheme>().unwrap(),
            TuiTheme::Catppuccin
        );
        assert_eq!("mocha".parse::<TuiTheme>().unwrap(), TuiTheme::Catppuccin);
        assert_eq!("dracula".parse::<TuiTheme>().unwrap(), TuiTheme::Dracula);
        assert_eq!("gruvbox".parse::<TuiTheme>().unwrap(), TuiTheme::Gruvbox);
        assert_eq!("nord".parse::<TuiTheme>().unwrap(), TuiTheme::Nord);
        assert_eq!(
            "tokyo-night".parse::<TuiTheme>().unwrap(),
            TuiTheme::TokyoNight
        );
        assert_eq!(
            "tokyonight".parse::<TuiTheme>().unwrap(),
            TuiTheme::TokyoNight
        );
        assert_eq!("light".parse::<TuiTheme>().unwrap(), TuiTheme::Light);
        assert_eq!("latte".parse::<TuiTheme>().unwrap(), TuiTheme::Light);
    }

    #[test]
    fn rejects_unknown_theme() {
        assert!("invalid-theme".parse::<TuiTheme>().is_err());
    }

    #[test]
    fn switching_theme_updates_active_palette_and_styles() {
        let _guard = TEST_THEME_LOCK.lock().unwrap();
        set_active(TuiTheme::Default);
        assert_eq!(active(), TuiTheme::Default);
        let default_accent = accent_style();
        let default_secondary = secondary_style();

        set_active(TuiTheme::Nord);
        assert_eq!(active(), TuiTheme::Nord);
        let nord_accent = accent_style();
        let nord_secondary = secondary_style();
        assert_ne!(default_accent, nord_accent);
        assert_ne!(default_secondary, nord_secondary);

        set_active(TuiTheme::Default);
    }

    #[test]
    fn latency_style_thresholds() {
        let _guard = TEST_THEME_LOCK.lock().unwrap();
        set_active(TuiTheme::Default);
        assert_eq!(latency_style(50), success_style());
        assert_eq!(latency_style(250), warning_style());
        assert_eq!(latency_style(600), failure_style());

        assert_eq!(latency_text_style("42 ms"), success_style());
        assert_eq!(latency_text_style("200ms"), warning_style());
        assert_eq!(latency_text_style("500 ms"), failure_style());
        assert_eq!(latency_text_style("-"), muted_style());
    }

    #[test]
    fn semantic_value_style() {
        let _guard = TEST_THEME_LOCK.lock().unwrap();
        set_active(TuiTheme::Default);
        assert_eq!(value_style("Enabled", "yes"), success_style());
        assert_eq!(value_style("Enabled", "no"), muted_style());
        assert_eq!(value_style("Deleted", "yes"), failure_style());
        assert_eq!(value_style("Daemon", "running"), success_style());
        assert_eq!(value_style("Daemon", "stopped"), muted_style());
        assert_eq!(value_style("Failure", "timeout"), failure_style());
        assert_eq!(value_style("Any", "-"), muted_style());
    }
}
