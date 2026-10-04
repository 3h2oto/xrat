mod collect;

use std::time::Duration;

use crate::app::AppError;
use crate::app::commands::output::{self, Align, Cell, Column, Style};
use crate::app::commands::progress::CliProgress;
use crate::app::context::AppContext;
use crate::cli::{ListFormat, ScanArgs};
use xrat_db::{CfScanResultRecord, CfScanResultUpsert};
use xrat_prober::tcp_check;

mod runner;
pub use runner::run;
