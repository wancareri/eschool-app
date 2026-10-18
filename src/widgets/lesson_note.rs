use crate::app::AppState;
use crate::res;
use crate::shared::colors;
use day::prelude::*;
use std::collections::HashMap;

const NOTES_CACHE_KEY: &str = "lesson_notes_map";

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LessonNoteTarget {
    pub key: String,
    pub subject_title: String,
    pub lesson_number: u32,
    pub start_time: String,
    pub topic: Option<String>,
}

impl Route for LessonNoteTarget {
    fn key(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| self.key.clone())
    }
    fn from_key(key: &str) -> Option<Self> {
        serde_json::from_str(key).ok()
    }
}

pub fn load_notes_map() -> HashMap<String, String> {
    crate::shared::cache::load_json(NOTES_CACHE_KEY).unwrap_or_default()
}

pub fn save_notes_map(notes: &HashMap<String, String>) {
    crate::shared::cache::save_json(NOTES_CACHE_KEY, notes);
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
    crate::shared::haptics::tick();
}

pub fn delete_note(state: AppState, key: &str) {
    let mut notes = state.lesson_notes.get();
    notes.remove(key);
    save_notes_map(&notes);
    state.lesson_notes.set(notes);
    crate::shared::haptics::tick();
}

pub fn open_note(state: AppState, target: LessonNoteTarget) {
    crate::shared::nslog::nslog(&format!("[LessonNote] open_note: key='{}'", target.key));
    crate::shared::keyboard::init();
    state.active_note.set(Some(target));
}

pub fn close_note(state: AppState) {
    crate::shared::nslog::nslog("[LessonNote] close_note");
    crate::shared::keyboard::dismiss();
    crate::shared::keyboard::set_keyboard_height(0.0);
    state.active_note.set(None);
}

pub fn modal(state: AppState) -> impl Piece {
    let min_h = (crate::pages::diary::get_screen_height() * 0.40).min(320.0);
    crate::widgets::modal::modal_window(state.active_note, min_h, move |target| {
        crate::shared::nslog::nslog(&format!("[LessonNote] build_card: key='{}'", target.key));
        build_card(state, target.clone())
    })
}

fn build_card(state: AppState, target: LessonNoteTarget) -> impl Piece {
    let existing_note = get_note(state, &target.key).unwrap_or_default();
    let note_text = Signal::new(existing_note.clone());
    let has_existing = !existing_note.is_empty();

    let key_save = target.key.clone();
    let text_save = note_text.clone();
    let on_save = move || {
        let t = text_save.get();
        save_note(state, &key_save, &t);
        close_note(state);
    };

    let key_del = target.key.clone();
    let on_delete = move || {
        delete_note(state, &key_del);
        close_note(state);
    };

    let s_top = target.topic.clone();
    let time_str = crate::shared::utils::strip_seconds(&target.start_time);

    column((
        // Header
        row((
            vector(res::vectors::kind_note)
                .frame(22.0, 22.0)
                .tint(move || Color::hex(state.accent_color.get())),
            column((
                label(target.subject_title.clone())
                    .font(Font::Headline)
                    .bold(),
                label(format!("Урок {} • {}", target.lesson_number, time_str))
                    .font(Font::Caption)
                    .secondary(),
            ))
            .spacing(2.0)
            .grow(),
            label("✕")
                .font(Font::Headline)
                .secondary()
                .padding(Insets {
                    top: 10.0,
                    leading: 16.0,
                    bottom: 10.0,
                    trailing: 16.0,
                })
                .on_tap(move || close_note(state)),
        ))
        .spacing(12.0)
        .align(VAlign::Center)
        .padding(Insets {
            top: 4.0,
            leading: 20.0,
            bottom: 8.0,
            trailing: 20.0,
        }),

        // Topic if present
        when(
            move || s_top.is_some() && !s_top.as_ref().unwrap().is_empty(),
            move || {
                let top_str = target.topic.clone().unwrap_or_default();
                label(format!("Тема: {}", top_str))
                    .font(Font::Caption)
                    .secondary()
                    .padding(Insets {
                        top: 0.0,
                        leading: 20.0,
                        bottom: 8.0,
                        trailing: 20.0,
                    })
            },
        ),

        // Text input field
        column((
            text_field(note_text.clone())
                .placeholder("Заметка к уроку (видна только в приложении)...")
                .on_submit(move || {
                    crate::shared::keyboard::dismiss();
                }),
        ))
        .padding(Insets {
            top: 12.0,
            leading: 14.0,
            bottom: 12.0,
            trailing: 14.0,
        })
        .background(move || {
            if day::dark_mode() {
                Color::rgba(0.22, 0.22, 0.25, 1.0)
            } else {
                Color::rgba(0.92, 0.92, 0.95, 1.0)
            }
        })
        .corner_radius(12.0)
        .padding(Insets {
            top: 4.0,
            leading: 20.0,
            bottom: 12.0,
            trailing: 20.0,
        }),

        // Action buttons
        row((
            when(
                move || has_existing,
                move || {
                    let del = on_delete.clone();
                    label("Удалить")
                        .font(Font::Subheadline)
                        .color(colors::ERROR)
                        .padding(Insets {
                            top: 10.0,
                            leading: 16.0,
                            bottom: 10.0,
                            trailing: 16.0,
                        })
                        .on_tap(del)
                        .any()
                },
            ),
            spacer(),
            label("Сохранить")
                .font(Font::Headline)
                .color(Color::WHITE)
                .padding(Insets {
                    top: 10.0,
                    leading: 24.0,
                    bottom: 10.0,
                    trailing: 24.0,
                })
                .background(move || Color::hex(state.accent_color.get()))
                .corner_radius(20.0)
                .on_tap(on_save),
        ))
        .spacing(8.0)
        .align(VAlign::Center)
        .padding(Insets {
            top: 8.0,
            leading: 20.0,
            bottom: 24.0,
            trailing: 20.0,
        }),
    ))
    .spacing(4.0)
}
