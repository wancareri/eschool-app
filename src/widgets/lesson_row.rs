use crate::app::AppState;
use eschool_api::entities::*;
use crate::shared::{colors, utils};
use day::prelude::*;

const NUM_WIDTH: f64 = 28.0;
const HW_LEFT: f64 = 16.0 + NUM_WIDTH + 12.0;

pub fn render(state: AppState, date: u64, number: u32) -> impl Piece {
    column((
        row((
            label(number.to_string())
                .font(Font::Caption)
                .color(colors::WHITE)
                .frame(NUM_WIDTH, 20.0),
            column((
                label(move || find_field(state, date, number, |s| s.subject_title.clone()))
                    .font(Font::Body),
                label(move || {
                    let time = find_field(state, date, number, |s| {
                        let t = &s.start_time;
                        t.get(..5).unwrap_or(t).to_string()
                    });
                    let topic = find_field(state, date, number, |s| {
                        s.topic.clone().unwrap_or_default()
                    });
                    if topic.is_empty() { time } else { format!("{time} · {topic}") }
                })
                .font(Font::Caption)
                .secondary(),
            ))
            .spacing(2.0)
            .align(HAlign::Leading)
            .grow(),
            label(move || {
                find_field(state, date, number, |s| {
                    s.lesson_mark.as_ref()
                        .and_then(|m| m.mark.clone())
                        .unwrap_or_else(|| "—".into())
                })
            })
            .font(Font::Title3)
            .color(move || {
                let mark = find_field(state, date, number, |s| {
                    s.lesson_mark.as_ref()
                        .and_then(|m| m.mark.clone())
                        .unwrap_or_default()
                });
                utils::grade_color(&mark)
            }),
        ))
        .spacing(12.0)
        .padding(Insets { top: 8.0, leading: 16.0, bottom: 0.0, trailing: 20.0 }),
        when(
            move || find_field_bool(state, date, number, |s| {
                s.lesson_mark.as_ref()
                    .and_then(|m| m.comment.as_ref())
                    .map_or(false, |c| !c.is_empty())
            }),
            move || {
                label(move || find_field(state, date, number, |s| {
                    s.lesson_mark.as_ref()
                        .and_then(|m| m.comment.clone())
                        .unwrap_or_default()
                }))
                .font(Font::Caption)
                .color(colors::SECONDARY)
                .padding(Insets { top: 2.0, leading: HW_LEFT, bottom: 2.0, trailing: 20.0 })
            },
        ),
        when(
            move || find_field_bool(state, date, number, |s| {
                s.homework.as_ref().map_or(false, |h| !h.is_empty())
            }),
            move || {
                label(move || find_field(state, date, number, |s| {
                    s.homework.clone().unwrap_or_default()
                }))
                .font(Font::Caption)
                .color(colors::ACCENT)
                .padding(Insets { top: 4.0, leading: HW_LEFT, bottom: 4.0, trailing: 20.0 })
            },
        ),
        when(
            move || find_field_bool(state, date, number, |s| {
                s.message.as_ref().map_or(false, |m| !m.is_empty())
            }),
            move || {
                label(move || {
                    let msg = find_field(state, date, number, |s| {
                        s.message.clone().unwrap_or_default()
                    });
                    format!("ℹ {msg}")
                })
                .font(Font::Caption)
                .color(colors::SECONDARY)
                .padding(Insets { top: 2.0, leading: HW_LEFT, bottom: 8.0, trailing: 20.0 })
            },
        ),
    ))
    .spacing(0.0)
}

fn find_lesson(lessons: &[DaySchedule], date: u64, number: u32) -> Option<&LessonSlot> {
    lessons
        .iter()
        .find(|d| d.date == date)
        .and_then(|d| d.slots.iter().find(|s| s.number == number))
}

fn find_field(
    state: AppState,
    date: u64,
    number: u32,
    f: impl Fn(&LessonSlot) -> String,
) -> String {
    let lessons = state.lessons.get();
    find_lesson(&lessons, date, number)
        .map(&f)
        .unwrap_or_default()
}

fn find_field_bool(
    state: AppState,
    date: u64,
    number: u32,
    f: impl Fn(&LessonSlot) -> bool,
) -> bool {
    let lessons = state.lessons.get();
    find_lesson(&lessons, date, number)
        .map(&f)
        .unwrap_or(false)
}
