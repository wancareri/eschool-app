use crate::app::AppState;
use eschool_api::entities::*;
use crate::shared::utils;
use crate::widgets;
use crate::res;
use day::prelude::*;

pub fn render() -> impl Piece {
    let state = AppState::ambient();

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
                spinner(),
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

fn schedule_content(state: AppState) -> impl Piece {
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

fn bell_section(state: AppState) -> impl Piece {
    column((
        label("Звонки")
            .font(Font::Headline)
            .color(move || Color::hex(state.accent_color.get()))
            .padding(Insets { top: 16.0, leading: 20.0, bottom: 8.0, trailing: 20.0 }),
        each(
            items(
                move || state.bell_times.get(),
                |b: &BellTime| b.number,
            ),
            move |slot| {
                let num = slot.key();
                widgets::bell_row::render(state, num).any()
            },
        ),
    ))
    .spacing(0.0)
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}

fn timetable_section(state: AppState) -> impl Piece {
    column((
        label("Расписание уроков")
            .font(Font::Headline)
            .color(move || Color::hex(state.accent_color.get()))
            .padding(Insets { top: 16.0, leading: 20.0, bottom: 8.0, trailing: 20.0 }),
        each(
            items(
                move || state.timetable_days.get(),
                |d: &TimetableDay| d.day_of_week,
            ),
            move |slot| {
                let dow = slot.key();
                timetable_day_card(state, dow).any()
            },
        ),
    ))
    .spacing(0.0)
}

fn timetable_day_card(state: AppState, dow: u32) -> impl Piece {
    column((
        label(utils::weekday_name(dow))
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
