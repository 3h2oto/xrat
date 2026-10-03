mod chrome;
mod configs;
mod modals;
mod shared;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::tui::app::TuiApp;

mod render;
pub use render::render;
