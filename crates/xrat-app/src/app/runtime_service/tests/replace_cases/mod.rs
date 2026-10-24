use super::super::*;
use super::test_support::{test_context, test_node, test_node_with, test_source};
use crate::app::daemon::ipc::RotationTrigger;
use crate::app::tests::platform::spawn_sleep;
use std::fs;
#[cfg(unix)]
use xrat_engines::xray::runtime_process as xray_runtime;

mod fake_runtime;
// The handoff tests start a real runtime through a POSIX fake runtime
// script, so they only run on Unix.
#[cfg(unix)]
mod handoff_cases;
mod rejection_cases;
mod spawn_cases;
mod support;
