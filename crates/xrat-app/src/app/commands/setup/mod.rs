//! `xrat setup` — post-install orchestration. Runs each setup step idempotently
//! (init, dependency checks, daemon, linger, completions, man pages, desktop,
//! xratui shortcut, PATH) and supports a read-only `--check` diagnostic mode.

mod cores;
mod desktop;
mod report;
mod steps;

use crate::app::commands::output;
use crate::app::context::AppContext;
use crate::app::events;
use crate::cli::{InstallArgs, InstallCore, SetupArgs, SetupFormat};
use xrat_support::platform;

use report::{StepOutcome, StepStatus};

#[cfg(test)]
mod tests;

mod installer;
pub use installer::install;
pub use installer::run;
#[cfg(test)]
use installer::unattended_dependency_change;
