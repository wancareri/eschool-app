use crate::core::colors;
use crate::core::network::models::*;
use crate::core::state::{self, ESchoolState};
use crate::res;
use day::prelude::*;

pub(crate) fn diary_page() -> impl Piece {
    let state = ESchoolState::ambient();

    scroll(column((
        // Header
        column((
            label(move || res::str::diary_title().format())
                .font(Font::LargeTitle),
            label(move || {
                let w = state.current_week.get();
                if w.is_empty() { "Текущая неделя".into() } else { w }
            })
            .font(Font::Subheadline)
            .secondary(),
        ))
        .spacing(6.0)
        .padding(Insets { top: 16.0, leading: 20.0, bottom: 8.0, trailing: 20.0 }),
        // States
        when(
            move || !state.is_authenticated.get(),
            || column((
                spacer(),
                label("Войдите для просмотра дневника")
                    .font(Font::Body).secondary().align(TextAlign::Center),
                spacer(),
            )).grow(),
        ),
        when(
            move || state.is_authenticated.get() && state.lessons_loading.get(),
            || column((
                spacer(),
                label("Загрузка расписания…")
                    .font(Font::Body).secondary().align(TextAlign::Center),
                spacer(),
            )).grow(),
        ),
        when(
            move || state.is_authenticated.get() && !state.lessons_loading.get(),
            move || diary_list(state),
        ),
    ))
    .spacing(0.0)
    .grow())
    .grow()
}

fn diary_list(state: ESchoolState) -> impl Piece {
    each(
        items(
            move || state.lessons.get(),
            |d: &DaySchedule| d.date,
        ),
        move |day_slot| {
            let date = day_slot.key();
            day_card(state, date).any()
        },
    )
}

fn day_card(state: ESchoolState, date: u64) -> impl Piece {
    column((
        // Day header
        label(move || {
            let lessons = state.lessons.get();
            lessons
                .iter()
                .find(|d| d.date == date)
                .map(|d| state::format_date_header(d.day_of_week, d.date))
                .unwrap_or_default()
        })
        .font(Font::Headline)
        .color(colors::PRIMARY)
        .padding(Insets { top: 16.0, leading: 20.0, bottom: 6.0, trailing: 20.0 }),
        // Lessons
        each(
            items(
                move || {
                    state.lessons.get()
                        .into_iter()
                        .find(|d| d.date == date)
                        .map(|d| d.slots)
                        .unwrap_or_default()
                },
                |s: &LessonSlot| s.number,
            ),
            move |slot| {
                let num = slot.key();
                lesson_row(state, date, num).any()
            },
        ),
    ))
    .spacing(0.0)
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 8.0, trailing: 0.0 })
}

fn lesson_row(state: ESchoolState, date: u64, number: u32) -> impl Piece {
    column((
        row((
            // Lesson number badge
            label(number.to_string())
                .font(Font::Caption)
                .color(colors::WHITE)
                .align(TextAlign::Center),
            // Subject + time
            column((
                label(move || find_field(state, date, number, |s| s.subject_title.clone()))
                    .font(Font::Body),
                label(move || find_field(state, date, number, |s| {
                    let t = &s.start_time;
                    t.get(..5).unwrap_or(t).to_string()
                }))
                .font(Font::Caption)
                .secondary(),
            ))
            .spacing(2.0)
            .align(HAlign::Leading)
            .grow(),
            // Grade
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
                state::grade_color(&mark)
            }),
        ))
        .spacing(12.0)
        .padding(Insets { top: 8.0, leading: 16.0, bottom: 0.0, trailing: 20.0 }),
        // Homework
        when(
            move || find_field_bool(state, date, number, |s| s.homework.is_some()),
            move || {
                row((
                    label(move || find_field(state, date, number, |s| {
                        s.homework.clone().unwrap_or_default()
                    }))
                    .font(Font::Caption)
                    .color(colors::ACCENT),
                ))
                .padding(Insets { top: 4.0, leading: 40.0, bottom: 4.0, trailing: 20.0 })
            },
        ),
    ))
    .spacing(0.0)
}

// ── helpers ──────────────────────────────────────────────────────────────

fn find_lesson(lessons: &[DaySchedule], date: u64, number: u32) -> Option<&LessonSlot> {
    lessons
        .iter()
        .find(|d| d.date == date)
        .and_then(|d| d.slots.iter().find(|s| s.number == number))
}

fn find_field(
    state: ESchoolState,
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
    state: ESchoolState,
    date: u64,
    number: u32,
    f: impl Fn(&LessonSlot) -> bool,
) -> bool {
    let lessons = state.lessons.get();
    find_lesson(&lessons, date, number)
        .map(&f)
        .unwrap_or(false)
}
