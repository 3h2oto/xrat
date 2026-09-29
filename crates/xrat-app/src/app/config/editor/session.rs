use super::prelude::*;
use super::types::*;
use super::values::*;

impl ConfigEditSession {
    pub(crate) fn open(path: &Path) -> Result<Self, String> {
        let original_contents = fs::read_to_string(path)
            .map_err(|error| format!("could not read {}: {error}", path.display()))?;
        let document = original_contents
            .parse::<DocumentMut>()
            .map_err(|error| format!("config is not valid TOML: {error}"))?;
        let config: AppConfig = toml::from_str(&original_contents)
            .map_err(|error| format!("config could not be loaded: {error}"))?;
        let defaults = operational_values(&AppConfig::default())?;
        let current = operational_values(&config)?;
        let mut settings = Vec::new();
        flatten_settings("", &current, &defaults, &document, &mut settings);
        settings
            .sort_by(|left, right| (&left.section, &left.path).cmp(&(&right.section, &right.path)));
        Ok(Self {
            path: path.to_path_buf(),
            original_contents,
            document,
            settings,
        })
    }

    pub(crate) fn is_dirty(&self) -> bool {
        self.settings.iter().any(EditableSetting::is_dirty)
    }

    pub(crate) fn path_display(&self) -> String {
        self.path.display().to_string()
    }

    pub(crate) fn sections(&self, query: &str) -> Vec<String> {
        let query = query.trim().to_ascii_lowercase();
        let mut sections = BTreeSet::new();
        for setting in &self.settings {
            if query.is_empty()
                || setting.path.to_ascii_lowercase().contains(&query)
                || setting.label.to_ascii_lowercase().contains(&query)
            {
                sections.insert(setting.section.clone());
            }
        }
        sections.into_iter().collect()
    }

    pub(crate) fn set_value(&mut self, path: &str, input: &str) -> Result<(), String> {
        let setting = self
            .settings
            .iter_mut()
            .find(|setting| setting.path == path)
            .ok_or_else(|| format!("setting {path:?} was not found"))?;
        setting.set_from_input(input)
    }

    pub(crate) fn save(&mut self) -> Result<ConfigSaveOutcome, String> {
        let current_contents = fs::read_to_string(&self.path)
            .map_err(|error| format!("could not re-read {}: {error}", self.path.display()))?;
        if current_contents != self.original_contents {
            return Err(
                "config changed on disk; close and reopen settings before saving".to_string(),
            );
        }
        if !self.is_dirty() {
            let config = toml::from_str(&current_contents)
                .map_err(|error| format!("current config is invalid: {error}"))?;
            return Ok(ConfigSaveOutcome {
                config,
                changed_paths: Vec::new(),
                effects: BTreeSet::new(),
            });
        }

        let mut document = self.document.clone();
        let mut changed_paths = Vec::new();
        let mut effects = BTreeSet::new();
        for setting in self.settings.iter().filter(|setting| setting.is_dirty()) {
            if setting.reset {
                remove_path(&mut document, &setting.path);
            } else {
                set_path(&mut document, &setting.path, setting_item(setting)?);
            }
            changed_paths.push(setting.path.clone());
            effects.insert(setting.effect);
        }

        let candidate = document.to_string();
        let config: AppConfig = toml::from_str(&candidate)
            .map_err(|error| format!("updated config is invalid: {error}"))?;
        let diagnostics = crate::app::commands::validate::validate_app_config(&config);
        if let Some(diagnostic) = diagnostics.first() {
            return Err(diagnostic.clone());
        }
        atomic_write(&self.path, candidate.as_bytes())?;

        self.original_contents = candidate;
        self.document = document;
        for setting in &mut self.settings {
            setting.original_value = setting.value.clone();
            setting.explicit = path_exists(&self.document, &setting.path);
            setting.reset = false;
        }

        Ok(ConfigSaveOutcome {
            config,
            changed_paths,
            effects,
        })
    }
}

pub(crate) fn update_runtime_binary_path(
    config_path: &Path,
    key: &str,
    binary_path: &Path,
) -> Result<(), String> {
    let value = binary_path
        .to_str()
        .ok_or_else(|| "managed binary path is not valid UTF-8".to_string())?;
    let contents = fs::read_to_string(config_path)
        .map_err(|error| format!("could not read {}: {error}", config_path.display()))?;
    let mut document = contents
        .parse::<DocumentMut>()
        .map_err(|error| format!("config is not valid TOML: {error}"))?;
    set_path(&mut document, &format!("paths.{key}"), toml_value(value));
    let candidate = document.to_string();
    let config: AppConfig = toml::from_str(&candidate)
        .map_err(|error| format!("updated config is invalid: {error}"))?;
    if let Some(diagnostic) = crate::app::commands::validate::validate_app_config(&config).first() {
        return Err(diagnostic.clone());
    }
    atomic_write(config_path, candidate.as_bytes())
}
