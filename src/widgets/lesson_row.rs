use eschool_api::entities::*;
use crate::app::AppState;
use crate::shared::{colors, utils};
use day::prelude::*;

const RAIL_WIDTH: f64 = 40.0;
const RAIL_GAP: f64 = 10.0;

fn mark_of(s: &LessonSlot) -> Option<&str> {
    s.lesson_mark
        .as_ref()
        .and_then(|m| m.mark.as_deref())
        .filter(|m| !m.trim().is_empty())
}

fn lesson_key(s: &LessonSlot) -> String {
    if !s.lesson_uuid.is_empty() {
        s.lesson_uuid.clone()
    } else {
        format!("{}_{}_{}", s.subject_title, s.number, s.start_time)
    }
}

fn find_teacher_for_lesson(state: AppState, s: &LessonSlot) -> Option<String> {
    if !s.teacher_id.is_empty() {
        if let Some(st) = state.subjects_teachers.get().iter().find(|t| t.teacher_id == s.teacher_id) {
            return Some(st.teacher.clone());
        }
    }
    state.subjects_teachers.get().iter().find(|t| t.subject_title == s.subject_title).map(|t| t.teacher.clone())
}

pub fn render(slot: ItemSlot<LessonSlot, u32>) -> impl Piece {
    let state = AppState::ambient();
    let s_num = slot;
    let s_time = slot;
    let s_title = slot;
    let s_topic = slot;
    let s_mark = slot;
    let s_has_comment = slot;
    let s_comment = slot;
    let s_has_hw = slot;
    let s_hw = slot;
    let s_has_msg = slot;
    let s_msg = slot;
    let s_has_note = slot;
    let s_note_txt = slot;
    let s_note_tap = slot;
    let s_lp = slot;

    column((
        row((
            // Lesson number & start time
            column((
                label(move || s_num.with(|s| s.number.to_string()))
                    .font(Font::Title3)
                    .bold()
                    .tabular(),
                label(move || s_time.with(|s| utils::strip_seconds(&s.start_time)))
                    .font(Font::Caption)
                    .secondary()
                    .tabular(),
            ))
            .width(RAIL_WIDTH)
            .spacing(3.0)
            .align(HAlign::Center),

            // Subject, Topic, Homework, Note hint
            column((
                row((
                    label(move || s_title.with(|s| s.subject_title.clone()))
                        .font(Font::Body)
                        .weight(FontWeight::Semibold)
                        .grow(),
                    when(
                        move || s_has_note.with(|s| {
                            let key = lesson_key(s);
                            crate::widgets::lesson_note::get_note(state, &key).is_some()
                        }),
                        move || {
                            circle()
                                .fill(move || Color::hex(state.accent_color.get()))
                                .frame(6.0, 6.0)
                        },
                    ),
                ))
                .spacing(6.0)
                .align(VAlign::Center),

                when(
                    move || s_topic.with(|s| {
                        s.topic.as_deref().map(|t| !t.is_empty()).unwrap_or(false)
                    }),
                    move || label(move || s_topic.with(|s| s.topic.clone().unwrap_or_default()))
                        .font(Font::Caption)
                        .secondary(),
                ),
                when(
                    move || s_has_comment.with(|s| {
                        s.lesson_mark
                            .as_ref()
                            .and_then(|m| m.comment.as_ref())
                            .map(|c| !c.is_empty())
                            .unwrap_or(false)
                    }),
                    move || label(move || {
                        s_comment.with(|s| {
                            s.lesson_mark.as_ref().and_then(|m| m.comment.clone()).unwrap_or_default()
                        })
                    })
                    .font(Font::Caption)
                    .italic()
                    .secondary()
                    .padding(Insets { top: 1.0, leading: 0.0, bottom: 1.0, trailing: 0.0 }),
                ),
                when(
                    move || s_has_hw.with(|s| {
                        s.homework.as_ref().map(|h| !h.is_empty()).unwrap_or(false)
                    }),
                    move || label(move || s_hw.with(|s| s.homework.clone().unwrap_or_default()))
                    .font(Font::Caption)
                    .weight(FontWeight::Semibold)
                    .color(colors::ACCENT)
                    .padding(Insets { top: 2.0, leading: 0.0, bottom: 2.0, trailing: 0.0 }),
                ),
                when(
                    move || s_has_msg.with(|s| {
                        s.message.as_ref().map(|m| !m.is_empty()).unwrap_or(false)
                    }),
                    move || label(move || {
                        s_msg.with(|s| {
                            format!("ℹ {}", s.message.as_ref().map(|m| m.as_str()).unwrap_or(""))
                        })
                    })
                    .font(Font::Caption)
                    .secondary()
                    .padding(Insets { top: 1.0, leading: 0.0, bottom: 1.0, trailing: 0.0 }),
                ),
                when(
                    move || s_has_note.with(|s| {
                        let key = lesson_key(s);
                        crate::widgets::lesson_note::get_note(state, &key).is_some()
                    }),
                    move || {
                        let s_txt = s_note_txt;
                        let s_tap = s_note_tap;
                        label(move || {
                            s_txt.with(|s| {
                                let key = lesson_key(s);
                                format!("📝 {}", crate::widgets::lesson_note::get_note(state, &key).unwrap_or_default())
                            })
                        })
                        .font(Font::Caption)
                        .color(move || Color::hex(state.accent_color.get()))
                        .padding(Insets { top: 2.0, leading: 0.0, bottom: 2.0, trailing: 0.0 })
                        .on_tap(move || {
                            s_tap.with(|s| {
                                let teacher = find_teacher_for_lesson(state, s);
                                let target = crate::widgets::lesson_peek::LessonDetailTarget {
                                    key: lesson_key(s),
                                    subject_title: s.subject_title.clone(),
                                    lesson_number: s.number,
                                    start_time: s.start_time.clone(),
                                    end_time: None,
                                    topic: s.topic.clone(),
                                    homework: s.homework.clone(),
                                    teacher,
                                    comment: s.lesson_mark.as_ref().and_then(|m| m.comment.clone()),
                                    mark: mark_of(s).map(|m| m.to_string()),
                                };
                                crate::widgets::lesson_peek::open_peek(state, target);
                            });
                        })
                    },
                ),
            ))
            .spacing(1.0)
            .align(HAlign::Leading)
            .grow(),

            // Grade
            label(move || {
                s_mark.with(|s| mark_of(s).map(|m| m.to_string()).unwrap_or_else(|| "—".into()))
            })
            .font(Font::Title3)
            .bold()
            .tabular()
            .color(move || {
                s_mark.with(|s| match mark_of(s) {
                    Some(m) => utils::grade_color(m),
                    None => colors::GRAY_400,
                })
            })
            .align(TextAlign::Center)
            .frame(36.0, 36.0),
        ))
        .spacing(RAIL_GAP)
        .align(VAlign::Center)
        .grow()
        .padding(Insets { top: 8.0, leading: 16.0, bottom: 2.0, trailing: 16.0 })
        // Telegram-style long press peek
        .on_long_press(move |_p| {
            s_lp.with(|s| {
                let teacher = find_teacher_for_lesson(state, s);
                let target = crate::widgets::lesson_peek::LessonDetailTarget {
                    key: lesson_key(s),
                    subject_title: s.subject_title.clone(),
                    lesson_number: s.number,
                    start_time: s.start_time.clone(),
                    end_time: None,
                    topic: s.topic.clone(),
                    homework: s.homework.clone(),
                    teacher,
                    comment: s.lesson_mark.as_ref().and_then(|m| m.comment.clone()),
                    mark: mark_of(s).map(|m| m.to_string()),
                };
                crate::widgets::lesson_peek::open_peek(state, target);
            });
        }),

        divider().padding(Insets { top: 4.0, leading: 16.0, bottom: 0.0, trailing: 0.0 }),
    ))
    .spacing(0.0)
    .align(HAlign::Leading)
}
