use std::collections::HashMap;
use std::env::VarError;
use std::ffi::{OsStr, OsString};

pub trait EnvVars: Send + Sync {
    fn get_os(&self, key: &OsStr) -> Option<OsString>;

    fn get(&self, key: &OsStr) -> Result<String, VarError> {
        self.get_os(key)
            .ok_or(VarError::NotPresent)?
            .into_string()
            .map_err(VarError::NotUnicode)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemEnvVars;
impl EnvVars for SystemEnvVars {
    fn get_os(&self, key: &OsStr) -> Option<OsString> {
        std::env::var_os(key)
    }
}

#[derive(Debug, Clone, Default)]
pub struct MapEnvVars(pub HashMap<OsString, OsString>);
impl EnvVars for MapEnvVars {
    fn get_os(&self, key: &OsStr) -> Option<OsString> {
        self.0.get(key).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distinguishes_empty_and_missing() {
        let env = MapEnvVars(HashMap::from([("EMPTY".into(), "".into())]));
        assert_eq!(env.get(OsStr::new("EMPTY")).unwrap(), "");
        assert!(matches!(
            env.get(OsStr::new("MISSING")),
            Err(VarError::NotPresent)
        ));
    }
    #[cfg(unix)]
    #[test]
    fn preserves_non_unicode_values() {
        use std::os::unix::ffi::OsStringExt;
        let value = OsString::from_vec(vec![255]);
        let env = MapEnvVars(HashMap::from([("PATH".into(), value.clone())]));
        assert_eq!(env.get_os(OsStr::new("PATH")), Some(value));
        assert!(matches!(
            env.get(OsStr::new("PATH")),
            Err(VarError::NotUnicode(_))
        ));
    }
}
