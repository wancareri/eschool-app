use crate::core::colors;
use crate::core::network::models::*;
use crate::core::state::ESchoolState;
use crate::res;
use day::prelude::*;

pub(crate) fn teachers_page() -> impl Piece {
    let state = ESchoolState::ambient();

    scroll(column((
        column((
            label(move || res::str::teachers_title().format())
                .font(Font::LargeTitle),
            label("Предметы и преподаватели")
                .font(Font::Subheadline)
                .secondary(),
        ))
        .spacing(6.0)
        .padding(Insets { top: 16.0, leading: 20.0, bottom: 8.0, trailing: 20.0 }),
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
            || column((
                spacer(),
                label("Загрузка…")
                    .font(Font::Body).secondary().align(TextAlign::Center),
                spacer(),
            )).grow(),
        ),
        when(
            move || state.is_authenticated.get() && !state.teachers_loading.get(),
            move || teachers_list(state),
        ),
    ))
    .spacing(0.0)
    .grow())
    .grow()
}

fn teachers_list(state: ESchoolState) -> impl Piece {
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
            teacher_row(state, key).any()
        },
    )
}

fn teacher_row(state: ESchoolState, key: String) -> impl Piece {
    let k1 = key.clone();
    let k2 = key.clone();
    let k3 = key.clone();
    let k5 = key;

    let subject_fn = move || {
        state.subjects_teachers.get()
            .iter()
            .find(|s| format!("{}:{}", s.teacher_id, s.id) == k1)
            .map(|s| s.subject_title.clone())
            .unwrap_or_default()
    };
    let teacher_fn = move || {
        state.subjects_teachers.get()
            .iter()
            .find(|s| format!("{}:{}", s.teacher_id, s.id) == k2)
            .map(|s| s.teacher.clone())
            .unwrap_or_default()
    };

    let level_key = k3.clone();
    let level_check = move || {
        state.subjects_teachers.get()
            .iter()
            .find(|s| format!("{}:{}", s.teacher_id, s.id) == k3)
            .map(|s| !s.level_of_study.is_empty())
            .unwrap_or(false)
    };

    let group_key_check = k5.clone();
    let group_check = move || {
        state.subjects_teachers.get()
            .iter()
            .find(|s| format!("{}:{}", s.teacher_id, s.id) == group_key_check)
            .and_then(|s| s.group_name.as_ref())
            .map(|g| !g.is_empty())
            .unwrap_or(false)
    };

    row((
        label("📚").font(Font::Title2),
        column((
            label(subject_fn).font(Font::Headline),
            label(teacher_fn).font(Font::Body).secondary(),
            row((
                when(level_check, move || {
                    let lk = level_key.clone();
                    label(move || {
                        state.subjects_teachers.get()
                            .iter()
                            .find(|s| format!("{}:{}", s.teacher_id, s.id) == lk)
                            .map(|s| format!("Уровень: {}", s.level_of_study))
                            .unwrap_or_default()
                    }).font(Font::Caption).color(colors::INFO)
                }),
                when(group_check, move || {
                    let gk = k5.clone();
                    label(move || {
                        state.subjects_teachers.get()
                            .iter()
                            .find(|s| format!("{}:{}", s.teacher_id, s.id) == gk)
                            .and_then(|s| s.group_name.clone())
                            .map(|g| format!(" · Группа: {}", g))
                            .unwrap_or_default()
                    }).font(Font::Caption).secondary()
                }),
            )).spacing(0.0),
        ))
        .spacing(3.0)
        .align(HAlign::Leading)
        .grow(),
    ))
    .spacing(14.0)
    .padding(Insets { top: 10.0, leading: 20.0, bottom: 10.0, trailing: 20.0 })
}
