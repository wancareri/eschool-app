use crate::app::AppState;
use crate::features;
use crate::shared::{colors};
use crate::res;
use day::prelude::*;

pub fn render() -> impl Piece {
    let state = AppState::ambient();

    column((
        when(
            move || state.is_authenticated.get(),
            move || profile_view(state),
        ),
        when(
            move || !state.is_authenticated.get(),
            move || auth_form(state),
        ),
    ))
    .spacing(0.0)
    .grow()
}

fn profile_view(state: AppState) -> impl Piece {
    let remember = crate::shared::secure::load("auth.remember_me")
        .map(|v| v == "true")
        .unwrap_or(false);

    column((
        column((
            label(move || res::str::settings_title().format())
                .font(Font::LargeTitle),
            label("Профиль")
                .font(Font::Subheadline)
                .secondary(),
        ))
        .spacing(6.0)
        .padding(Insets { top: 16.0, leading: 20.0, bottom: 12.0, trailing: 20.0 }),
        spacer(),
        form((
            section(
                (
                    label(move || state.full_name.get())
                        .font(Font::Title2),
                    label(move || state.school_name.get())
                        .font(Font::Body).secondary(),
                    when(
                        move || !state.class_label.get().is_empty(),
                        move || label(move || format!("Класс: {}", state.class_label.get()))
                            .font(Font::Body).secondary(),
                    ),
                    label(move || if remember { "Сессия сохранена" } else { "Сессия не сохранена" })
                        .font(Font::Caption)
                        .secondary(),
                )
            ).title("Ученик"),
            section(
                (
                    when(
                        move || state.loading.get(),
                        || label("Загрузка данных…")
                            .font(Font::Caption)
                            .secondary(),
                    ),
                    when(
                        move || !state.error_msg.get().is_empty(),
                        move || label(move || state.error_msg.get())
                            .font(Font::Caption)
                            .color(colors::ERROR),
                    ),
                )
            ).title("Статус"),
        ))
        .padding(Insets { top: 0.0, leading: 0.0, bottom: 16.0, trailing: 0.0 }),
        form((
            section(
                (
                    crate::native::button::render("Обновить данные", "arrow.clockwise"),
                    divider(),
                    crate::native::button::render("Выйти", "rectangle.portrait.and.arrow.right"),
                )
            ).title("Действия"),
        ))
        .padding(Insets { top: 0.0, leading: 0.0, bottom: 20.0, trailing: 0.0 }),
    ))
    .spacing(0.0)
    .grow()
}

