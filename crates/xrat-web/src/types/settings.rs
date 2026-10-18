use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Setting {
    pub path: String,
    pub label: String,
    pub value: String,
    pub kind: String,
    pub options: Vec<String>,
    pub effect: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub revision: String,
    pub items: Vec<Setting>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SettingUpdate {
    pub revision: String,
    pub path: String,
    pub value: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SettingChange {
    pub path: String,
    pub value: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SettingsBatchUpdate {
    pub revision: String,
    pub changes: Vec<SettingChange>,
}
