use crate::app::AppState;
use crate::features;
use crate::widgets;
use crate::shared::utils;
use crate::res;
use day::prelude::*;
use day_piece_pullrefresh::pull_to_refresh;

const PAD: f64 = 20.0;

pub fn render() -> impl Piece {
    let state = AppState::ambient();
    let refreshing = Signal::new(false);

    zstack((
        pull_to_refresh(refreshing, scroll(column((
            column((
                label(move || res::str::schedule_title().format())
                    .font(Font::LargeTitle)
                    .align(TextAlign::Center),
                label("Расписание уроков")
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
        .grow()))
        .on_refresh(move || {
            let state = AppState::ambient();
            let done = refreshing.setter();
            if state.is_authenticated.get() {
                features::diary::load_all(state);
            }
            done.set(false);
        })
        .grow(),

        // Sticky status indicator in top-left corner
        widgets::conn_status::render()
            .padding(Insets { top: 16.0, leading: 16.0, bottom: 0.0, trailing: 0.0 }),
    ))
    .align(Alignment::TopLeading)
    .grow()
}

fn schedule_content(state: AppState) -> impl Piece {
    column((
        when(
            move || !state.timetable_days.get().is_empty(),
            move || timetable_section(state),
        ),
        when(
            move || state.timetable_days.get().is_empty(),
            || label("Расписание не найдено")
                .font(Font::Body)
                .secondary()
                .align(TextAlign::Center)
                .padding(Insets { top: 60.0, leading: PAD, bottom: 60.0, trailing: PAD }),
        ),
    ))
    .spacing(0.0)
    .grow()
}

fn timetable_section(state: AppState) -> impl Piece {
    column((
        label("Расписание уроков")
            .font(Font::Title3)
            .color(move || Color::hex(state.accent_color.get()))
            .padding(Insets { top: 0.0, leading: PAD, bottom: 8.0, trailing: PAD }),
        each(
            items(
                move || (1..=7u32).collect::<Vec<_>>(),
                |&dow| dow,
            ),
            move |slot| {
                let dow = slot.get();
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
            let day_opt = state.timetable_days.get().into_iter().find(|d| d.day_of_week == dow);
            let count = day_opt.as_ref()
                .map(|d| d.timetable_slots.iter().filter(|ts| !ts.slots.is_empty()).count())
                .unwrap_or(0);
            if count > 0 {
                format!("{}  ·  {} ур.", name, count)
            } else if dow >= 6 {
                format!("{}  ·  Выходной", name)
            } else {
                format!("{}  ·  Нет уроков", name)
            }
        })
        .font(Font::Headline)
        .color(move || Color::hex(state.accent_color.get()))
        .padding(Insets { top: 12.0, leading: PAD, bottom: 6.0, trailing: PAD }),

        label(move || {
            let day_opt = state.timetable_days.get().into_iter().find(|d| d.day_of_week == dow);
            let active_slots: Vec<_> = day_opt
                .as_ref()
                .map(|d| {
                    d.timetable_slots.iter()
                        .filter(|ts| !ts.slots.is_empty())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            if active_slots.is_empty() {
                if dow >= 6 {
                    "Выходной день".to_string()
                } else {
                    "Нет уроков в расписании".to_string()
                }
            } else {
                active_slots.iter().map(|ts| {
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
            }
        })
        .font(Font::Body)
        .secondary()
        .padding(Insets { top: 0.0, leading: PAD, bottom: 8.0, trailing: PAD }),

        divider().padding(Insets { top: 0.0, leading: PAD, bottom: 0.0, trailing: PAD }),
    ))
    .spacing(0.0)
}
