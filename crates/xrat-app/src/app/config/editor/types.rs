use super::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SettingEffect {
    Live,
    RuntimeRestart,
    DaemonRestart,
}

impl SettingEffect {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::RuntimeRestart => "runtime restart",
            Self::DaemonRestart => "daemon restart",
        }
    }

    pub(crate) fn help_text(self) -> &'static str {
        match self {
            Self::Live => "Applies immediately to subsequent TUI operations.",
            Self::RuntimeRestart => "Requires restarting the active proxy runtime.",
            Self::DaemonRestart => "Requires restarting the xrat daemon.",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SettingKind {
    Bool,
    Integer,
    Text,
    List { numeric: bool },
    Enum(&'static [&'static str]),
    Secret,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SettingValue {
    Bool(bool),
    Integer(i64),
    Text(String),
    List(Vec<String>),
    Secret(String),
}

impl SettingValue {
    pub(crate) fn display(&self, secret: bool) -> String {
        if secret {
            return if matches!(self, Self::Secret(value) if value.is_empty()) {
                "not set".to_string()
            } else {
                "•••• configured".to_string()
            };
        }
        match self {
            Self::Bool(value) => value.to_string(),
            Self::Integer(value) => value.to_string(),
            Self::Text(value) => {
                if value.is_empty() {
                    "<empty>".to_string()
                } else {
                    value.clone()
                }
            }
            Self::List(values) => {
                if values.is_empty() {
                    "[]".to_string()
                } else {
                    values.join(", ")
                }
            }
            Self::Secret(_) => "•••• configured".to_string(),
        }
    }

    pub(crate) fn edit_text(&self) -> String {
        match self {
            Self::Bool(value) => value.to_string(),
            Self::Integer(value) => value.to_string(),
            Self::Text(value) | Self::Secret(value) => value.clone(),
            Self::List(values) => values.join(", "),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct EditableSetting {
    pub(crate) path: String,
    pub(crate) section: String,
    pub(crate) label: String,
    pub(crate) kind: SettingKind,
    pub(crate) effect: SettingEffect,
    pub(crate) help: SettingHelp,
    pub(crate) value: SettingValue,
    pub(crate) default_value: SettingValue,
    pub(crate) original_value: SettingValue,
    pub(crate) explicit: bool,
    pub(crate) reset: bool,
}

impl EditableSetting {
    pub(crate) fn is_dirty(&self) -> bool {
        self.value != self.original_value || (self.reset && self.explicit)
    }

    pub(crate) fn is_explicit(&self) -> bool {
        self.explicit
    }

    pub(crate) fn is_reset(&self) -> bool {
        self.reset
    }

    pub(crate) fn default_value(&self) -> &SettingValue {
        &self.default_value
    }

    pub(crate) fn possible_values(&self) -> String {
        match &self.kind {
            SettingKind::Bool => "✓ enabled · ✗ disabled".to_string(),
            SettingKind::Enum(options) => options.join(" · "),
            _ => self.help.value_hint.to_string(),
        }
    }

    pub(crate) fn toggle(&mut self) -> bool {
        let SettingValue::Bool(value) = &mut self.value else {
            return false;
        };
        *value = !*value;
        self.reset = false;
        true
    }

    pub(crate) fn cycle_enum(&mut self, direction: i32) -> bool {
        let SettingKind::Enum(options) = &self.kind else {
            return false;
        };
        let SettingValue::Text(current) = &mut self.value else {
            return false;
        };
        let position = options
            .iter()
            .position(|option| *option == current)
            .unwrap_or(0);
        let next = if direction < 0 {
            (position + options.len() - 1) % options.len()
        } else {
            (position + 1) % options.len()
        };
        *current = options[next].to_string();
        self.reset = false;
        true
    }

    pub(crate) fn set_from_input(&mut self, input: &str) -> Result<(), String> {
        self.value = match self.kind {
            SettingKind::Integer => SettingValue::Integer(
                input
                    .trim()
                    .parse()
                    .map_err(|_| "enter a whole number".to_string())?,
            ),
            SettingKind::Text => SettingValue::Text(input.trim().to_string()),
            SettingKind::List { numeric } => {
                let values: Vec<String> = input
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string)
                    .collect();
                if numeric && values.iter().any(|value| value.parse::<i64>().is_err()) {
                    return Err("list values must be whole numbers".to_string());
                }
                SettingValue::List(values)
            }
            SettingKind::Secret => SettingValue::Secret(input.trim().to_string()),
            SettingKind::Bool | SettingKind::Enum(_) => {
                return Err("use the toggle or cycle keys for this setting".to_string());
            }
        };
        self.reset = false;
        Ok(())
    }

    pub(crate) fn reset_to_default(&mut self) {
        self.value = self.default_value.clone();
        self.reset = true;
    }
}

#[derive(Debug)]
pub(crate) struct ConfigEditSession {
    pub(crate) path: PathBuf,
    pub(crate) original_contents: String,
    pub(crate) document: DocumentMut,
    pub(crate) settings: Vec<EditableSetting>,
}

#[derive(Debug)]
pub(crate) struct ConfigSaveOutcome {
    pub(crate) config: AppConfig,
    pub(crate) changed_paths: Vec<String>,
    pub(crate) effects: BTreeSet<SettingEffect>,
}
