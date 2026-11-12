//! Eschool App — a Day framework cross-platform school diary.

pub mod app;
pub mod pages;
pub mod ios;
pub mod components;

pub mod swiftui {
    include!(concat!(env!("OUT_DIR"), "/day_swiftui.rs"));
}

pub use app::{AppState, Section};
pub use app::window::{window, root};

day::day_start!(options: app::window(), app::root);

day::resources!();

pub(crate) const THEME_KEY: &str = "app.theme";
pub(crate) const LOCALE_KEY: &str = "app.locale";
