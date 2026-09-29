mod install;
mod prelude;
mod release;

#[cfg(test)]
mod tests;

pub(super) use install::{install, probe_all};
#[cfg(test)]
pub(super) use prelude::CORE_KINDS;
pub(super) use prelude::{CoreKind, CoreProbe, CoreRelease};
pub(super) use release::fetch_release;
