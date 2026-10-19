use crate::app::AppState;
use crate::shared::utils;
use day::prelude::*;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LessonDetailTarget {
    pub key: String,
    #[serde(default)]
    pub lesson_uuid: String,
    pub subject_title: String,
    pub lesson_number: u32,
    #[serde(default)]
    pub date_str: Option<String>,
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
    crate::shared::haptics::pop();
    state.active_lesson_peek.set(Some(target));
}

pub fn close_peek(state: AppState) {
    crate::shared::nslog::nslog("[LessonPeek] close_peek");
    crate::shared::keyboard::dismiss();
    state.active_lesson_peek.set(None);
}

pub fn modal(state: AppState) -> impl Piece {
    let min_h = (crate::pages::diary::get_screen_height() * 0.50).min(460.0);
    crate::widgets::modal::modal_window(state.active_lesson_peek, min_h, move |target| {
        crate::shared::nslog::nslog(&format!("[LessonPeek] build_card: subject='{}'", target.subject_title));
        build_card(state, target.clone())
    })
}

fn attachment_row(state: AppState, att: crate::features::attachments::LessonAttachment) -> impl Piece {
    let is_link = att.is_link;
    let title_str = att.title.clone();
    let size_str = att.file_size.clone();
    let att_open = att.clone();
    let att_share = att.clone();
    let att_id = if !att.id.is_empty() { att.id.clone() } else { att.url.clone() };

    row((
        // Left icon
        if is_link {
            label("🔗")
                .font(Font::Headline)
                .any()
        } else {
            vector(crate::res::vectors::attachment)
                .frame(16.0, 16.0)
                .tint(move || Color::hex(state.accent_color.get()))
                .any()
        },

        // Title and size column
        column((
            label(title_str)
                .font(Font::Subheadline)
                .weight(FontWeight::Medium)
                .grow(),
            if let Some(sz) = size_str {
                label(sz)
                    .font(Font::Caption2)
                    .secondary()
                    .any()
            } else if is_link {
                label("Внешняя ссылка")
                    .font(Font::Caption2)
                    .secondary()
                    .any()
            } else {
                spacer().height(0.0).any()
            },
        ))
        .spacing(1.0)
        .grow(),

        // Action buttons (vertical column of clean icons)
        if is_link {
            let on_open = move || {
                crate::features::attachments::open_or_preview_attachment(state, &att_open);
            };
            vector(crate::res::vectors::external_link)
                .frame(18.0, 18.0)
                .tint(move || Color::hex(state.accent_color.get()))
                .padding(6.0)
                .on_tap(on_open)
                .any()
        } else {
            let id_check = att_id.clone();
            let open_fn = move || {
                crate::features::attachments::open_or_preview_attachment(state, &att_open);
            };
            let share_fn = move || {
                crate::features::attachments::share_attachment(state, &att_share);
            };
            column((
                // QuickLook / View icon button
                zstack((
                    when(
                        move || state.downloading_files.with(|s| s.contains(&id_check)),
                        move || crate::widgets::spinner::render(state, 16.0).any(),
                    )
                    .otherwise(move || {
                        vector(crate::res::vectors::eye)
                            .frame(18.0, 18.0)
                            .tint(move || Color::hex(state.accent_color.get()))
                            .any()
                    }),
                ))
                .padding(Insets { top: 4.0, leading: 6.0, bottom: 2.0, trailing: 6.0 })
                .on_tap(open_fn),

                // Share / Save to Files / Download icon button
                vector(crate::res::vectors::download)
                    .frame(18.0, 18.0)
                    .tint(move || Color::hex(state.accent_color.get()))
                    .padding(Insets { top: 2.0, leading: 6.0, bottom: 4.0, trailing: 6.0 })
                    .on_tap(share_fn),
            ))
            .spacing(0.0)
            .align(HAlign::Center)
            .any()
        },
    ))
    .spacing(10.0)
    .align(VAlign::Center)
    .padding(Insets { top: 6.0, leading: 8.0, bottom: 6.0, trailing: 8.0 })
    .background(move || {
        if day::dark_mode() {
            Color::rgba(1.0, 1.0, 1.0, 0.05)
        } else {
            Color::rgba(0.0, 0.0, 0.0, 0.03)
        }
    })
    .corner_radius(8.0)
}

