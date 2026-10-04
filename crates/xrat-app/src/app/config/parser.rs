use serde::{Deserialize, Serialize};

use xrat_config::parsing::ParseMode;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct ParserSettings {
    pub parse_mode: ParseMode,
}

impl Default for ParserSettings {
    fn default() -> Self {
        Self {
            parse_mode: ParseMode::Strict,
        }
    }
}
