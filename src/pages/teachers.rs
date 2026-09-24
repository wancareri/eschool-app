use crate::app::AppState;
use crate::features;
use crate::widgets;
use eschool_api::entities::*;
use crate::res;
use day::prelude::*;
use day_piece_pullrefresh::pull_to_refresh;

const PAD: f64 = 20.0;

pub fn render() -> impl Piece {
    let state = AppState::ambient();
    let refreshing = Signal::new(false);

    pull_to_refresh(refreshing, scroll(column((
            widgets::conn_status::render()
                .padding(Insets { top: 8.0, leading: 16.0, bottom: 0.0, trailing: 16.0 }),
            column((
                label(move || res::str::teachers_title().format())
                    .font(Font::LargeTitle)
                    .align(TextAlign::Center),
                label("Предметы и преподаватели")
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
                    label("Войдите для просмотра списка учителей")
                        .font(Font::Body).secondary().align(TextAlign::Center),
                    spacer(),
                )).grow(),
            ),
            when(
                move || state.is_authenticated.get() && state.teachers_loading.get(),
                move || column((
                    spacer(),
                    widgets::spinner::render(state, 11.0),
                    label("  Загрузка…").font(Font::Caption).secondary(),
                    spacer(),
                )).grow(),
            ),
            when(
                move || state.is_authenticated.get() && !state.teachers_loading.get(),
                move || teachers_list(state),
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

fn teachers_list(state: AppState) -> impl Piece {
    each(
        items(
            move || {
                let all = state.subjects_teachers.get();
                let mut seen = std::collections::HashSet::new();
                all.into_iter()
                    .filter(|s| seen.insert(format!("{}:{}", s.subject_title, s.teacher)))
                    .collect::<Vec<_>>()
            },
            |s: &SubjectWithTeacher| format!("{}:{}", s.teacher_id, s.id),
        ),
        move |slot| {
            let key = slot.key();
            teacher_card(state, key).any()
        },
    )
}

fn teacher_card(state: AppState, key: String) -> impl Piece {
    let k1 = key.clone();
    let k2 = key.clone();
    let k3 = key.clone();
    let k3b = key.clone();
    let k5 = key.clone();
    let k5b = key;

    column((
        // Subject name — accent colored
        label(move || {
            state.subjects_teachers.get()
                .iter()
                .find(|s| format!("{}:{}", s.teacher_id, s.id) == k1)
                .map(|s| s.subject_title.clone())
                .unwrap_or_default()
        })
        .font(Font::Headline)
        .color(move || Color::hex(state.accent_color.get()))
        .padding(Insets { top: 10.0, leading: PAD, bottom: 2.0, trailing: PAD }),
        // Teacher name
        label(move || {
            state.subjects_teachers.get()
                .iter()
                .find(|s| format!("{}:{}", s.teacher_id, s.id) == k2)
                .map(|s| s.teacher.clone())
                .unwrap_or_default()
        })
        .font(Font::Body).secondary()
        .padding(Insets { top: 0.0, leading: PAD, bottom: 2.0, trailing: PAD }),
        // Level + group info
        row((
            when(move || {
                state.subjects_teachers.get()
                    .iter()
                    .find(|s| format!("{}:{}", s.teacher_id, s.id) == k3)
                    .map(|s| !s.level_of_study.is_empty())
                    .unwrap_or(false)
            }, move || {
                let lk = k3b.clone();
                label(move || {
                    state.subjects_teachers.get()
                        .iter()
                        .find(|s| format!("{}:{}", s.teacher_id, s.id) == lk)
                        .map(|s| format!("Уровень: {}", s.level_of_study))
                        .unwrap_or_default()
                }).font(Font::Caption).color(move || Color::hex(state.accent_color.get()))
            }),
            when(move || {
                state.subjects_teachers.get()
                    .iter()
                    .find(|s| format!("{}:{}", s.teacher_id, s.id) == k5)
                    .and_then(|s| s.group_name.as_ref())
                    .map(|g| !g.is_empty())
                    .unwrap_or(false)
            }, move || {
                let gk = k5b.clone();
                label(move || {
                    state.subjects_teachers.get()
                        .iter()
                        .find(|s| format!("{}:{}", s.teacher_id, s.id) == gk)
                        .and_then(|s| s.group_name.clone())
                        .map(|g| format!("Группа: {}", g))
                        .unwrap_or_default()
                }).font(Font::Caption).secondary()
            }),
        ))
        .spacing(8.0)
        .padding(Insets { top: 0.0, leading: PAD, bottom: 10.0, trailing: PAD }),
        divider().padding(Insets { top: 0.0, leading: PAD, bottom: 0.0, trailing: PAD }),
    ))
    .spacing(0.0)
}
