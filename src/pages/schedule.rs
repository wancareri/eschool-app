use crate::app::AppState;
use eschool_api::entities::*;
use crate::shared::utils;
use crate::widgets;
use crate::res;
use day::prelude::*;

const PAD: f64 = 20.0;

pub fn render() -> impl Piece {
    let state = AppState::ambient();

    scroll(column((
        column((
            label(move || res::str::schedule_title().format())
                .font(Font::LargeTitle)
                .align(TextAlign::Center),
            label("Расписание звонков и уроков")
                .font(Font::Subheadline)
                .secondary()
                .align(TextAlign::Center),
        ))
        .spacing(6.0)
        .padding(Insets { top: 16.0, leading: PAD, bottom: 12.0, trailing: PAD }),

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
                label("  Загрузка…").font(Font::Caption).secondary(),
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
            .font(Font::Title3)
            .color(move || Color::hex(state.accent_color.get()))
            .padding(Insets { top: 8.0, leading: PAD, bottom: 8.0, trailing: PAD }),
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
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 20.0, trailing: 0.0 })
}

fn timetable_section(state: AppState) -> impl Piece {
    column((
        label("Расписание уроков")
            .font(Font::Title3)
            .color(move || Color::hex(state.accent_color.get()))
            .padding(Insets { top: 0.0, leading: PAD, bottom: 8.0, trailing: PAD }),
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
        label(move || {
            let name = utils::weekday_name(dow);
            let count = state.timetable_days.get()
                .iter()
                .find(|d| d.day_of_week == dow)
                .map(|d| d.timetable_slots.len())
                .unwrap_or(0);
            if count > 0 { format!("{}  ·  {} ур.", name, count) } else { name.to_string() }
        })
        .font(Font::Headline)
        .color(move || Color::hex(state.accent_color.get()))
        .padding(Insets { top: 12.0, leading: PAD, bottom: 6.0, trailing: PAD }),
        label(move || {
            state.timetable_days.get()
                .iter()
                .find(|d| d.day_of_week == dow)
                .map(|d| {
                    d.timetable_slots.iter().map(|ts| {
                        let lesson_num = ts.time_of_bells.number;
                        let start = &ts.time_of_bells.start_time;
                        let end = &ts.time_of_bells.end_time;
                        let subjects: Vec<String> = ts.slots.iter()
                            .filter_map(|s| s.summary.clone())
                            .collect();
                        let subj = if subjects.is_empty() {
                            "—".to_string()
                        } else {
                            subjects.join(" / ")
                        };
                        format!("{}. {}–{}  {}", lesson_num, start, end, subj)
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
                })
                .unwrap_or_default()
        })
        .font(Font::Body)
        .secondary()
        .padding(Insets { top: 0.0, leading: PAD, bottom: 8.0, trailing: PAD }),
        divider().padding(Insets { top: 0.0, leading: PAD, bottom: 0.0, trailing: PAD }),
    ))
    .spacing(0.0)
}
