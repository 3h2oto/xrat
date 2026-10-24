use crate::app::context::{AppContext, RuntimePaths};
use crate::app::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::PathBuf;
use xrat_support::windows::ProtectedDirectory;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Registration {
    version: u16,
    operator_sid: String,
}

pub struct MachineProfile {
    root: ProtectedDirectory,
    _directories: Vec<ProtectedDirectory>,
    operator_sid: String,
}

impl MachineProfile {
    pub fn provision(operator_sid: &str) -> Result<Self> {
        super::validate_operator(operator_sid)?;
        Self::provision_at(
            xrat_support::windows::program_data()?.join("XRAT-Service"),
            operator_sid,
        )
    }

    fn provision_at(path: PathBuf, operator_sid: &str) -> Result<Self> {
        super::validate_operator(operator_sid)?;
        let root = ProtectedDirectory::create(&path)?;
        let directories = ["runtime", "cores", "logs", "mmdb"]
            .iter()
            .map(|name| ProtectedDirectory::create(&root.path().join(name)))
            .collect::<std::io::Result<Vec<_>>>()?;
        let registration = Registration {
            version: 1,
            operator_sid: operator_sid.to_owned(),
        };
        let metadata = root.path().join("operator.json");
        root.verify_tree()?;
        match root.create_file("operator.json") {
            Ok(mut file) => {
                file.write_all(&serde_json::to_vec(&registration)?)?;
                file.sync_all()?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let existing = read_registration(&metadata)?;
                if existing.operator_sid != operator_sid {
                    return Err(AppError::InvalidArgument(
                        "machine profile belongs to a different installing user".into(),
                    ));
                }
            }
            Err(error) => return Err(error.into()),
        }
        match root.create_file("config.toml") {
            Ok(mut file) => {
                file.write_all(b"# Administrator-managed XRAT service configuration\n")?;
                file.sync_all()?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
        Ok(Self {
            root,
            _directories: directories,
            operator_sid: operator_sid.to_owned(),
        })
    }

    pub fn open() -> Result<Self> {
        let root =
            ProtectedDirectory::open(&xrat_support::windows::program_data()?.join("XRAT-Service"))?;
        let directories = ["runtime", "cores", "logs", "mmdb"]
            .iter()
            .map(|name| ProtectedDirectory::open(&root.path().join(name)))
            .collect::<std::io::Result<Vec<_>>>()?;
        root.verify_tree()?;
        let registration = read_registration(&root.path().join("operator.json"))?;
        Ok(Self {
            root,
            _directories: directories,
            operator_sid: registration.operator_sid,
        })
    }

    pub fn operator_sid(&self) -> &str {
        &self.operator_sid
    }
    pub fn config_path(&self) -> PathBuf {
        self.root.path().join("config.toml")
    }

    pub fn lifecycle_lease(&self) -> Result<std::fs::File> {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::{FILE_SHARE_READ, FILE_SHARE_WRITE};
        let path = self.root.path().join("lifecycle.lock");
        if !path.try_exists()? {
            match self.root.create_file("lifecycle.lock") {
                Ok(file) => drop(file),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error.into()),
            }
        }
        self.root.verify_tree()?;
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .open(path)?;
        file.try_lock().map_err(std::io::Error::from).map_err(|error| AppError::InvalidArgument(
            format!("machine profile is active or undergoing maintenance: {error}; stop the machine service before provisioning")
        ))?;
        Ok(file)
    }

    pub fn core_directory(&self) -> Result<ProtectedDirectory> {
        Ok(ProtectedDirectory::open(&self.root.path().join("cores"))?)
    }

    pub async fn context(&self) -> Result<AppContext> {
        if xrat_support::windows::current_user_sid()? != super::SYSTEM_SID {
            return Err(AppError::InvalidArgument(
                "machine profile runtime requires LocalSystem".into(),
            ));
        }
        self.root.verify_tree()?;
        let root = self.root.path();
        let database_path = root.join("db.sqlite");
        let database_config = xrat_db::DatabaseConnectionConfig::Sqlite {
            path: database_path.clone(),
        };
        let paths = RuntimePaths {
            root_dir: root.to_path_buf(),
            database_path,
            database_label: database_config.label(),
            database_config,
            config_path: self.config_path(),
            runtime_dir: root.join("runtime"),
            xray_path: root.join("cores/xray.exe"),
            v2ray_path: root.join("cores/v2ray.exe"),
            sing_box_path: root.join("cores/sing-box.exe"),
        };
        let config = crate::app::config::load(&self.config_path())?;
        super::lifecycle::validate_config(&config)?;
        AppContext::from_parts(paths, config).await
    }
}

fn read_registration(path: &std::path::Path) -> Result<Registration> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(4097)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 4096 {
        return Err(AppError::InvalidArgument(
            "machine registration is too large".into(),
        ));
    }
    let registration: Registration = serde_json::from_slice(&bytes)?;
    super::validate_operator(&registration.operator_sid)?;
    if registration.version != 1 {
        return Err(AppError::InvalidArgument(
            "unsupported machine profile version".into(),
        ));
    }
    Ok(registration)
}

#[cfg(test)]
mod tests;
