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
    #[serde(default)]
    pub has_attachments: bool,
    #[serde(default)]
    pub is_substitution: bool,
    #[serde(default)]
    pub start_editing_note: bool,
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
    crate::shared::nslog::nslog(&format!("[LessonPeek] open_peek: subject='{}'", target.subject_title));
    crate::shared::haptics::tick();
    state.active_lesson_peek.set(Some(target));
}

pub fn close_peek(state: AppState) {
    crate::shared::nslog::nslog("[LessonPeek] close_peek");
    crate::shared::keyboard::dismiss();
    state.active_lesson_peek.set(None);
}

pub fn modal(state: AppState) -> impl Piece {
    let min_h = (crate::pages::diary::get_screen_height() * 0.45).min(380.0);
    crate::widgets::modal::modal_window(state.active_lesson_peek, min_h, move |target| {
        crate::shared::nslog::nslog(&format!("[LessonPeek] build_card: subject='{}'", target.subject_title));
        build_card(state, target.clone())
    })
}

fn build_card(state: AppState, target: LessonDetailTarget) -> impl Piece {
    let key = target.key.clone();
    let is_editing = Signal::new(target.start_editing_note);
    let edit_text = Signal::new(crate::widgets::lesson_note::get_note(state, &key).unwrap_or_default());

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
    let has_attachments = target.has_attachments;
    let is_substitution = target.is_substitution;

    let k_save = key.clone();
    let txt_save = edit_text.clone();
    let ed_save = is_editing.clone();
    let on_save = move || {
        let t = txt_save.get();
        crate::widgets::lesson_note::save_note(state, &k_save, &t);
        ed_save.set(false);
        crate::shared::keyboard::dismiss();
    };

    let k_cancel = key.clone();
    let txt_cancel = edit_text.clone();
    let ed_cancel = is_editing.clone();
    let on_cancel = move || {
        let prev = crate::widgets::lesson_note::get_note(state, &k_cancel).unwrap_or_default();
        txt_cancel.set(prev);
        ed_cancel.set(false);
        crate::shared::keyboard::dismiss();
    };

    let k_del_edit = key.clone();
    let txt_del_edit = edit_text.clone();
    let ed_del_edit = is_editing.clone();
    let on_del_edit = move || {
        crate::widgets::lesson_note::delete_note(state, &k_del_edit);
        txt_del_edit.set("".to_string());
        ed_del_edit.set(false);
        crate::shared::keyboard::dismiss();
    };

    let k_del_view = key.clone();
    let txt_del_view = edit_text.clone();
    let on_del_view = move || {
        crate::widgets::lesson_note::delete_note(state, &k_del_view);
        txt_del_view.set("".to_string());
    };

    let k_start_edit = key.clone();
    let txt_start_edit = edit_text.clone();
    let ed_start_edit = is_editing.clone();
    let on_start_edit = move || {
        let cur = crate::widgets::lesson_note::get_note(state, &k_start_edit).unwrap_or_default();
        txt_start_edit.set(cur);
        ed_start_edit.set(true);
    };

    let txt_add = edit_text.clone();
    let ed_add = is_editing.clone();
    let on_add_note = move || {
        txt_add.set("".to_string());
        ed_add.set(true);
    };

    let k_chk1 = key.clone();
    let k_chk2 = key.clone();
    let k_lbl = key.clone();
    let k_del_btn_chk = key.clone();

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
        .padding(Insets { top: 0.0, leading: 0.0, bottom: 8.0, trailing: 0.0 }),

        // Indicator pills: Attachments / Substitution
        if has_attachments || is_substitution {
            row((
                if has_attachments {
                    row((
                        vector(crate::res::vectors::attachment)
                            .frame(12.0, 12.0)
                            .tint(move || Color::hex(state.accent_color.get())),
                        label("Прикреплены файлы")
                            .font(Font::Caption2)
                            .weight(FontWeight::Medium)
                            .color(move || Color::hex(state.accent_color.get())),
                    ))
                    .spacing(5.0)
                    .align(VAlign::Center)
                    .padding(Insets { top: 3.0, leading: 8.0, bottom: 3.0, trailing: 8.0 })
                    .background(move || Color::hex(state.accent_color.get()).with_alpha(0.12))
                    .corner_radius(6.0)
                    .any()
                } else {
                    spacer().width(0.0).any()
                },
                if is_substitution {
                    row((
                        vector(crate::res::vectors::substitution)
                            .frame(12.0, 12.0)
                            .tint(move || Color::hex(state.accent_color.get())),
                        label("Замена урока")
                            .font(Font::Caption2)
                            .weight(FontWeight::Medium)
                            .color(move || Color::hex(state.accent_color.get())),
                    ))
                    .spacing(5.0)
                    .align(VAlign::Center)
                    .padding(Insets { top: 3.0, leading: 8.0, bottom: 3.0, trailing: 8.0 })
                    .background(move || Color::hex(state.accent_color.get()).with_alpha(0.12))
                    .corner_radius(6.0)
                    .any()
                } else {
                    spacer().width(0.0).any()
                },
            ))
            .spacing(8.0)
            .align(VAlign::Center)
            .padding(Insets { top: 0.0, leading: 0.0, bottom: 6.0, trailing: 0.0 })
            .any()
        } else {
            spacer().height(0.0).any()
        },

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

        spacer().height(6.0),

        // 1. Inline note editor mode
        when(
            move || is_editing.get(),
            move || {
                let save = on_save.clone();
                let cancel = on_cancel.clone();
                let del = on_del_edit.clone();
                let k_del_chk = k_del_btn_chk.clone();
                column((
                    row((
                        vector(crate::res::vectors::kind_note)
                            .frame(15.0, 15.0)
                            .tint(move || Color::hex(state.accent_color.get())),
                        label("Заметка к уроку")
                            .font(Font::Caption)
                            .bold()
                            .color(move || Color::hex(state.accent_color.get())),
                        spacer().grow(),
                        label("видна только вам")
                            .font(Font::Caption2)
                            .secondary(),
                    ))
                    .spacing(6.0)
                    .align(VAlign::Center),

                    column((
                        text_field(edit_text.clone())
                            .placeholder("Заметка к уроку (сохраняется на устройстве)...")
                            .on_submit(save.clone()),
                    ))
                    .padding(Insets { top: 8.0, leading: 10.0, bottom: 8.0, trailing: 10.0 })
                    .background(move || {
                        if day::dark_mode() {
                            Color::rgba(0.22, 0.22, 0.25, 1.0)
                        } else {
                            Color::rgba(0.92, 0.92, 0.95, 1.0)
                        }
                    })
                    .corner_radius(10.0),

                    row((
                        button("Отмена")
                            .action(cancel),
                        spacer().grow(),
                        when(
                            move || {
                                state.lesson_notes.with(|m| m.get(&k_del_chk).map(|s| !s.trim().is_empty()).unwrap_or(false))
                            },
                            move || {
                                button("Удалить")
                                    .action(del.clone())
                            },
                        ),
                        button("Сохранить")
                            .prominent()
                            .action(save),
                    ))
                    .spacing(8.0)
                    .align(VAlign::Center),
                ))
                .spacing(8.0)
                .padding(Insets { top: 8.0, leading: 12.0, bottom: 8.0, trailing: 12.0 })
                .background(move || Color::hex(state.accent_color.get()).with_alpha(0.08))
                .corner_radius(12.0)
            },
        ),

        // 2. Note exists, not editing
        when(
            move || !is_editing.get() && state.lesson_notes.with(|m| m.get(&k_chk1).map(|s| !s.trim().is_empty()).unwrap_or(false)),
            move || {
                let k_label = k_lbl.clone();
                let start_edit = on_start_edit.clone();
                let del = on_del_view.clone();
                column((
                    row((
                        vector(crate::res::vectors::kind_note)
                            .frame(15.0, 15.0)
                            .tint(move || Color::hex(state.accent_color.get())),
                        label("Ваша заметка")
                            .font(Font::Caption)
                            .bold()
                            .color(move || Color::hex(state.accent_color.get())),
                        spacer().grow(),
                        label("видна только вам")
                            .font(Font::Caption2)
                            .secondary(),
                    ))
                    .spacing(6.0)
                    .align(VAlign::Center),

                    label(move || {
                        state.lesson_notes.with(|m| m.get(&k_label).cloned().unwrap_or_default())
                    })
                    .font(Font::Body)
                    .padding(Insets { top: 2.0, leading: 0.0, bottom: 2.0, trailing: 0.0 }),

                    row((
                        button("Изменить")
                            .action(start_edit)
                            .grow(),
                        button("Удалить")
                            .action(del)
                            .padding(Insets { top: 4.0, leading: 16.0, bottom: 4.0, trailing: 16.0 }),
                    ))
                    .spacing(10.0),
                ))
                .spacing(6.0)
                .padding(Insets { top: 8.0, leading: 12.0, bottom: 8.0, trailing: 12.0 })
                .background(move || Color::hex(state.accent_color.get()).with_alpha(0.08))
                .corner_radius(12.0)
            },
        ),

        // 3. No note exists, not editing
        when(
            move || !is_editing.get() && !state.lesson_notes.with(|m| m.get(&k_chk2).map(|s| !s.trim().is_empty()).unwrap_or(false)),
            move || {
                let add = on_add_note.clone();
                column((
                    row((
                        vector(crate::res::vectors::kind_note)
                            .frame(16.0, 16.0)
                            .tint(move || Color::WHITE),
                        label("Добавить заметку")
                            .font(Font::Headline)
                            .bold()
                            .color(Color::WHITE),
                    ))
                    .spacing(8.0)
                    .align(VAlign::Center)
                    .padding(Insets { top: 12.0, leading: 16.0, bottom: 12.0, trailing: 16.0 })
                    .background(move || Color::hex(state.accent_color.get()))
                    .corner_radius(12.0)
                    .on_tap(add),

                    label("Заметка сохраняется локально и видна только вам")
                        .font(Font::Caption2)
                        .secondary()
                        .align(TextAlign::Center),
                ))
                .spacing(6.0)
            },
        ),
    ))
    .spacing(8.0)
    .padding(Insets { top: 4.0, leading: 16.0, bottom: 12.0, trailing: 16.0 })
}
