use crate::app::AppState;
use crate::features;
use crate::widgets;
use crate::shared::utils;
use crate::res;
use day::prelude::*;
use day_piece_pullrefresh::pull_to_refresh;

const PAD: f64 = 16.0;

pub fn render() -> impl Piece {
    let state = AppState::ambient();
    let refreshing = Signal::new(false);

    pull_to_refresh(refreshing, scroll(column((
            widgets::conn_status::render()
                .padding(Insets { top: 8.0, leading: 16.0, bottom: 0.0, trailing: 16.0 }),
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
        when(
            move || !state.subjects_teachers.get().is_empty(),
            move || teachers_section(state),
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
    column((
        row((
            label(move || slot.with(|r| r.num.to_string()))
                .font(Font::Body)
                .secondary()
                .frame(28.0, 20.0),
            label(move || slot.with(|r| r.time.clone()))
                .font(Font::Body)
                .secondary()
                .frame(80.0, 20.0),
            label(move || slot.with(|r| r.subject.clone()))
                .font(Font::Body)
                .grow(),
        ))
        .spacing(6.0)
        .align(VAlign::Center)
        .padding(Insets { top: 6.0, leading: PAD, bottom: 2.0, trailing: PAD }),
        when(
            move || slot.with(|r| !r.teacher.is_empty()),
            move || label(move || format!("└ {}", slot.with(|r| r.teacher.clone())))
                .font(Font::Caption)
                .color(move || Color::hex(state.accent_color.get()))
                .padding(Insets { top: 0.0, leading: 60.0, bottom: 6.0, trailing: PAD }),
        ),
    ))
    .spacing(0.0)
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
                if dow >= 6 { format!("{}  ·  Выходной", name) }
                else { format!("{}  ·  Нет уроков", name) }
            } else {
                let count = day_data()
                    .map(|d| d.timetable_slots.iter().filter(|ts| !ts.slots.is_empty()).count())
                    .unwrap_or(0);
                format!("{}  ·  {} ур.", name, count)
            }
        })
        .font(Font::Headline)
        .color(move || Color::hex(state.accent_color.get()))
        .padding(Insets { top: 14.0, leading: PAD, bottom: 6.0, trailing: PAD }),

        when(
            move || has_lessons(),
            move || row((
                label("№")
                    .font(Font::Caption)
                    .secondary()
                    .frame(28.0, 16.0),
                label("Время")
                    .font(Font::Caption)
                    .secondary()
                    .frame(80.0, 16.0),
                label("Предмет")
                    .font(Font::Caption)
                    .secondary()
                    .grow(),
            ))
            .spacing(6.0)
            .padding(Insets { top: 0.0, leading: PAD, bottom: 4.0, trailing: PAD }),
        ),

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

fn teachers_section(state: AppState) -> impl Piece {
    column((
        label("Преподаватели")
            .font(Font::Title3)
            .color(move || Color::hex(state.accent_color.get()))
            .padding(Insets { top: 24.0, leading: PAD, bottom: 4.0, trailing: PAD }),

        // Header
        row((
            label("Предмет")
                .font(Font::Caption)
                .secondary()
                .grow(),
            label("Учитель")
                .font(Font::Caption)
                .secondary()
                .grow(),
            label("Уровень")
                .font(Font::Caption)
                .secondary()
                .frame(70.0, 16.0),
        ))
        .spacing(6.0)
        .padding(Insets { top: 4.0, leading: PAD, bottom: 4.0, trailing: PAD }),

        divider().padding(Insets { top: 0.0, leading: PAD, bottom: 0.0, trailing: PAD }),

        each(
            items(
                move || state.subjects_teachers.get(),
                |st| format!("{}:{}", st.teacher_id, st.id),
            ),
            move |slot| {
                let s_st = slot;
                column((
                    row((
                        label(move || s_st.with(|st| st.subject_title.clone()))
                            .font(Font::Body)
                            .grow(),
                        label(move || s_st.with(|st| st.teacher.clone()))
                            .font(Font::Body)
                            .secondary()
                            .grow(),
                        label(move || s_st.with(|st| st.level_of_study.clone()))
                            .font(Font::Caption)
                            .color(move || Color::hex(state.accent_color.get()))
                            .frame(70.0, 16.0),
                    ))
                    .spacing(6.0)
                    .padding(Insets { top: 6.0, leading: PAD, bottom: 6.0, trailing: PAD }),
                    divider().padding(Insets { top: 0.0, leading: PAD, bottom: 0.0, trailing: PAD }),
                ))
                .spacing(0.0)
                .any()
            },
        ),
    ))
    .spacing(0.0)
}
