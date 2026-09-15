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

        #[cfg(debug_assertions)]
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
                    label("Тема оформления").font(Font::Headline),
                    theme_option(state, "Классическая", "legacy"),
                    theme_option(state, "Liquid Glass", "liquid_glass"),
                )
            ).title("Внешний вид"),
        )),

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

fn theme_option(state: AppState, lbl: &'static str, key: &'static str) -> impl Piece {
    let k = key.to_string();
    let s = state;
    button(move || {
        // Read from signal or prefs — but we need a signal for reactivity
        // Since we don't have a theme signal, read from prefs each time
        // The button closure is re-evaluated when any tracked signal changes
        // We track state.accent_color to force re-eval on any settings change
        let _ = s.accent_color.get(); // force reactivity
        let current = day::prefs::get("app.theme_style").unwrap_or_default();
        if current == key { format!("\u{2713} {lbl}") } else { lbl.to_string() }
    })
    .id(format!("theme-{key}"))
    .action(move || {
        day::prefs::set("app.theme_style", &k);
        // Force re-render by toggling a signal
        let cur = state.accent_color.get();
        state.accent_color.set(cur); // trigger reactivity
    })
}

fn accent_option(state: AppState, lbl: &'static str, hex: u32) -> impl Piece {
    let s = state;
    button(move || {
        if s.accent_color.get() == hex { format!("\u{2713} \u{25CF} {lbl}") } else { format!("\u{25CF} {lbl}") }
    })
    .id(format!("accent-{hex}"))
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

#[cfg(debug_assertions)]
fn dev_settings(state: AppState) -> impl Piece {
    form((
        section(
            (
                label("Инструменты разработчика").font(Font::Headline).color(colors::WARNING),

                button("Показать токен")
                    .action(move || {
                        if let Some(t) = features::auth::get_token() {
                            let preview = if t.len() > 40 { &t[..40] } else { &t };
                            nslog::nslog(&format!("[Dev] Token: {preview}..."));
                        } else {
                            nslog::nslog("[Dev] No token");
                        }
                    }),

                button("Обновить токен")
                    .action(move || {
                        nslog::nslog("[Dev] Manual token refresh...");
                        match features::auth::try_refresh_token() {
                            Some(_) => nslog::nslog("[Dev] Refresh OK"),
                            None => nslog::nslog("[Dev] Refresh FAILED"),
                        }
                    }),

                button("Очистить кэш")
                    .action(move || {
                        day::prefs::set("diary.cache", "");
                        nslog::nslog("[Dev] Cache cleared");
                    }),

                button("Перезагрузить данные")
                    .action(move || { features::diary::load_all(state); }),

                button("Показать ID")
                    .action(move || {
                        let (sid, cid, pid) = features::auth::get_stored_ids();
                        nslog::nslog(&format!("[Dev] school={sid} class={cid} profile={pid}"));
                    }),
            )
        ).title("Dev Tools"),

        section(
            (
                label("Логи приложения").font(Font::Headline).color(colors::INFO),

                scroll(
                    label(move || nslog::get_logs())
                        .font(Font::Caption)
                        .align(TextAlign::Leading)
                )
                .height(300.0),

                row((
                    button("Обновить").id("dev-log-refresh"),
                    button("Очистить логи")
                        .id("dev-log-clear")
                        .action(move || nslog::clear_logs()),
                )).spacing(8.0),
            )
        ).title("Логи"),
    ))
    .padding(Insets { top: 16.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}
