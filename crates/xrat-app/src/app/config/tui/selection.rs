use crate::tui::theme::TuiTheme;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThemeName {
    Builtin(TuiTheme),
    Custom(String),
}

impl Default for ThemeName {
    fn default() -> Self {
        Self::Builtin(TuiTheme::Default)
    }
}

impl From<TuiTheme> for ThemeName {
    fn from(theme: TuiTheme) -> Self {
        Self::Builtin(theme)
    }
}

impl ThemeName {
    pub fn parse(name: &str) -> Result<Self, String> {
        if let Ok(theme) = name.parse::<TuiTheme>() {
            return Ok(Self::Builtin(theme));
        }
        if valid_custom_name(name) {
            return Ok(Self::Custom(name.to_string()));
        }
        Err(format!("tui.theme: invalid theme name {name:?}"))
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Builtin(theme) => theme.as_str(),
            Self::Custom(name) => name,
        }
    }
}

pub(super) fn valid_custom_name(name: &str) -> bool {
    name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

impl Serialize for ThemeName {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ThemeName {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::parse(&String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
