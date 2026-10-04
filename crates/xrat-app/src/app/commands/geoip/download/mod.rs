mod executor;
mod progress;
mod request;
mod summary;

#[cfg(test)]
mod tests;

use crate::app::AppError;
use crate::app::context::AppContext;
use crate::cli::GeoIpDownloadArgs;

use executor::execute_downloads;
use request::DownloadRequest;

mod transfer;
pub(crate) use transfer::run;