fn auth_form(state: AppState) -> impl Piece {
    let username = Signal::new(String::new());
    let password = Signal::new(String::new());
    let remember = Signal::new(true); // Default to true like Telegram
    let error_local = Signal::new(String::new());
    let logging_in = Signal::new(false);

    scroll(column((
        spacer().height(24.0),

        // Telegram-style circular app icon
        circle()
            .fill(move || Color::hex(state.accent_color.get()).with_alpha(0.15))
            .frame(80.0, 80.0)
            .overlay(
                label("🎓")
                    .font(Font::System(38.0))
                    .align(TextAlign::Center),
            ),

        spacer().height(16.0),

        // Title and welcome subtitle
        label("Электронный дневник")
            .font(Font::Title)
            .bold()
            .align(TextAlign::Center),

        label("Войдите с помощью школьного логина для доступа к расписанию и оценкам")
            .font(Font::Subheadline)
            .secondary()
            .align(TextAlign::Center)
            .padding(Insets { top: 6.0, leading: 32.0, bottom: 24.0, trailing: 32.0 }),

        // Input card (Telegram-style grouped fields)
        column((
            // Username row
            row((
                label("👤")
                    .font(Font::Callout)
                    .frame(24.0, 24.0),
                text_field(username)
                    .placeholder("Имя пользователя или логин")
                    .id("username-field")
                    .grow(),
            ))
            .spacing(12.0)
            .align(VAlign::Center)
            .padding(Insets { top: 12.0, leading: 14.0, bottom: 12.0, trailing: 14.0 }),

            // Divider between fields
            spacer()
                .frame(0.0, 0.5)
                .background(move || {
                    if day::dark_mode() {
                        Color::rgba(1.0, 1.0, 1.0, 0.08)
                    } else {
                        Color::rgba(0.0, 0.0, 0.0, 0.08)
                    }
                })
                .padding(Insets { top: 0.0, leading: 48.0, bottom: 0.0, trailing: 0.0 })
                .grow_w(),

            // Password row
            row((
                label("🔒")
                    .font(Font::Callout)
                    .frame(24.0, 24.0),
                text_field(password)
                    .placeholder("Пароль")
                    .secure()
                    .id("password-field")
                    .grow(),
            ))
            .spacing(12.0)
            .align(VAlign::Center)
            .padding(Insets { top: 12.0, leading: 14.0, bottom: 12.0, trailing: 14.0 }),
        ))
        .background(move || {
            if day::dark_mode() {
                Color::rgba(1.0, 1.0, 1.0, 0.06)
            } else {
                Color::rgba(0.0, 0.0, 0.0, 0.04)
            }
        })
        .corner_radius(16.0)
        .padding(Insets { top: 2.0, leading: 0.0, bottom: 2.0, trailing: 0.0 })
        .padding(Insets { top: 0.0, leading: 20.0, bottom: 14.0, trailing: 20.0 }),

        // Remember me toggle row
        row((
            label("Сохранить вход на этом устройстве")
                .font(Font::Subheadline)
                .grow(),
            toggle(remember),
        ))
        .align(VAlign::Center)
        .padding(Insets { top: 0.0, leading: 24.0, bottom: 18.0, trailing: 24.0 }),

        // Error message card
        when(
            move || {
                let local_err = error_local.get();
                let state_err = state.error_msg.get();
                let is_loading = state.loading.get();
                (!local_err.is_empty() || !state_err.is_empty()) && !is_loading
            },
            move || {
                let local_err = error_local.get();
                let state_err = state.error_msg.get();
                let msg = if !local_err.is_empty() { local_err } else { state_err };
                row((
                    label("⚠️").font(Font::Callout),
                    label(msg)
                        .font(Font::Callout)
                        .color(colors::ERROR)
                        .grow(),
                ))
                .spacing(8.0)
                .align(VAlign::Center)
                .padding(Insets { top: 10.0, leading: 14.0, bottom: 10.0, trailing: 14.0 })
                .background(colors::ERROR.with_alpha(0.12))
                .corner_radius(12.0)
                .padding(Insets { top: 0.0, leading: 20.0, bottom: 16.0, trailing: 20.0 })
            },
        ),

        // Loading message
        when(
            move || state.loading.get(),
            || label("Выполняется вход…")
                .font(Font::Subheadline)
                .secondary()
                .align(TextAlign::Center)
                .padding(Insets { top: 0.0, leading: 20.0, bottom: 14.0, trailing: 20.0 }),
        ),

        // Primary Action Button (Telegram-style prominent rounded button)
        button(move || {
            if logging_in.get() || state.loading.get() { "Вход…" } else { "Войти" }
        })
        .prominent()
        .action(move || {
            if state.loading.get() { return; }
            let u = username.get_untracked();
            let p = password.get_untracked();
            let r = remember.get_untracked();
            if u.trim().is_empty() {
                error_local.set("Введите имя пользователя".into());
                return;
            }
            if p.is_empty() {
                error_local.set("Введите пароль".into());
                return;
            }
            error_local.set(String::new());
            logging_in.set(true);
            features::auth::login(state, &u, &p, r);
        })
        .id("login-btn")
        .grow()
        .padding(Insets { top: 0.0, leading: 20.0, bottom: 24.0, trailing: 20.0 }),

        spacer().height(20.0),
    ))
    .align(HAlign::Center)
    .grow())
    .grow()
}
