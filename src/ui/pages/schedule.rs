use crate::core::colors;
use crate::core::network::models::*;
use crate::core::state::{self, ESchoolState};
use crate::res;
use day::prelude::*;

pub(crate) fn schedule_page() -> impl Piece {
    let state = ESchoolState::ambient();

    scroll(column((
        column((
            label(move || res::str::schedule_title().format())
                .font(Font::LargeTitle),
            label("Расписание звонков и уроков")
                .font(Font::Subheadline)
                .secondary(),
        ))
        .spacing(6.0)
        .padding(Insets { top: 16.0, leading: 20.0, bottom: 8.0, trailing: 20.0 }),
        when(
            move || !state.is_authenticated.get(),
            || column((
                spacer(),
                label("Войдите для просмотра расписания")
                    .font(Font::Body).secondary().align(TextAlign::Center),
                spacer(),
            )).grow(),
        ),
        when(
            move || state.is_authenticated.get() && state.schedule_loading.get(),
            || column((
                spacer(),
                label("Загрузка расписания…")
                    .font(Font::Body).secondary().align(TextAlign::Center),
                spacer(),
            )).grow(),
        ),
        when(
            move || state.is_authenticated.get() && !state.schedule_loading.get(),
            move || schedule_content(state),
        ),
    ))
    .spacing(0.0)
    .grow())
    .grow()
}

fn schedule_content(state: ESchoolState) -> impl Piece {
    column((
        when(
            move || !state.bell_times.get().is_empty(),
            move || bell_section(state),
        ),
        when(
            move || !state.timetable_days.get().is_empty(),
            move || timetable_section(state),
        ),
    ))
    .spacing(0.0)
    .grow()
}

// ── bell schedule ────────────────────────────────────────────────────────

fn bell_section(state: ESchoolState) -> impl Piece {
    column((
        label("Звонки")
            .font(Font::Headline)
            .color(colors::PRIMARY)
            .padding(Insets { top: 16.0, leading: 20.0, bottom: 8.0, trailing: 20.0 }),
        each(
            items(
                move || state.bell_times.get(),
                |b: &BellTime| b.number,
            ),
            move |slot| {
                let num = slot.key();
                bell_row(state, num).any()
            },
        ),
    ))
    .spacing(0.0)
}

fn bell_row(state: ESchoolState, number: u32) -> impl Piece {
    let start = move || {
        state.bell_times.get()
            .iter()
            .find(|b| b.number == number)
            .map(|b| b.start_time.get(..5).unwrap_or(&b.start_time).to_string())
            .unwrap_or_default()
    };
    let end = move || {
        state.bell_times.get()
            .iter()
            .find(|b| b.number == number)
            .map(|b| b.end_time.get(..5).unwrap_or(&b.end_time).to_string())
            .unwrap_or_default()
    };
    row((
        label(number.to_string())
            .font(Font::Caption)
            .color(colors::PRIMARY)
            .align(TextAlign::Center),
        label(move || format!("{} — {}", start(), end()))
            .font(Font::Body),
    ))
    .spacing(16.0)
    .padding(Insets { top: 8.0, leading: 20.0, bottom: 8.0, trailing: 20.0 })
}

// ── timetable ────────────────────────────────────────────────────────────

fn timetable_section(state: ESchoolState) -> impl Piece {
    column((
        label("Расписание уроков")
            .font(Font::Headline)
            .color(colors::PRIMARY)
            .padding(Insets { top: 16.0, leading: 20.0, bottom: 8.0, trailing: 20.0 }),
        each(
            items(
                move || state.timetable_days.get(),
                |d: &TimetableDay| d.day_of_week,
            ),
            move |slot| {
                let dow = slot.key();
                timetable_day_row(state, dow).any()
            },
        ),
    ))
    .spacing(0.0)
}

fn timetable_day_row(state: ESchoolState, dow: u32) -> impl Piece {
    column((
        label(state::weekday_name(dow))
            .font(Font::Headline)
            .padding(Insets { top: 12.0, leading: 20.0, bottom: 4.0, trailing: 20.0 }),
        label(move || {
            state.timetable_days.get()
                .iter()
                .find(|d| d.day_of_week == dow)
                .map(|d| {
                    d.timetable_slots.iter().map(|ts| {
                        let lesson_num = ts.time_of_bells.number;
                        let subjects: Vec<String> = ts.slots.iter()
                            .filter_map(|s| s.summary.clone())
                            .collect();
                        let subj = if subjects.is_empty() {
                            "—".to_string()
                        } else {
                            subjects.join(" / ")
                        };
                        format!("{}. {}", lesson_num, subj)
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
                })
                .unwrap_or_default()
        })
        .font(Font::Body)
        .secondary()
        .padding(Insets { top: 0.0, leading: 20.0, bottom: 8.0, trailing: 20.0 }),
    ))
    .spacing(0.0)
}
