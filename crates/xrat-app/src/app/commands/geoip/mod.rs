use std::path::{Path, PathBuf};

use crate::app::context::AppContext;
use crate::app::paths::mmdb;
use crate::cli::{GeoIpAction, GeoIpArgs};

mod backend;
use crate::app::services::geoip_assets as download;
use crate::app::services::geoip_assets::edition;
mod lookup;
mod path;
mod status;
mod update;

#[cfg(test)]
mod tests;

mod command;
pub use command::run;
use command::*;
