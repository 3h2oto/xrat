use super::colors::ThemeColor;
use crate::tui::theme::{ThemePalette, TuiTheme};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct CustomTheme {
    pub extends: TuiTheme,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<ThemeColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreground: Option<ThemeColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent: Option<ThemeColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary: Option<ThemeColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muted: Option<ThemeColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success: Option<ThemeColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<ThemeColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<ThemeColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download: Option<ThemeColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upload: Option<ThemeColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection_background: Option<ThemeColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection_foreground: Option<ThemeColor>,
}

impl CustomTheme {
    pub const COLOR_FIELDS: &'static [&'static str] = &[
        "background",
        "foreground",
        "accent",
        "secondary",
        "muted",
        "success",
        "failure",
        "warning",
        "download",
        "upload",
        "selection_background",
        "selection_foreground",
    ];
    pub fn palette(&self) -> ThemePalette {
        let mut palette = self.extends.palette();
        if let Some(color) = self.background {
            palette.background = color.0;
        }
        if let Some(color) = self.foreground {
            palette.foreground = color.0;
        }
        if let Some(color) = self.accent {
            palette.accent = color.0;
        }
        if let Some(color) = self.secondary {
            palette.secondary = color.0;
        }
        if let Some(color) = self.muted {
            palette.muted = color.0;
        }
        if let Some(color) = self.success {
            palette.success = color.0;
        }
        if let Some(color) = self.failure {
            palette.failure = color.0;
        }
        if let Some(color) = self.warning {
            palette.warning = color.0;
        }
        if let Some(color) = self.download {
            palette.download = color.0;
        }
        if let Some(color) = self.upload {
            palette.upload = color.0;
        }
        if let Some(color) = self.selection_background {
            palette.selection_background = color.0;
        }
        if let Some(color) = self.selection_foreground {
            palette.selection_foreground = color.0;
        }
        palette
    }
}
