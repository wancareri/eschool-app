use crate::core::state::ESchoolState;
use crate::res;
use day::prelude::*;

pub(crate) fn settings_body() -> impl Piece {
    form((day_piece_settings::settings_sections(
        crate::THEME_KEY,
        crate::LOCALE_KEY,
        res::locales::ALL,
    ),))
}

pub(crate) fn settings_page() -> impl Piece {
    let state = ESchoolState::ambient();

    scroll(column((
        column((
            label(move || res::str::settings_title().format())
                .font(Font::LargeTitle),
            label("Настройки")
                .font(Font::Subheadline)
                .secondary(),
        ))
        .spacing(6.0)
        .padding(Insets { top: 16.0, leading: 20.0, bottom: 8.0, trailing: 20.0 }),

        when(
            move || state.is_authenticated.get(),
            move || {
                form((
                    section(
                        (
                            label(move || state.full_name.get())
                                .font(Font::Title3),
                            label(move || state.school_name.get())
                                .font(Font::Body).secondary(),
                            when(
                                move || !state.class_label.get().is_empty(),
                                move || label(move || format!("Класс: {}", state.class_label.get()))
                                    .font(Font::Body).secondary(),
                            ),
                        )
                    ).title("Профиль"),
                ))
                .padding(Insets { top: 0.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
            },
        ),

        settings_body(),

        when(
            move || state.is_authenticated.get(),
            move || {
                form((
                    section(
                        (button("Выйти из аккаунта")
                            .action(move || state.logout()),)
                    ).title("Аккаунт"),
                ))
                .padding(Insets { top: 16.0, leading: 0.0, bottom: 20.0, trailing: 0.0 })
            },
        ),
    ))
    .spacing(0.0)
    .grow())
    .grow()
}
