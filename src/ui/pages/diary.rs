use crate::core::colors;
use crate::core::network::models::*;
use crate::core::state::{self, ESchoolState};
use crate::native;
use crate::res;
use day::prelude::*;

/// Diary page — shows lessons for the current week grouped by day.
pub(crate) fn diary_page() -> impl Piece {
    let state = ESchoolState::ambient();

    column((
        native::header::render(
            res::str::diary_title().format(),
            "Расписание уроков",
        ),
        // Current week label
        label(move || {
            let w = state.current_week.get();
            if w.is_empty() { "Текущая неделя".into() } else { w }
        })
        .font(Font::Subheadline)
        .secondary()
        .padding(Insets { top: 0.0, leading: 16.0, bottom: 0.0, trailing: 16.0 }),
        // ── not logged in ──
        when(
            move || !state.is_authenticated.get(),
            || column((
                spacer(),
                label("Войдите в аккаунт для просмотра дневника")
                    .font(Font::Body)
                    .secondary()
                    .align(TextAlign::Center),
                spacer(),
            )).grow(),
        ),
        // ── loading ──
        when(
            move || state.is_authenticated.get() && state.lessons_loading.get(),
            || column((
                spacer(),
                label("Загрузка расписания…")
                    .font(Font::Body)
                    .secondary()
                    .align(TextAlign::Center),
                spacer(),
            )).grow(),
        ),
        // ── loaded ──
        when(
            move || state.is_authenticated.get() && !state.lessons_loading.get(),
            move || diary_list(state),
        ),
    ))
    .spacing(8.0)
    .grow()
}

// ── diary content ────────────────────────────────────────────────────────

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
        // Day header: "Понедельник, 07.09"
        label(move || {
            let lessons = state.lessons.get();
            lessons
                .iter()
                .find(|d| d.date == date)
                .map(|d| state::format_date_header(d.day_of_week, d.date))
                .unwrap_or_default()
        })
        .font(Font::Title3)
        .padding(Insets { top: 8.0, leading: 16.0, bottom: 4.0, trailing: 16.0 }),
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
    .spacing(2.0)
    .padding(Insets { top: 4.0, leading: 0.0, bottom: 12.0, trailing: 0.0 })
}

fn lesson_row(state: ESchoolState, date: u64, number: u32) -> impl Piece {
    row((
        // Lesson number badge
        label(number.to_string())
            .font(Font::Caption)
            .color(colors::WHITE)
            .align(TextAlign::Center),
        // Subject + time column
        column((
            label(move || find_field(state, date, number, |s| s.subject_title.clone()))
                .font(Font::Body),
            row((
                label(move || find_field(state, date, number, |s| {
                    // "08:30" — trim the seconds portion if present
                    let t = &s.start_time;
                    t.get(..5).unwrap_or(t).to_string()
                }))
                .font(Font::Caption)
                .secondary(),
                when(
                    move || find_field_bool(state, date, number, |s| s.homework.is_some()),
                    || label(" · ДЗ").font(Font::Caption).color(colors::ACCENT),
                ),
            )).spacing(4.0),
        ))
        .spacing(2.0)
        .align(HAlign::Leading)
        .grow(),
        // Grade
        label(move || {
            find_field(state, date, number, |s| {
                s.lesson_mark.clone().unwrap_or_else(|| "—".into())
            })
        })
        .font(Font::Title2)
        .color(move || {
            let mark = find_field(state, date, number, |s| {
                s.lesson_mark.clone().unwrap_or_default()
            });
            state::grade_color(&mark)
        }),
    ))
    .spacing(12.0)
    .padding(Insets { top: 6.0, leading: 16.0, bottom: 6.0, trailing: 16.0 })
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
