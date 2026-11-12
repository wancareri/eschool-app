//! Shared native page chrome. One window toolbar owns all trailing commands.
use crate::app::{AppState, Section};
use crate::res;
use day::prelude::*;

pub fn page<R: Route, M>(
    title: impl IntoText<M>,
    path: Signal<Vec<R>>,
    content: impl Piece,
) -> NavStack<Signal<Vec<R>>, R> {
    nav_stack(path, content).title(title)
}

pub fn section_title(section: Section) -> String {
    match section {
        Section::Diary => res::str::diary_title().format(),
        Section::Schedule => res::str::schedule_title().format(),
        Section::Teachers => res::str::teachers_title().format(),
        Section::Settings => res::str::settings_title().format(),
        Section::Login => res::str::auth_username_title().format(),
    }
}

pub fn toolbar(state: AppState) -> Vec<ToolbarEntry> {
    if state.pin_lock_active.get() || state.pin_setup_active.get() {
        Vec::new()
    } else if state.is_authenticated.get() {
        super::connection_status::toolbar(state)
    } else {
        crate::pages::login::auth_toolbar()
    }
}
