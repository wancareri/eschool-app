use crate::app::AppState;
use crate::features;
use crate::shared::colors;
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
    let remember = day::prefs::get("auth.remember_me")
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
    let remember = Signal::new(false);
    let error_local = Signal::new(String::new());
    let logging_in = Signal::new(false);

    column((
        column((
            label(move || res::str::app_title().format())
                .font(Font::LargeTitle),
            label("Вход в дневник")
                .font(Font::Subheadline)
                .secondary(),
        ))
        .spacing(6.0)
        .padding(Insets { top: 16.0, leading: 20.0, bottom: 12.0, trailing: 20.0 }),
        spacer(),
        form((
            section(
                (
                    labeled("Логин",
                        text_field(username)
                            .placeholder("Имя пользователя")
                            .id("username-field"),
                    ),
                    labeled("Пароль",
                        text_field(password)
                            .placeholder("Пароль")
                            .secure()
                            .id("password-field"),
                    ),
                    row((
                        toggle(remember),
                        label("Сохранить сессию")
                            .font(Font::Body),
                    ))
                    .spacing(8.0)
                    .padding(Insets { top: 8.0, leading: 0.0, bottom: 0.0, trailing: 0.0 }),
                )
            ).title("Учётные данные"),
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
                    label(msg)
                        .font(Font::Caption)
                        .color(colors::ERROR)
                        .align(TextAlign::Center)
                }
            ),
            when(
                move || state.loading.get(),
                || label("Выполняется вход…")
                    .font(Font::Caption)
                    .secondary()
                    .align(TextAlign::Center),
            ),
        ))
        .padding(Insets { top: 0.0, leading: 0.0, bottom: 12.0, trailing: 0.0 }),
        button(move || {
            if logging_in.get() { "Вход…" } else { "Войти" }
        })
        
        .action(move || {
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
            logging_in.set(false);
        })
        .id("login-btn")
        .padding(Insets { top: 0.0, leading: 20.0, bottom: 0.0, trailing: 20.0 }),
        spacer(),
    ))
    .spacing(0.0)
    .grow()
}
