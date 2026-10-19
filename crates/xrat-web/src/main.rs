mod api;
mod app;
mod components;
mod hooks;
mod pages;
mod routing;
mod shell;
mod ui;

pub use app::{Notice, Session};
pub use ui::feedback::Feedback;

fn main() {
    dioxus::launch(app::App);
}
