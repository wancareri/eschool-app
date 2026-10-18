use crate::app::AppState;
use crate::shared::{colors, utils};
use day::prelude::*;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LessonDetailTarget {
    pub key: String,
    pub subject_title: String,
    pub lesson_number: u32,
    pub start_time: String,
    pub end_time: Option<String>,
    pub topic: Option<String>,
    pub homework: Option<String>,
    pub teacher: Option<String>,
    pub comment: Option<String>,
    pub mark: Option<String>,
}

impl Route for LessonDetailTarget {
    fn key(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| self.key.clone())
    }
    fn from_key(key: &str) -> Option<Self> {
        serde_json::from_str(key).ok()
    }
}

pub fn open_peek(state: AppState, target: LessonDetailTarget) {
    crate::shared::haptics::tick();
    state.active_lesson_peek.set(Some(target));
}

pub fn close_peek(state: AppState) {
    state.active_lesson_peek.set(None);
}

pub fn modal(state: AppState) -> impl Piece {
    let min_h = (crate::pages::diary::get_screen_height() * 0.45).min(380.0);
    crate::widgets::modal::modal_window(state.active_lesson_peek, min_h, move |target| {
        build_card(state, target.clone())
    })
}

fn build_card(state: AppState, target: LessonDetailTarget) -> impl Piece {
    let target = std::sync::Arc::new(target);
    let key = target.key.clone();
    let note = crate::widgets::lesson_note::get_note(state, &key);
    let s_target = target.clone();
    let s_del_key = key.clone();

    let initial = target
        .subject_title
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "У".into());

    let time_str = {
        let start = utils::strip_seconds(&target.start_time);
        if let Some(end) = target.end_time.as_ref() {
            format!("{} – {}", start, utils::strip_seconds(end))
        } else {
            start
        }
    };

    let subject_title = target.subject_title.clone();
    let lesson_num_str = format!("Урок {}", target.lesson_number);

    let t_teacher = target.teacher.clone();
    let t_topic = target.topic.clone();
    let t_homework = target.homework.clone();
    let t_comment = target.comment.clone();
    let t_mark = target.mark.clone();

    column((
        // Header: Subject + Number & Time
        row((
            // Subject initial avatar circle
            circle()
                .fill(move || Color::hex(state.accent_color.get()).with_alpha(0.18))
                .frame(44.0, 44.0)
                .overlay(
                    label(initial)
                        .font(Font::Headline)
                        .bold()
                        .color(move || Color::hex(state.accent_color.get()))
                        .align(TextAlign::Center),
                ),

            column((
                label(subject_title)
                    .font(Font::Title2)
                    .bold()
                    .grow(),
                row((
                    label(lesson_num_str)
                        .font(Font::Subheadline)
                        .weight(FontWeight::Semibold)
                        .color(move || Color::hex(state.accent_color.get())),
                    label("•").font(Font::Subheadline).secondary(),
                    label(time_str)
                        .font(Font::Subheadline)
                        .secondary(),
                ))
                .spacing(6.0),
            ))
            .spacing(2.0)
            .grow(),

            // Mark badge if available
            if let Some(m_val) = t_mark {
                let m_col = utils::grade_color(&m_val);
                capsule()
                    .fill(m_col.with_alpha(0.18))
                    .frame(36.0, 36.0)
                    .overlay(
                        label(m_val)
                            .font(Font::Title3)
                            .bold()
                            .color(m_col)
                            .align(TextAlign::Center),
                    )
                    .any()
            } else {
                spacer().width(0.0).any()
            },
        ))
        .spacing(12.0)
        .align(VAlign::Center)
        .padding(Insets { top: 0.0, leading: 0.0, bottom: 10.0, trailing: 0.0 }),

        divider().padding(Insets { top: 0.0, leading: 0.0, bottom: 8.0, trailing: 0.0 }),

        // Teacher row (if available)
        if let Some(t_name) = t_teacher.filter(|t| !t.trim().is_empty()) {
            row((
                label("👤").font(Font::Callout),
                column((
                    label("Преподаватель").font(Font::Caption2).secondary(),
                    label(t_name).font(Font::Subheadline).weight(FontWeight::Medium),
                ))
                .spacing(1.0)
                .grow(),
            ))
            .spacing(10.0)
            .align(VAlign::Center)
            .padding(Insets { top: 4.0, leading: 4.0, bottom: 6.0, trailing: 4.0 })
            .any()
        } else {
            spacer().height(0.0).any()
        },

        // Topic (Тема урока)
        if let Some(topic_txt) = t_topic.filter(|t| !t.trim().is_empty()) {
            column((
                label("Тема урока").font(Font::Caption2).secondary(),
                label(topic_txt)
                    .font(Font::Body)
                    .weight(FontWeight::Medium),
            ))
            .spacing(2.0)
            .padding(Insets { top: 6.0, leading: 10.0, bottom: 6.0, trailing: 10.0 })
            .background(move || {
                if day::dark_mode() {
                    Color::rgba(1.0, 1.0, 1.0, 0.05)
                } else {
                    Color::rgba(0.0, 0.0, 0.0, 0.04)
                }
            })
            .corner_radius(10.0)
            .any()
        } else {
            spacer().height(0.0).any()
        },

        // Homework (Домашнее задание)
        if let Some(hw_txt) = t_homework.filter(|h| !h.trim().is_empty()) {
            column((
                label("Домашнее задание")
                    .font(Font::Caption2)
                    .bold()
                    .color(colors::ACCENT),
                label(hw_txt)
                    .font(Font::Body)
                    .weight(FontWeight::Medium),
            ))
            .spacing(2.0)
            .padding(Insets { top: 6.0, leading: 10.0, bottom: 6.0, trailing: 10.0 })
            .background(move || colors::ACCENT.with_alpha(0.08))
            .corner_radius(10.0)
            .any()
        } else {
            spacer().height(0.0).any()
        },

        // Teacher's comment on mark
        if let Some(c_txt) = t_comment.filter(|c| !c.trim().is_empty()) {
            column((
                label("Комментарий к оценке").font(Font::Caption2).secondary(),
                label(c_txt)
                    .font(Font::Callout)
                    .italic()
                    .secondary(),
            ))
            .spacing(2.0)
            .padding(Insets { top: 4.0, leading: 10.0, bottom: 4.0, trailing: 10.0 })
            .any()
        } else {
            spacer().height(0.0).any()
        },

        spacer().height(8.0),

        // Note section: If note exists, show note and edit/delete. If not, prominent "Add note" button.
        if let Some(existing_text) = note {
            let edit_target = s_target.clone();
            let note_display = existing_text.clone();
            column((
                column((
                    row((
                        label("📝 Ваша заметка").font(Font::Caption2).bold().color(move || Color::hex(state.accent_color.get())),
                        spacer().grow(),
                        label("видна только вам").font(Font::Caption2).secondary(),
                    )),
                    label(move || note_display.clone())
                        .font(Font::Body)
                        .padding(Insets { top: 2.0, leading: 0.0, bottom: 2.0, trailing: 0.0 }),
                ))
                .spacing(3.0)
                .padding(Insets { top: 8.0, leading: 12.0, bottom: 8.0, trailing: 12.0 })
                .background(move || Color::hex(state.accent_color.get()).with_alpha(0.08))
                .corner_radius(12.0),

                row((
                    button("Изменить")
                        .action(move || {
                            let note_t = crate::widgets::lesson_note::LessonNoteTarget {
                                key: edit_target.key.clone(),
                                subject_title: edit_target.subject_title.clone(),
                                lesson_number: edit_target.lesson_number,
                                start_time: edit_target.start_time.clone(),
                                topic: edit_target.topic.clone(),
                            };
                            close_peek(state);
                            crate::widgets::lesson_note::open_note(state, note_t);
                        })
                        .grow(),

                    button("Удалить")
                        .action(move || {
                            crate::widgets::lesson_note::delete_note(state, &s_del_key);
                            close_peek(state);
                        })
                        .padding(Insets { top: 4.0, leading: 16.0, bottom: 4.0, trailing: 16.0 }),
                ))
                .spacing(10.0)
                .padding(Insets { top: 4.0, leading: 0.0, bottom: 0.0, trailing: 0.0 }),
            ))
            .spacing(6.0)
            .any()
        } else {
            let add_target = s_target.clone();
            column((
                button("📝 Добавить заметку")
                    .prominent()
                    .action(move || {
                        let note_t = crate::widgets::lesson_note::LessonNoteTarget {
                            key: add_target.key.clone(),
                            subject_title: add_target.subject_title.clone(),
                            lesson_number: add_target.lesson_number,
                            start_time: add_target.start_time.clone(),
                            topic: add_target.topic.clone(),
                        };
                        close_peek(state);
                        crate::widgets::lesson_note::open_note(state, note_t);
                    })
                    .grow(),
                label("Заметка сохраняется локально и видна только вам")
                    .font(Font::Caption2)
                    .secondary()
                    .align(TextAlign::Center),
            ))
            .spacing(6.0)
            .any()
        },
    ))
    .spacing(8.0)
    .padding(Insets { top: 4.0, leading: 16.0, bottom: 12.0, trailing: 16.0 })
}
