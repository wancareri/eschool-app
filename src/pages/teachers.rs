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

    zstack((
        pull_to_refresh(refreshing, scroll(column((
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
        .grow(),

        widgets::conn_status::page_overlay(),
    ))
    .align(Alignment::TopLeading)
    .grow()
}

fn teachers_list(state: AppState) -> impl Piece {
    let items_sig = move || {
        let all = state.subjects_teachers.get();
        let mut seen = std::collections::HashSet::new();
        all.into_iter()
            .filter(|s| seen.insert(format!("{}:{}", s.subject_title, s.teacher)))
            .collect::<Vec<_>>()
    };

    column((
        when(
            move || items_sig().is_empty(),
            || column((
                spacer().height(40.0),
                label("👨‍🏫")
                    .font(Font::System(48.0))
                    .align(TextAlign::Center),
                spacer().height(12.0),
                label("Список учителей пуст")
                    .font(Font::Headline)
                    .align(TextAlign::Center),
                label("Данные появятся автоматически после синхронизации с электронным журналом")
                    .font(Font::Subheadline)
                    .secondary()
                    .align(TextAlign::Center)
                    .padding(Insets { top: 4.0, leading: 32.0, bottom: 20.0, trailing: 32.0 }),
            ))
            .grow(),
        ),

        when(
            move || !items_sig().is_empty(),
            move || {
                column((
                    // Count indicator
                    label(move || {
                        let count = items_sig().len();
                        format!("Всего предметов: {}", count)
                    })
                    .font(Font::Caption)
                    .secondary()
                    .padding(Insets { top: 0.0, leading: 20.0, bottom: 8.0, trailing: 20.0 }),

                    each(
                        items(
                            items_sig,
                            |s: &SubjectWithTeacher| format!("{}:{}", s.teacher_id, s.id),
                        ),
                        move |slot| {
                            let key = slot.key();
                            teacher_card(state, key).any()
                        },
                    ),
                    spacer().height(24.0),
                ))
            },
        ),
    ))
    .spacing(0.0)
    .grow()
}

fn teacher_card(state: AppState, key: String) -> impl Piece {
    let k1 = key.clone();
    let k2 = key.clone();
    let k3 = key.clone();
    let k3b = key.clone();
    let k5 = key.clone();
    let k5b = key.clone();
    let ka = key;

    let sub_initial = move || {
        state.subjects_teachers.get()
            .iter()
            .find(|s| format!("{}:{}", s.teacher_id, s.id) == ka)
            .and_then(|s| s.subject_title.chars().next())
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_else(|| "У".into())
    };

    column((
        row((
            // Subject Avatar Circle
            circle()
                .fill(move || Color::hex(state.accent_color.get()).with_alpha(0.15))
                .frame(44.0, 44.0)
                .overlay(
                    label(sub_initial)
                        .font(Font::Headline)
                        .bold()
                        .color(move || Color::hex(state.accent_color.get()))
                        .align(TextAlign::Center),
                ),

            // Subject and Teacher information
            column((
                // Subject title
                label(move || {
                    state.subjects_teachers.get()
                        .iter()
                        .find(|s| format!("{}:{}", s.teacher_id, s.id) == k1)
                        .map(|s| s.subject_title.clone())
                        .unwrap_or_default()
                })
                .font(Font::Headline)
                .bold()
                .grow(),

                // Teacher full name
                row((
                    label("👤").font(Font::Caption),
                    label(move || {
                        state.subjects_teachers.get()
                            .iter()
                            .find(|s| format!("{}:{}", s.teacher_id, s.id) == k2)
                            .map(|s| s.teacher.clone())
                            .unwrap_or_default()
                    })
                    .font(Font::Subheadline)
                    .secondary()
                    .grow(),
                ))
                .spacing(4.0)
                .align(VAlign::Center),

                // Tags for study level and group
                row((
                    when(move || {
                        state.subjects_teachers.get()
                            .iter()
                            .find(|s| format!("{}:{}", s.teacher_id, s.id) == k3)
                            .map(|s| !s.level_of_study.is_empty())
                            .unwrap_or(false)
                    }, move || {
                        let lk = k3b.clone();
                        column((
                            label(move || {
                                state.subjects_teachers.get()
                                    .iter()
                                    .find(|s| format!("{}:{}", s.teacher_id, s.id) == lk)
                                    .map(|s| s.level_of_study.clone())
                                    .unwrap_or_default()
                            })
                            .font(Font::Caption2)
                            .weight(FontWeight::Medium)
                            .color(move || Color::hex(state.accent_color.get())),
                        ))
                        .padding(Insets { top: 2.0, leading: 6.0, bottom: 2.0, trailing: 6.0 })
                        .background(move || Color::hex(state.accent_color.get()).with_alpha(0.1))
                        .corner_radius(6.0)
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
                        column((
                            label(move || {
                                state.subjects_teachers.get()
                                    .iter()
                                    .find(|s| format!("{}:{}", s.teacher_id, s.id) == gk)
                                    .and_then(|s| s.group_name.clone())
                                    .map(|g| format!("Гр: {}", g))
                                    .unwrap_or_default()
                            })
                            .font(Font::Caption2)
                            .secondary(),
                        ))
                        .padding(Insets { top: 2.0, leading: 6.0, bottom: 2.0, trailing: 6.0 })
                        .background(move || {
                            if day::dark_mode() {
                                Color::rgba(1.0, 1.0, 1.0, 0.08)
                            } else {
                                Color::rgba(0.0, 0.0, 0.0, 0.05)
                            }
                        })
                        .corner_radius(6.0)
                    }),
                ))
                .spacing(6.0)
                .padding(Insets { top: 2.0, leading: 0.0, bottom: 0.0, trailing: 0.0 }),
            ))
            .spacing(3.0)
            .grow(),
        ))
        .spacing(12.0)
        .align(VAlign::Center)
        .padding(Insets { top: 12.0, leading: 14.0, bottom: 12.0, trailing: 14.0 })
        .background(move || {
            if day::dark_mode() {
                Color::rgba(1.0, 1.0, 1.0, 0.06)
            } else {
                Color::rgba(0.0, 0.0, 0.0, 0.04)
            }
        })
        .corner_radius(16.0)
        .padding(Insets { top: 4.0, leading: 16.0, bottom: 4.0, trailing: 16.0 }),
    ))
    .spacing(0.0)
}
