use crate::app::context::AppContext;
use crate::app::daemon::ipc::PingPayload;
use crate::app::daemon::ipc::RotationTrigger;
use crate::app::daemon::supervisor::{SupervisorEvent, SupervisorState};
use tokio::sync::oneshot;

mod health;
mod runtime;

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

mod dispatch;
#[cfg(test)]
pub use dispatch::handle_event;
pub(super) use dispatch::handle_event_with_sender;
