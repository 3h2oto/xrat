mod state;
#[cfg(test)]
mod tests;

use crate::tui::data::{TuiConfigRow, TuiData};

pub use state::TuiTaskState;

mod events;
pub use events::TuiTaskEvent;
pub use events::TuiTaskKind;
