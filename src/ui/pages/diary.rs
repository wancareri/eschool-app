use crate::core::colors;
use crate::core::network::models::*;
use crate::core::state::{self, ESchoolState};
use crate::res;
use day::prelude::*;

pub(crate) fn diary_page() -> impl Piece {
    let state = ESchoolState::ambient();

    scroll(column((
        // ── Header ──────────────────────────────────────────────────
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
        .padding(Insets { top: 16.0, leading: 20.0, bottom: 4.0, trailing: 20.0 }),

        // ── Week navigator ──────────────────────────────────────────
        when(
            move || state.is_authenticated.get(),
            move || week_navigator(state),
        ),

        // ── Today summary card ─────────────────────────────────────
        when(
            move || state.is_authenticated.get() && !state.lessons_loading.get(),
            move || today_summary_card(state),
        ),

        // ── States ──────────────────────────────────────────────────
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

// ── Week navigator ───────────────────────────────────────────────────────

fn week_navigator(state: ESchoolState) -> impl Piece {
    column((
        row((
            button("<").action(move || {
                let idx = state.current_week_index.get();
                if idx > 0 { state.load_week(idx - 1); }
            }).id("wk-prev"),
            label(move || state.current_week.get())
                .font(Font::Headline)
                .grow(),
            button(">").action(move || {
                let idx = state.current_week_index.get();
                let total = state.all_weeks.get().len() as i32;
                if idx + 1 < total { state.load_week(idx + 1); }
            }).id("wk-next"),
        ))
        .spacing(12.0)
        .padding(Insets { top: 0.0, leading: 16.0, bottom: 6.0, trailing: 16.0 }),

        // Quarter tabs: I, II, III, IV
        row((
            quarter_tab(state, "I", 0),
            quarter_tab(state, "II", 9),
            quarter_tab(state, "III", 18),
            quarter_tab(state, "IV", 27),
        ))
        .spacing(6.0)
        .padding(Insets { top: 0.0, leading: 16.0, bottom: 8.0, trailing: 16.0 }),
    ))
    .spacing(4.0)
}

fn quarter_tab(state: ESchoolState, label: &'static str, from_week: i32) -> impl Piece {
    let s1 = state;
    let s2 = state;
    let lbl = label.to_string();
    button(move || {
        let idx = s1.current_week_index.get();
        let is_active = idx >= from_week && idx < from_week + 9;
        if is_active { format!("[{}]", lbl) } else { lbl.clone() }
    })
    .action(move || {
        s2.load_week(from_week);
    })
    .id(format!("q-{label}"))
}

// ── Today summary card ───────────────────────────────────────────────────

fn today_summary_card(state: ESchoolState) -> impl Piece {
    column((
        label("Сегодня")
            .font(Font::Headline)
            .color(colors::PRIMARY)
            .padding(Insets { top: 16.0, leading: 20.0, bottom: 6.0, trailing: 20.0 }),

        row((
            stat_block(state, "Уроков", move || {
                count_today(state, |s| s.slots.len())
            }),
            stat_block(state, "Оценок", move || {
                count_today(state, |d| d.slots.iter().filter(|s| s.lesson_mark.is_some()).count())
            }),
            stat_block(state, "Ср. балл", move || {
                avg_today(state)
            }),
        ))
        .spacing(8.0)
        .padding(Insets { top: 0.0, leading: 20.0, bottom: 8.0, trailing: 20.0 }),
    ))
    .spacing(0.0)
}

fn stat_block(_state: ESchoolState, title: &'static str, value_fn: impl Fn() -> String + 'static) -> impl Piece {
    column((
        label(move || value_fn())
            .font(Font::Title2)
            .color(colors::PRIMARY)
            .align(TextAlign::Center),
        label(title)
            .font(Font::Caption)
            .secondary()
            .align(TextAlign::Center),
    ))
    .spacing(2.0)
    .padding(Insets { top: 8.0, leading: 8.0, bottom: 8.0, trailing: 8.0 })
    .grow()
}

fn count_today(state: ESchoolState, f: impl Fn(&DaySchedule) -> usize) -> String {
    let lessons = state.lessons.get();
    let today = today_date();
    lessons.iter()
        .find(|d| d.date == today)
        .map(|d| f(d))
        .unwrap_or(0)
        .to_string()
}

fn avg_today(state: ESchoolState) -> String {
    let lessons = state.lessons.get();
    let today = today_date();
    lessons.iter()
        .find(|d| d.date == today)
        .and_then(|d| {
            let marks: Vec<f64> = d.slots.iter()
                .filter_map(|s| s.lesson_mark.as_ref())
                .filter_map(|m| m.mark.as_ref())
                .filter_map(|m| m.parse::<f64>().ok())
                .collect();
            if marks.is_empty() { None }
            else { Some(format!("{:.1}", marks.iter().sum::<f64>() / marks.len() as f64)) }
        })
        .unwrap_or_else(|| "—".into())
}

fn today_date() -> u64 {
    let today = day_piece_datetime::DayDate::today();
    let d = day_piece_datetime::DayDate::new(today.year, today.month, today.day).unwrap();
    (d.to_epoch_days() as u64) * 86_400_000
}

// ── Diary list ───────────────────────────────────────────────────────────

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
            label(number.to_string())
                .font(Font::Caption)
                .color(colors::WHITE)
                .align(TextAlign::Center),
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
