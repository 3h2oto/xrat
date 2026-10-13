use serde::{Deserialize, Serialize};

use crate::tui::theme::TuiTheme;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct TuiSettings {
    pub theme: TuiTheme,
}
