use super::super::*;

mod health_cooldown_blocks;
mod health_cooldown_suppressed;
mod helpers;
mod manual_failure;
// The override and timer replacements start a real runtime through a POSIX
// fake runtime script, so they only run on Unix.
#[cfg(unix)]
mod manual_override_success;
mod status_fields;
mod timer_failure;
#[cfg(unix)]
mod timer_success;
