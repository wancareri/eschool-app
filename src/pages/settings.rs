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
            move || {
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
            },
        ),

        general_settings(),

        settings_body(),

        #[cfg(debug_assertions)]
        dev_settings(state),

        when(
            move || state.is_authenticated.get(),
            move || {
                form((
                    section(
                        (button("Выйти из аккаунта")
                            .action(move || features::auth::logout(state)),)
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

pub fn settings_body() -> impl Piece {
    form((day_piece_settings::settings_sections(
        crate::THEME_KEY,
        crate::LOCALE_KEY,
        res::locales::ALL,
    ),))
}

fn general_settings() -> impl Piece {
    column((
        form((
            section(
                (
                    label("Тема оформления").font(Font::Headline),
                    theme_option("Классическая", "legacy"),
                    theme_option("Liquid Glass", "liquid_glass"),
                )
            ).title("Внешний вид"),
        )),

        form((
            section(
                (
                    label("Акцентный цвет").font(Font::Headline),
                    accent_option("Синий", colors::BLUE),
                    accent_option("Зелёный", colors::GREEN),
                    accent_option("Фиолетовый", colors::PURPLE),
                    accent_option("Оранжевый", colors::ORANGE),
                    accent_option("Красный", colors::RED),
                )
            ).title("Цвета"),
        )),
    ))
    .spacing(0.0)
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}

fn theme_option(lbl: &'static str, key: &'static str) -> impl Piece {
    let k = key.to_string();
    let current = day::prefs::get("app.theme_style").unwrap_or_default();
    let is_active = current == key;
    let display = if is_active { format!("\u{2713} {lbl}") } else { lbl.to_string() };
    button(display)
        .id(format!("theme-{key}"))
        .action(move || { day::prefs::set("app.theme_style", &k); })
}

fn accent_option(lbl: &'static str, hex: u32) -> impl Piece {
    let current = day::prefs::get("app.accent_color")
        .map(|v| v.parse::<u32>().unwrap_or(0x3B82F6))
        .unwrap_or(0x3B82F6);
    let is_active = current == hex;
    let marker = "\u{25CF}";
    let display = if is_active { format!("\u{2713} {marker} {lbl}") } else { format!("{marker} {lbl}") };
    button(display)
        .id(format!("accent-{hex}"))
        .action(move || { day::prefs::set("app.accent_color", &hex.to_string()); })
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
    ))
    .padding(Insets { top: 16.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}
