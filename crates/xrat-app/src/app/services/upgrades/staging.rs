use crate::app::Result;
use std::{fs::File, path::Path};
pub(super) struct StagedFile {
    file: File,
    path: tempfile::TempPath,
    #[cfg(windows)]
    _directory: Option<xrat_support::windows::ProtectedDirectory>,
}
impl StagedFile {
    pub fn new(directory: &Path) -> Result<Self> {
        #[cfg(windows)]
        if super::windows::coordination::is_service_path(directory)? {
            let guard = super::windows::coordination::open_directory(directory)?;
            guard.verify_tree()?;
            let name = format!(".upgrade-stage-{}", uuid::Uuid::new_v4());
            let file = guard.create_file(&name)?;
            let path = tempfile::TempPath::try_from_path(directory.join(name))?;
            return Ok(Self {
                file,
                path,
                _directory: Some(guard),
            });
        }
        let (file, path) = tempfile::NamedTempFile::new_in(directory)?.into_parts();
        Ok(Self {
            file,
            path,
            #[cfg(windows)]
            _directory: None,
        })
    }
    pub fn as_file_mut(&mut self) -> &mut File {
        &mut self.file
    }
    pub fn as_file(&self) -> &File {
        &self.file
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn persist(self, target: impl AsRef<Path>) -> Result<()> {
        let Self {
            file,
            path,
            #[cfg(windows)]
            _directory,
        } = self;
        drop(file);
        path.persist(target)
            .map_err(|error| crate::app::AppError::Io(error.error))?;
        Ok(())
    }
}