fn build_card(state: AppState, target: LessonDetailTarget) -> impl Piece {
    let key = target.key.clone();
    let is_editing = Signal::new(target.start_editing_note);
    let edit_text = Signal::new(crate::widgets::lesson_note::get_note(state, &key).unwrap_or_default());

    let time_str = {
        let start = utils::strip_seconds(&target.start_time);
        if let Some(end) = target.end_time.as_ref() {
            format!("{} – {}", start, utils::strip_seconds(end))
        } else {
            start
        }
    };

    let subject_title = target.subject_title.clone();
    let lesson_num_str = format!("{} урок", target.lesson_number);
    let date_display = target.date_str.clone();

    let t_teacher = target.teacher.clone();
    let t_topic = target.topic.clone().filter(|t| !t.trim().is_empty());
    let t_homework = target.homework.clone().filter(|h| !h.trim().is_empty());
    let t_comment = target.comment.clone().filter(|c| !c.trim().is_empty());
    let t_mark = target.mark.clone().filter(|m| !m.trim().is_empty());
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

    // Attachments fetching
    let uuid_for_att = if !target.lesson_uuid.is_empty() {
        target.lesson_uuid.clone()
    } else {
        target.key.clone()
    };
    let has_attachments = target.has_attachments
        || (!uuid_for_att.is_empty() && state.lesson_attachments.with(|m| m.get(&uuid_for_att).map_or(false, |list| !list.is_empty())));
    if (has_attachments || !target.lesson_uuid.is_empty()) && !uuid_for_att.is_empty() {
        crate::features::attachments::fetch_attachments_async(state, &uuid_for_att);
    }
    let u_load = uuid_for_att.clone();
    let u_check = uuid_for_att.clone();
    let u_list = uuid_for_att.clone();

    column((
        // ── 1. ШАПКА: Предмет, Дата, Время, Урок, Учитель, Отметка ──────
        column((
            // Верхняя строка: Название предмета (и бейдж замены) слева + Отметка справа
            row((
                column((
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
                        .spacing(4.0)
                        .align(VAlign::Center)
                        .padding(Insets { top: 2.0, leading: 6.0, bottom: 2.0, trailing: 6.0 })
                        .background(move || Color::hex(state.accent_color.get()).with_alpha(0.12))
                        .corner_radius(6.0)
                        .any()
                    } else {
                        spacer().height(0.0).any()
                    },

                    label(subject_title)
                        .font(Font::Title)
                        .bold()
                        .align(TextAlign::Leading),
                ))
                .spacing(3.0)
                .grow(),

                if let Some(ref m_val) = t_mark {
                    let m_col = utils::grade_color(m_val);
                    column((
                        label(m_val.clone())
                            .font(Font::LargeTitle)
                            .bold()
                            .color(m_col),
                        if let Some(ref c) = t_comment {
                            label(c.clone())
                                .font(Font::Caption2)
                                .secondary()
                                .italic()
                                .any()
                        } else {
                            spacer().height(0.0).any()
                        },
                    ))
                    .spacing(1.0)
                    .align(HAlign::Trailing)
                    .any()
                } else {
                    spacer().width(0.0).any()
                },
            ))
            .align(VAlign::Center),

            // Единая строка деталей: Дата • Урок (время) • Преподаватель
            row((
                if let Some(ref d_str) = date_display {
                    row((
                        label(d_str.clone())
                            .font(Font::Subheadline)
                            .secondary(),
                        label("•").font(Font::Subheadline).secondary(),
                    ))
                    .spacing(5.0)
                    .any()
                } else {
                    spacer().width(0.0).any()
                },
                label(format!("{lesson_num_str} ({time_str})"))
                    .font(Font::Subheadline)
                    .secondary(),
                if let Some(ref t_name) = t_teacher.filter(|t| !t.trim().is_empty()) {
                    row((
                        label("•").font(Font::Subheadline).secondary(),
                        label(t_name.clone())
                            .font(Font::Subheadline)
                            .weight(FontWeight::Medium)
                            .secondary(),
                    ))
                    .spacing(5.0)
                    .any()
                } else {
                    spacer().width(0.0).any()
                },
            ))
            .spacing(5.0)
            .align(VAlign::Center),
        ))
        .spacing(6.0)
        .align(HAlign::Leading),

        divider().padding(Insets { top: 12.0, leading: 0.0, bottom: 12.0, trailing: 0.0 }),

        // ── 2. ПОСЕРЕДИНЕ: Тема урока + Домашнее задание ────────────
        column((
            // Тема
            column((
                label("ТЕМА УРОКА")
                    .font(Font::Caption)
                    .weight(FontWeight::Semibold)
                    .secondary(),
                if let Some(ref top) = t_topic {
                    label(top.clone())
                        .font(Font::Body)
                        .any()
                } else {
                    label("Тема не указана")
                        .font(Font::Body)
                        .secondary()
                        .italic()
                        .any()
                },
            ))
            .spacing(3.0)
            .align(HAlign::Leading),

            // Домашнее задание
            column((
                label("ДОМАШНЕЕ ЗАДАНИЕ")
                    .font(Font::Caption)
                    .weight(FontWeight::Semibold)
                    .color(move || Color::hex(state.accent_color.get())),
                if let Some(ref hw) = t_homework {
                    label(hw.clone())
                        .font(Font::Body)
                        .weight(FontWeight::Medium)
                        .any()
                } else {
                    label("Задание не задано")
                        .font(Font::Body)
                        .secondary()
                        .italic()
                        .any()
                },
            ))
            .spacing(3.0)
            .align(HAlign::Leading),
        ))
        .spacing(12.0)
        .align(HAlign::Leading),

        divider().padding(Insets { top: 12.0, leading: 0.0, bottom: 12.0, trailing: 0.0 }),

        // ── 3. СНИЗУ: Файлы (Материалы) + Заметки ────────────────────
        column((
            // Материалы к уроку (файлы)
            if has_attachments || !target.lesson_uuid.is_empty() {
                column((
                    row((
                        vector(crate::res::vectors::attachment)
                            .frame(14.0, 14.0)
                            .tint(move || Color::hex(state.accent_color.get())),
                        label("МАТЕРИАЛЫ К УРОКУ")
                            .font(Font::Caption)
                            .weight(FontWeight::Semibold)
                            .secondary(),
                        spacer().grow(),
                        when(
                            move || state.attachments_loading.with(|s| s.contains(&u_load)),
                            move || label("загрузка...").font(Font::Caption2).secondary(),
                        ),
                    ))
                    .spacing(6.0)
                    .align(VAlign::Center),

                    when(
                        move || {
                            state.lesson_attachments.with(|m| m.get(&u_check).map(|v| v.is_empty()).unwrap_or(false))
                                && !state.attachments_loading.with(|s| s.contains(&u_check))
                        },
                        move || {
                            label("Нет прикреплённых файлов к этому уроку")
                                .font(Font::Caption)
                                .secondary()
                                .italic()
                                .padding(Insets { top: 2.0, leading: 0.0, bottom: 2.0, trailing: 0.0 })
                        },
                    ),

                    each(
                        items(
                            move || state.lesson_attachments.with(|m| m.get(&u_list).cloned().unwrap_or_default()),
                            |att: &crate::features::attachments::LessonAttachment| {
                                if !att.id.is_empty() { att.id.clone() } else { att.url.clone() }
                            },
                        ),
                        move |slot| {
                            let att = slot.with(|a| a.clone());
                            attachment_row(state, att).any()
                        },
                    ),

                    divider().padding(Insets { top: 10.0, leading: 0.0, bottom: 10.0, trailing: 0.0 }),
                ))
                .spacing(6.0)
                .align(HAlign::Leading)
                .any()
            } else {
                spacer().height(0.0).any()
            },

        // ── 7. ЗАМЕТКА К УРОКУ (NOTE) ────────────────────────────────
        // 1. Inline editing mode
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
                            .frame(14.0, 14.0)
                            .tint(move || Color::hex(state.accent_color.get())),
                        label("ЗАМЕТКА К УРОКУ")
                            .font(Font::Caption)
                            .weight(FontWeight::Semibold)
                            .color(move || Color::hex(state.accent_color.get())),
                        spacer().grow(),
                        label("видна только вам")
                            .font(Font::Caption2)
                            .secondary(),
                    ))
                    .spacing(6.0)
                    .align(VAlign::Center),

                    text_field(edit_text.clone())
                        .placeholder("Заметка к уроку (сохраняется на устройстве)...")
                        .on_submit(save.clone())
                        .padding(Insets { top: 6.0, leading: 0.0, bottom: 6.0, trailing: 0.0 }),

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
                .align(HAlign::Leading)
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
                            .frame(14.0, 14.0)
                            .tint(move || Color::hex(state.accent_color.get())),
                        label("ВАША ЗАМЕТКА")
                            .font(Font::Caption)
                            .weight(FontWeight::Semibold)
                            .secondary(),
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
                    .font(Font::Body),

                    row((
                        button("Изменить")
                            .action(start_edit),
                        button("Удалить")
                            .action(del),
                    ))
                    .spacing(12.0),
                ))
                .spacing(6.0)
                .align(HAlign::Leading)
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
                            .frame(14.0, 14.0)
                            .tint(move || Color::hex(state.accent_color.get())),
                        label("ЗАМЕТКА К УРОКУ")
                            .font(Font::Caption)
                            .weight(FontWeight::Semibold)
                            .secondary(),
                        spacer().grow(),
                        label("видна только вам")
                            .font(Font::Caption2)
                            .secondary(),
                    ))
                    .spacing(6.0)
                    .align(VAlign::Center),

                    button("+ Добавить заметку к уроку")
                        .action(add),
                ))
                .spacing(6.0)
                .align(HAlign::Leading)
            },
        ),
    ))
    .spacing(6.0)
    .align(HAlign::Leading),
))
.spacing(0.0)
.align(HAlign::Leading)
.padding(Insets { top: 8.0, leading: 20.0, bottom: 24.0, trailing: 20.0 })
}
