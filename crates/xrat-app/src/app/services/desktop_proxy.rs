use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::app::{AppError, Result};

#[cfg(windows)]
#[path = "desktop_proxy/windows.rs"]
pub(crate) mod windows;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Settings {
    pub flags: u32,
    pub server: Option<String>,
    pub bypass: Option<String>,
    pub pac: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Ownership {
    before: Settings,
    applied: Settings,
}

pub(crate) trait Backend {
    fn read(&self) -> Result<Settings>;
    fn write(&self, settings: &Settings) -> Result<()>;
}

pub(crate) struct DesktopProxy<'a> {
    backend: &'a dyn Backend,
    path: PathBuf,
    _lock: std::fs::File,
}

impl<'a> DesktopProxy<'a> {
    pub fn open(backend: &'a dyn Backend, directory: &Path) -> Result<Self> {
        std::fs::create_dir_all(directory)?;
        let lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(directory.join("desktop-proxy.lock"))?;
        lock.try_lock().map_err(|error| {
            AppError::InvalidArgument(format!(
                "another desktop proxy operation is active: {error}"
            ))
        })?;
        Ok(Self {
            backend,
            path: directory.join("desktop-proxy.json"),
            _lock: lock,
        })
    }

    fn ownership(&self) -> Result<Option<Ownership>> {
        match std::fs::read(&self.path) {
            Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn status(&self) -> Result<&'static str> {
        let current = self.backend.read()?;
        if self
            .ownership()?
            .is_some_and(|owned| owned.applied == current)
        {
            Ok("XRAT-managed")
        } else if current.flags & (2 | 4 | 8) == 0 {
            Ok("disabled")
        } else {
            Ok("unrelated proxy settings")
        }
    }

    pub fn enable(&self, settings: Settings) -> Result<()> {
        let current = self.backend.read()?;
        let previous = self.ownership()?;
        if previous
            .as_ref()
            .is_some_and(|owned| owned.applied != current)
        {
            return Err(AppError::InvalidArgument("desktop proxy settings changed outside XRAT; refusing to overwrite them; restore or resolve the existing ownership record first".into()));
        }
        let before = previous
            .as_ref()
            .map_or_else(|| current.clone(), |owned| owned.before.clone());
        let ownership = Ownership {
            before,
            applied: settings.clone(),
        };
        let parent = self.path.parent().ok_or_else(|| {
            AppError::InvalidArgument("desktop ownership path has no parent".into())
        })?;
        let mut staged = tempfile::NamedTempFile::new_in(parent)?;
        use std::io::Write;
        staged.write_all(&serde_json::to_vec(&ownership)?)?;
        staged.as_file().sync_all()?;
        staged
            .persist(&self.path)
            .map_err(|error| AppError::Io(error.error))?;
        if let Err(error) = self.backend.write(&settings) {
            return match self.backend.write(&current) {
                Ok(()) => {
                    if let Some(previous) = previous {
                        std::fs::write(&self.path, serde_json::to_vec(&previous)?)?;
                    } else {
                        std::fs::remove_file(&self.path)?;
                    }
                    Err(error)
                }
                Err(rollback) => Err(AppError::InvalidArgument(format!(
                    "{error}; desktop proxy rollback failed: {rollback}; ownership retained at {}",
                    self.path.display()
                ))),
            };
        }
        Ok(())
    }

    pub fn disable(&self) -> Result<()> {
        let Some(owned) = self.ownership()? else {
            return Ok(());
        };
        let current = self.backend.read()?;
        if current == owned.before {
            std::fs::remove_file(&self.path)?;
            return Ok(());
        }
        if current != owned.applied {
            return Err(AppError::InvalidArgument("desktop proxy settings changed outside XRAT; current settings and recovery record were preserved".into()));
        }
        self.backend.write(&owned.before)?;
        std::fs::remove_file(&self.path)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "desktop_proxy/tests.rs"]
mod tests;
