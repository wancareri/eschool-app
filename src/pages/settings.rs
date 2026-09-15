use crate::app::AppState;
use crate::features;
use crate::res;
use crate::shared::{colors, nslog};
use day::prelude::*;

pub fn render() -> impl Piece {
    let state = AppState::ambient();

    scroll(column((
        column((
            label(move || res::str::settings_title().format())
                .font(Font::LargeTitle).align(TextAlign::Center),
            label("Настройки").font(Font::Subheadline).secondary().align(TextAlign::Center),
        ))
        .spacing(6.0)
        .padding(Insets { top: 16.0, leading: 20.0, bottom: 8.0, trailing: 20.0 }),

        when(
            move || state.is_authenticated.get(),
            move || profile_section(state),
        ),

        appearance_section(state),

        system_settings(),

        dev_settings(state),

        when(
            move || state.is_authenticated.get(),
            move || logout_section(state),
        ),
    ))
    .spacing(0.0)
    .grow())
    .grow()
}

fn profile_section(state: AppState) -> impl Piece {
    form((
        section(
            (
                label(move || state.full_name.get())
                    .font(Font::Title3).align(TextAlign::Center),
                label(move || state.school_name.get())
                    .font(Font::Body).secondary().align(TextAlign::Center),
                when(
                    move || !state.class_label.get().is_empty(),
                    move || label(move || format!("Класс: {}", state.class_label.get()))
                        .font(Font::Body).secondary().align(TextAlign::Center),
                ),
            )
        ).title("Профиль"),
    ))
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}

fn appearance_section(state: AppState) -> impl Piece {
    column((
        form((
            section(
                (
                    label("Акцентный цвет").font(Font::Headline),
                    accent_option(state, "Синий", colors::BLUE),
                    accent_option(state, "Зелёный", colors::GREEN),
                    accent_option(state, "Фиолетовый", colors::PURPLE),
                    accent_option(state, "Оранжевый", colors::ORANGE),
                    accent_option(state, "Красный", colors::RED),
                )
            ).title("Цвета"),
        )),
    ))
    .spacing(0.0)
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}

fn system_settings() -> impl Piece {
    form((day_piece_settings::settings_sections(
        crate::THEME_KEY,
        crate::LOCALE_KEY,
        res::locales::ALL,
    ),))
}

fn accent_option(state: AppState, lbl: &'static str, hex: u32) -> impl Piece {
    let s = state;
    button(move || {
        if s.accent_color.get() == hex { format!("\u{2713} \u{25CF} {lbl}") } else { format!("\u{25CF} {lbl}") }
    })
    .id(format!("accent-{hex}"))
    .tint(move || Color::hex(state.accent_color.get()))
    .action(move || {
        day::prefs::set("app.accent_color", &hex.to_string());
        colors::set_accent(hex);
        state.accent_color.set(hex);
    })
}

fn logout_section(state: AppState) -> impl Piece {
    form((
        section(
            (button("Выйти из аккаунта")
                .tint(move || Color::hex(state.accent_color.get()))
                .action(move || features::auth::logout(state)),)
        ).title("Аккаунт"),
    ))
    .padding(Insets { top: 16.0, leading: 0.0, bottom: 20.0, trailing: 0.0 })
}

pub fn settings_body() -> impl Piece {
    form((day_piece_settings::settings_sections(
        crate::THEME_KEY,
        crate::LOCALE_KEY,
        res::locales::ALL,
    ),))
}

fn dev_settings(state: AppState) -> impl Piece {
    let dev_output = Signal::new(String::new());
    let dev_output2 = dev_output.clone();

    form((
        section(
            (
                label("Инструменты разработчика").font(Font::Headline).color(colors::WARNING),

                label(move || {
                    let v = dev_output2.get();
                    if v.is_empty() { String::new() } else { v }
                }).font(Font::Caption).color(colors::INFO),

                button("Показать токен")
                    .tint(move || Color::hex(state.accent_color.get()))
                    .action(move || {
                        if let Some(t) = features::auth::get_token() {
                            let preview = if t.len() > 40 { &t[..40] } else { &t };
                            let msg = format!("Token: {preview}...");
                            nslog::nslog(&format!("[Dev] {msg}"));
                            dev_output2.set(msg);
                        } else {
                            nslog::nslog("[Dev] No token");
                            dev_output2.set("Нет токена".into());
                        }
                        state.log_version.set(state.log_version.get() + 1);
                    }),

                button("Обновить токен")
                    .tint(move || Color::hex(state.accent_color.get()))
                    .action(move || {
                        nslog::nslog("[Dev] Manual token refresh...");
                        let msg = match features::auth::try_refresh_token() {
                            Some(_) => "Refresh OK".into(),
                            None => "Refresh FAILED".into(),
                        };
                        nslog::nslog(&format!("[Dev] {msg}"));
                        dev_output.set(msg);
                        state.log_version.set(state.log_version.get() + 1);
                    }),

                button("Очистить кэш")
                    .tint(move || Color::hex(state.accent_color.get()))
                    .action(move || {
                        day::prefs::set("diary.cache", "");
                        nslog::nslog("[Dev] Cache cleared");
                        dev_output2.set("Кэш очищен".into());
                        state.log_version.set(state.log_version.get() + 1);
                    }),

                button("Перезагрузить данные")
                    .tint(move || Color::hex(state.accent_color.get()))
                    .action(move || {
                        dev_output2.set("Загрузка...".into());
                        state.log_version.set(state.log_version.get() + 1);
                        features::diary::load_all(state);
                    }),

                button("Показать ID")
                    .tint(move || Color::hex(state.accent_color.get()))
                    .action(move || {
                        let (sid, cid, pid) = features::auth::get_stored_ids();
                        let msg = format!("school={sid}\nclass={cid}\nprofile={pid}");
                        nslog::nslog(&format!("[Dev] {msg}"));
                        dev_output2.set(msg);
                        state.log_version.set(state.log_version.get() + 1);
                    }),
            )
        ).title("Dev Tools"),

        section(
            (
                label("Логи приложения").font(Font::Headline).color(colors::INFO),

                scroll(
                    label(move || {
                        let _ = state.log_version.get(); // track for reactivity
                        nslog::get_logs()
                    })
                        .font(Font::Caption)
                        .align(TextAlign::Leading)
                )
                .height(300.0),

                row((
                    button("Обновить")
                        .tint(move || Color::hex(state.accent_color.get()))
                        .action(move || {
                            state.log_version.set(state.log_version.get() + 1);
                        }),
                    button("Очистить логи")
                        .tint(move || Color::hex(state.accent_color.get()))
                        .action(move || {
                            nslog::clear_logs();
                            state.log_version.set(state.log_version.get() + 1);
                        }),
                )).spacing(8.0),
            )
        ).title("Логи"),
    ))
    .padding(Insets { top: 16.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}
