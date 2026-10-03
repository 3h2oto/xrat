use std::ffi::{OsStr, OsString};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Output};
use std::sync::Arc;

use async_trait::async_trait;
mod system;
pub use system::SystemProcessSpawner;

#[cfg(all(test, unix))]
mod tests;

mod contracts;
pub use contracts::Child;
pub use contracts::ChildHandle;
pub use contracts::Command;
pub use contracts::CommandSpec;
pub use contracts::ProcessSpawner;
pub use contracts::StartupChild;
pub use contracts::Stdio;
