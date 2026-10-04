use std::path::Path;
use std::time::Duration;

use crate::probe::{ProbeEngineKind, ProbeProcess};
use xrat_engines::xray::XrayGenOptions;
use xrat_model::Node;

mod proxied;
pub use proxied::make_proxied_download_with_client;
mod result;

pub use result::DownloadResult;
#[cfg(test)]
pub(crate) use result::calculate_mbps;

mod runner;
pub use runner::download_speed_check;
