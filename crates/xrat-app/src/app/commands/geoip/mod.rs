use std::path::{Path, PathBuf};

use crate::app::context::AppContext;
use crate::app::paths::mmdb;
use crate::cli::{GeoIpAction, GeoIpArgs};

mod backend;
mod download;
mod edition;
mod lookup;
mod path;
mod status;
mod update;

#[cfg(test)]
mod tests;

mod command;
pub use command::run;
use command::*;
