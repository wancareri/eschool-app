//! Persistence and state updates for per-lesson notes.

use crate::app::AppState;
use std::collections::HashMap;

const NOTES_CACHE_KEY: &str = "lesson_notes_map";

pub fn load_notes_map() -> HashMap<String, String> {
    crate::app::data::cache::load_json(NOTES_CACHE_KEY).unwrap_or_default()
}

fn save_notes_map(notes: &HashMap<String, String>) {
    crate::app::data::cache::save_json(NOTES_CACHE_KEY, notes);
}

pub fn get_note(state: AppState, key: &str) -> Option<String> {
    state
        .lesson_notes
        .with(|m| m.get(key).cloned())
        .filter(|s| !s.trim().is_empty())
}

pub fn save_note(state: AppState, key: &str, text: &str) {
    let mut notes = state.lesson_notes.get();
    let trimmed = text.trim();
    if trimmed.is_empty() {
        notes.remove(key);
    } else {
        notes.insert(key.to_string(), trimmed.to_string());
    }
    save_notes_map(&notes);
    state.lesson_notes.set(notes);
    crate::app::preferences::haptics::tick();
}

pub fn delete_note(state: AppState, key: &str) {
    let mut notes = state.lesson_notes.get();
    notes.remove(key);
    save_notes_map(&notes);
    state.lesson_notes.set(notes);
    crate::app::preferences::haptics::tick();
}
