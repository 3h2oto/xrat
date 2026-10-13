use ratatui::style::Color;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemeColor(pub Color);

impl ThemeColor {
    pub fn parse(value: &str) -> Result<Self, String> {
        if value.len() == 7
            && value.starts_with('#')
            && value[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            let rgb = u32::from_str_radix(&value[1..], 16).map_err(|error| error.to_string())?;
            return Ok(Self(Color::Rgb(
                (rgb >> 16) as u8,
                (rgb >> 8) as u8,
                rgb as u8,
            )));
        }
        let color = match value {
            "terminal" => Color::Reset,
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "gray" => Color::Gray,
            "dark-gray" => Color::DarkGray,
            "light-red" => Color::LightRed,
            "light-green" => Color::LightGreen,
            "light-yellow" => Color::LightYellow,
            "light-blue" => Color::LightBlue,
            "light-magenta" => Color::LightMagenta,
            "light-cyan" => Color::LightCyan,
            "white" => Color::White,
            _ => {
                return Err(format!(
                    "invalid color {value:?}; use #RRGGBB, an ANSI color name or terminal"
                ));
            }
        };
        Ok(Self(color))
    }
}

impl<'de> Deserialize<'de> for ThemeColor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::parse(&String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

impl Serialize for ThemeColor {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let value = match self.0 {
            Color::Rgb(red, green, blue) => format!("#{red:02x}{green:02x}{blue:02x}"),
            Color::Reset => "terminal".to_string(),
            Color::Black => "black".to_string(),
            Color::Red => "red".to_string(),
            Color::Green => "green".to_string(),
            Color::Yellow => "yellow".to_string(),
            Color::Blue => "blue".to_string(),
            Color::Magenta => "magenta".to_string(),
            Color::Cyan => "cyan".to_string(),
            Color::Gray => "gray".to_string(),
            Color::DarkGray => "dark-gray".to_string(),
            Color::LightRed => "light-red".to_string(),
            Color::LightGreen => "light-green".to_string(),
            Color::LightYellow => "light-yellow".to_string(),
            Color::LightBlue => "light-blue".to_string(),
            Color::LightMagenta => "light-magenta".to_string(),
            Color::LightCyan => "light-cyan".to_string(),
            Color::White => "white".to_string(),
            Color::Indexed(_) => {
                return Err(serde::ser::Error::custom(
                    "indexed colors cannot be serialized as theme colors",
                ));
            }
        };
        serializer.serialize_str(&value)
    }
}
