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

pub fn render(slot: ItemSlot<LessonSlot, u32>, date_str: String) -> impl Piece {
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
    let s_attach = slot;
    let s_subst = slot;
    let s_has_note = slot;
    let s_badge_tap = slot;
    let s_lp = slot;

    let d_badge = date_str.clone();
    let d_lp = date_str;

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
                    // Accent badges (attachments, substitution, note)
                    row((
                        // Attachment badge
                        when(
                            move || s_attach.with(|s| s.has_attachments_or_links),
                            move || {
                                vector(crate::res::vectors::attachment)
                                    .frame(13.0, 13.0)
                                    .tint(move || Color::hex(state.accent_color.get()))
                            },
                        ),
                        // Substitution badge
                        when(
                            move || s_subst.with(|s| s.substitution.is_some()),
                            move || {
                                vector(crate::res::vectors::substitution)
                                    .frame(13.0, 13.0)
                                    .tint(move || Color::hex(state.accent_color.get()))
                            },
                        ),
                        // Note badge
                        when(
                            move || {
                                let key = s_has_note.with(|s| lesson_key(s));
                                state.lesson_notes.with(|m| m.contains_key(&key))
                            },
                            move || {
                                vector(crate::res::vectors::kind_note)
                                    .frame(13.0, 13.0)
                                    .tint(move || Color::hex(state.accent_color.get()))
                            },
                        ),
                    ))
                    .spacing(5.0)
                    .align(VAlign::Center)
                    .on_tap(move || {
                        let d_str = d_badge.clone();
                        s_badge_tap.with(|s| {
                            let teacher = find_teacher_for_lesson(state, s);
                            let end_time = state.bell_times.get().iter().find(|b| b.number == s.number).map(|b| b.end_time.clone());
                            let target = crate::widgets::lesson_peek::LessonDetailTarget {
                                key: lesson_key(s),
                                lesson_uuid: s.lesson_uuid.clone(),
                                subject_title: s.subject_title.clone(),
                                lesson_number: s.number,
                                date_str: Some(d_str),
                                start_time: s.start_time.clone(),
                                end_time,
                                topic: s.topic.clone(),
                                homework: s.homework.clone(),
                                teacher,
                                comment: s.lesson_mark.as_ref().and_then(|m| m.comment.clone()),
                                mark: mark_of(s).map(|m| m.to_string()),
                                has_attachments: s.has_attachments_or_links,
                                is_substitution: s.substitution.is_some(),
                                start_editing_note: false,
                            };
                            crate::widgets::lesson_peek::open_peek(state, target);
                        });
                    }),
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
        // Tap to open lesson detail modal
        .on_tap({
            let d_str = d_lp.clone();
            let slot = s_lp;
            move || {
                let d = d_str.clone();
                slot.with(|s| {
                    let teacher = find_teacher_for_lesson(state, s);
                    let end_time = state.bell_times.get().iter().find(|b| b.number == s.number).map(|b| b.end_time.clone());
                    let target = crate::widgets::lesson_peek::LessonDetailTarget {
                        key: lesson_key(s),
                        lesson_uuid: s.lesson_uuid.clone(),
                        subject_title: s.subject_title.clone(),
                        lesson_number: s.number,
                        date_str: Some(d),
                        start_time: s.start_time.clone(),
                        end_time,
                        topic: s.topic.clone(),
                        homework: s.homework.clone(),
                        teacher,
                        comment: s.lesson_mark.as_ref().and_then(|m| m.comment.clone()),
                        mark: mark_of(s).map(|m| m.to_string()),
                        has_attachments: s.has_attachments_or_links,
                        is_substitution: s.substitution.is_some(),
                        start_editing_note: false,
                    };
                    crate::widgets::lesson_peek::open_peek(state, target);
                });
            }
        })
        // Telegram-style long press peek
        .on_long_press(move |_p| {
            let d_str = d_lp.clone();
            s_lp.with(|s| {
                let teacher = find_teacher_for_lesson(state, s);
                let end_time = state.bell_times.get().iter().find(|b| b.number == s.number).map(|b| b.end_time.clone());
                let target = crate::widgets::lesson_peek::LessonDetailTarget {
                    key: lesson_key(s),
                    lesson_uuid: s.lesson_uuid.clone(),
                    subject_title: s.subject_title.clone(),
                    lesson_number: s.number,
                    date_str: Some(d_str),
                    start_time: s.start_time.clone(),
                    end_time,
                    topic: s.topic.clone(),
                    homework: s.homework.clone(),
                    teacher,
                    comment: s.lesson_mark.as_ref().and_then(|m| m.comment.clone()),
                    mark: mark_of(s).map(|m| m.to_string()),
                    has_attachments: s.has_attachments_or_links,
                    is_substitution: s.substitution.is_some(),
                    start_editing_note: false,
                };
                crate::widgets::lesson_peek::open_peek(state, target);
            });
        }),

        divider().padding(Insets { top: 4.0, leading: 16.0, bottom: 0.0, trailing: 0.0 }),
    ))
    .spacing(0.0)
    .align(HAlign::Leading)
}
