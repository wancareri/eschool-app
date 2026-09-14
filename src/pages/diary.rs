use crate::app::AppState;
use crate::entities::*;
use crate::shared::{colors, utils};
use crate::widgets;
use crate::res;
use day::prelude::*;

pub fn render() -> impl Piece {
    let state = AppState::ambient();

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
            move || widgets::week_navigator::render(state),
        ),

        when(
            move || state.is_authenticated.get() && !state.lessons_loading.get(),
            move || widgets::week_summary::render(state),
        ),

        when(
            move || state.is_authenticated.get() && !state.lessons_loading.get(),
            move || widgets::quarter_stats::render(state),
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

fn diary_list(state: AppState) -> impl Piece {
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

fn day_card(state: AppState, date: u64) -> impl Piece {
    column((
        label(move || {
            let lessons = state.lessons.get();
            lessons
                .iter()
                .find(|d| d.date == date)
                .map(|d| utils::format_date_header(d.day_of_week, d.date))
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
                widgets::lesson_row::render(state, date, num).any()
            },
        ),
    ))
    .spacing(0.0)
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 8.0, trailing: 0.0 })
}
