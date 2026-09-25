use crate::app::AppState;
use crate::features;
use crate::widgets;
use crate::shared::utils;
use crate::shared::colors;
use crate::res;
use day::prelude::*;
use day_piece_pullrefresh::pull_to_refresh;

const PAD: f64 = 16.0;

pub fn render() -> impl Piece {
    let state = AppState::ambient();
    let refreshing = Signal::new(false);

    zstack((
        pull_to_refresh(refreshing, scroll(column((
                    column((
                        label(move || res::str::schedule_title().format())
                            .font(Font::LargeTitle)
                            .align(TextAlign::Center),
                        label("Расписание, оценки и преподаватели")
                            .font(Font::Subheadline)
                            .secondary()
                            .align(TextAlign::Center),
                    ))
                    .spacing(6.0)
                    .padding(Insets { top: 8.0, leading: PAD, bottom: 12.0, trailing: PAD }),

                    when(
                        move || !state.is_authenticated.get(),
                        || column((
                            spacer(),
                            label("Войдите для просмотра")
                                .font(Font::Body).secondary().align(TextAlign::Center),
                            spacer(),
                        )).grow(),
                    ),
                    when(
                        move || state.is_authenticated.get() && state.schedule_loading.get(),
                        move || column((
                            spacer(),
                            widgets::spinner::render(state, 11.0),
                            label("  Загрузка…").font(Font::Caption).secondary(),
                            spacer(),
                        )).align(HAlign::Center).grow(),
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

#[derive(Clone)]
struct LessonRow {
    key: String,
    num: u32,
    time: String,
    subject: String,
    teacher: String,
}

fn day_lesson_rows(state: AppState, dow: u32) -> Vec<LessonRow> {
    let days = state.timetable_days.get();
    let teachers = state.subjects_teachers.get();
    let Some(day) = days.into_iter().find(|d| d.day_of_week == dow) else {
        return Vec::new();
    };
    let mut rows = Vec::new();
    for (i, ts) in day.timetable_slots.iter().enumerate() {
        if ts.slots.is_empty() {
            continue;
        }
        let subjects: Vec<String> = ts.slots.iter().filter_map(|s| s.summary.clone()).collect();
        let subject = if subjects.is_empty() {
            "—".into()
        } else {
            subjects.join(" / ")
        };
        let teacher = teachers
            .iter()
            .find(|t| t.subject_title == subject)
            .filter(|t| !t.teacher.is_empty())
            .map(|t| t.teacher.clone())
            .unwrap_or_default();
        rows.push(LessonRow {
            key: format!("{dow}:{i}:{}", ts.time_of_bells.number),
            num: ts.time_of_bells.number,
            time: format!("{}–{}", ts.time_of_bells.start_time, ts.time_of_bells.end_time),
            subject,
            teacher,
        });
    }
    rows
}

fn lesson_table_row(state: AppState, slot: ItemSlot<LessonRow, String>) -> impl Piece {
    row((
        column((
            label(move || slot.with(|r| r.num.to_string()))
                .font(Font::Footnote)
                .weight(FontWeight::Semibold)
                .color(move || Color::hex(state.accent_color.get()))
                .align(TextAlign::Center)
                .frame(26.0, 16.0),
            label(move || slot.with(|r| r.time.clone()))
                .font(Font::Caption2)
                .secondary()
                .align(TextAlign::Center),
        ))
        .spacing(3.0)
        .align(HAlign::Center),
        column((
            label(move || slot.with(|r| r.subject.clone())).font(Font::Body),
            when(
                move || slot.with(|r| !r.teacher.is_empty()),
                move || label(move || slot.with(|r| r.teacher.clone()))
                    .font(Font::Caption)
                    .color(move || Color::hex(state.accent_color.get())),
            ),
        ))
        .spacing(3.0)
        .grow(),
    ))
    .spacing(12.0)
    .align(VAlign::Center)
    .padding(Insets { top: 10.0, leading: 12.0, bottom: 10.0, trailing: 12.0 })
    .background(Color::rgba(0.95, 0.95, 0.97, 1.0))
    .corner_radius(10.0)
    .padding(Insets { top: 4.0, leading: PAD, bottom: 4.0, trailing: PAD })
}

fn timetable_day_card(state: AppState, dow: u32) -> impl Piece {
    let day_data = move || {
        state.timetable_days.get().into_iter().find(|d| d.day_of_week == dow)
    };
    let has_lessons = move || {
        day_data()
            .map(|d| d.timetable_slots.iter().any(|ts| !ts.slots.is_empty()))
            .unwrap_or(false)
    };

    column((
        label(move || {
            let name = utils::weekday_name(dow);
            if !has_lessons() {
                name.to_string()
            } else {
                let count = day_data()
                    .map(|d| d.timetable_slots.iter().filter(|ts| !ts.slots.is_empty()).count())
                    .unwrap_or(0);
                format!("{}  ·  {} ур.", name, count)
            }
        })
        .font(Font::Headline)
        .color(move || {
            if has_lessons() {
                Color::hex(state.accent_color.get())
            } else {
                colors::SECONDARY
            }
        })
        .id(format!("day-{dow}"))
        .padding(Insets { top: 14.0, leading: PAD, bottom: 6.0, trailing: PAD }),

        when(
            move || has_lessons(),
            move || each(
                items(
                    move || day_lesson_rows(state, dow),
                    |r: &LessonRow| r.key.clone(),
                ),
                move |slot| lesson_table_row(state, slot).any(),
            ),
        ),

        when(
            move || !has_lessons(),
            move || label(move || {
                if dow >= 6 { "Выходной день" } else { "Нет уроков" }
            })
            .font(Font::Caption)
            .secondary()
            .padding(Insets { top: 0.0, leading: PAD, bottom: 0.0, trailing: PAD }),
        ),

        divider()
            .padding(Insets { top: 8.0, leading: PAD, bottom: 0.0, trailing: PAD }),
    ))
    .spacing(0.0)
}
