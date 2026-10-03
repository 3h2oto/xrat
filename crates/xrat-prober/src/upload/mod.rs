use std::path::Path;
use std::time::Duration;

use crate::probe::{ProbeEngineKind, ProbeProcess};
use xrat_engines::xray::XrayGenOptions;
use xrat_model::Node;

use super::FailureKind;

mod classify;
mod request;

pub use classify::classify_request_error;
use request::{find_available_port, make_proxied_upload};

mod runner;
pub use runner::UploadResult;
pub use runner::upload_speed_check;
