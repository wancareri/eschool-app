use crate::core::colors;
use crate::core::network::models::*;
use crate::core::state::{self, ESchoolState};
use crate::res;
use day::prelude::*;

pub(crate) fn diary_page() -> impl Piece {
    let state = ESchoolState::ambient();

    scroll(column((
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

        when(
            move || state.is_authenticated.get(),
            move || week_navigator(state),
        ),

        when(
            move || state.is_authenticated.get() && !state.lessons_loading.get(),
            move || week_summary_card(state),
        ),

        when(
            move || state.is_authenticated.get() && !state.lessons_loading.get(),
            move || quarter_stats_card(state),
        ),

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
                spinner(),
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
        s2.reset_marks();
        s2.load_week(from_week);
    })
    .id(format!("q-{label}"))
}

// ── Week summary card ────────────────────────────────────────────────────

fn week_summary_card(state: ESchoolState) -> impl Piece {
    column((
        label("Неделя")
            .font(Font::Headline)
            .color(colors::PRIMARY)
            .padding(Insets { top: 16.0, leading: 20.0, bottom: 6.0, trailing: 20.0 }),

        row((
            stat_block("Уроков", move || {
                let lessons = state.lessons.get();
                lessons.iter().map(|d| d.slots.len()).sum::<usize>().to_string()
            }),
            stat_block("Оценок", move || {
                let lessons = state.lessons.get();
                lessons.iter()
                    .flat_map(|d| &d.slots)
                    .filter(|s| s.lesson_mark.is_some())
                    .count()
                    .to_string()
            }),
            stat_block("Ср. балл", move || {
                let lessons = state.lessons.get();
                let marks: Vec<f64> = lessons.iter()
                    .flat_map(|d| &d.slots)
                    .filter_map(|s| s.lesson_mark.as_ref())
                    .filter_map(|m| m.mark.as_ref())
                    .filter_map(|m| m.parse::<f64>().ok())
                    .collect();
                if marks.is_empty() { "—".into() }
                else { format!("{:.1}", marks.iter().sum::<f64>() / marks.len() as f64) }
            }),
        ))
        .spacing(8.0)
        .padding(Insets { top: 0.0, leading: 20.0, bottom: 12.0, trailing: 20.0 }),
    ))
    .spacing(0.0)
}

fn stat_block(title: &'static str, value_fn: impl Fn() -> String + 'static) -> impl Piece {
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

// ── Quarter stats card ───────────────────────────────────────────────────

fn quarter_stats_card(state: ESchoolState) -> impl Piece {
    column((
        label("Четверть")
            .font(Font::Headline)
            .color(colors::PRIMARY)
            .padding(Insets { top: 16.0, leading: 20.0, bottom: 6.0, trailing: 20.0 }),

        row((
            stat_block("Четверть", move || {
                let idx = state.current_week_index.get();
                let q = if idx < 9 { "I" } else if idx < 18 { "II" } else if idx < 27 { "III" } else { "IV" };
                format!("{} четверть", q)
            }),
            stat_block("Оценок", move || {
                state.all_marks.get().len().to_string()
            }),
            stat_block("Средний балл", move || {
                let marks = state.all_marks.get();
                if marks.is_empty() { "—".into() }
                else {
                    let avg: f64 = marks.iter().map(|(_, v)| v).sum::<f64>() / marks.len() as f64;
                    format!("{:.2}", avg)
                }
            }),
        ))
        .spacing(8.0)
        .padding(Insets { top: 0.0, leading: 20.0, bottom: 8.0, trailing: 20.0 }),

        // Per-subject breakdown
        label(move || {
            let marks = state.all_marks.get();
            if marks.is_empty() {
                return "Нет оценок за четверть".into();
            }
            let mut by_subject: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();
            for (subj, val) in &marks {
                by_subject.entry(subj.clone()).or_default().push(*val);
            }
            let mut subjects: Vec<_> = by_subject.into_iter().collect();
            subjects.sort_by(|a, b| a.0.cmp(&b.0));

            subjects.iter().map(|(subj, vals)| {
                let avg = vals.iter().sum::<f64>() / vals.len() as f64;
                let cnt = vals.len();
                format!("{} — {:.1}  ({})", subj, avg, cnt)
            }).collect::<Vec<_>>().join("\n")
        })
        .font(Font::Caption)
        .secondary()
        .padding(Insets { top: 0.0, leading: 20.0, bottom: 12.0, trailing: 20.0 }),
    ))
    .spacing(0.0)
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
