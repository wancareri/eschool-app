use crate::app::AppState;
use day::prelude::*;

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

pub fn open_note(state: AppState, target: LessonNoteTarget) {
    crate::ios::nslog::nslog(&format!("[LessonNote] open_note forwarding to peek: key='{}'", target.key));
    let peek_target = crate::pages::diary::lesson_peek::LessonDetailTarget {
        key: target.key.clone(),
        lesson_uuid: target.key,
        subject_title: target.subject_title,
        lesson_number: target.lesson_number,
        date_str: None,
        start_time: target.start_time,
        end_time: None,
        topic: target.topic,
        homework: None,
        teacher: None,
        comment: None,
        mark: None,
        has_attachments: false,
        is_substitution: false,
        start_editing_note: true,
    };
    crate::pages::diary::lesson_peek::open_peek(state, peek_target);
}

pub fn close_note(state: AppState) {
    crate::ios::nslog::nslog("[LessonNote] close_note");
    crate::ios::keyboard::dismiss();
    state.active_note.set(None);
}

pub fn modal(_state: AppState) -> impl Piece {
    spacer().height(0.0)
}
