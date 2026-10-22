mod install;
mod prelude;
mod release;

#[cfg(test)]
mod tests;

pub(crate) use install::{install, probe_all};
#[cfg(test)]
pub(crate) use prelude::CORE_KINDS;
pub(crate) use prelude::{CoreKind, CoreProbe, CoreRelease};
pub(crate) use release::fetch_release;

#[cfg(any(windows, test))]
pub(crate) use install::prepare_machine_binary;
#[cfg(windows)]
pub(crate) use install::{binary_version, download_archive};

#[cfg(windows)]
pub(crate) use install::binary_version_with_spawner;
