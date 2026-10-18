use crate::app::AppState;
use crate::features;
use crate::shared::{colors};
use crate::res;
use day::prelude::*;

pub fn render() -> impl Piece {
    let state = AppState::ambient();

    zstack((
        when(
            move || state.is_authenticated.get(),
            move || profile_view(state).animation(AnimSpec::ease_out(350)).any(),
        ),
        when(
            move || !state.is_authenticated.get(),
            move || auth_form(state).animation(AnimSpec::ease_out(350)).any(),
        ),
    ))
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
    let step = Signal::new(0u8); // 0 = username, 1 = password
    let username = Signal::new(String::new());
    let password = Signal::new(String::new());
    let password_focused = Signal::new(false);
    let remember = Signal::new(true);
    let error_local = Signal::new(String::new());
    let kb_h = crate::shared::keyboard::keyboard_height_signal();

    column((
        scroll(
            column((
                spacer().height(36.0),

            // Telegram-style greeting badge
            circle()
                .fill(move || Color::hex(state.accent_color.get()).with_alpha(0.14))
                .frame(76.0, 76.0)
                .overlay(
                    label("👋")
                        .font(Font::System(38.0))
                        .align(TextAlign::Center),
                ),

            spacer().height(16.0),

            // Greeting header — adapts between Step 0 and Step 1
            when(
                move || step.get() == 0,
                || column((
                    label("Вход в аккаунт")
                        .font(Font::Title)
                        .bold()
                        .align(TextAlign::Center),
                    label("Введите логин или email вашей школы для доступа к дневнику")
                        .font(Font::Subheadline)
                        .secondary()
                        .align(TextAlign::Center)
                        .padding(Insets { top: 6.0, leading: 32.0, bottom: 20.0, trailing: 32.0 }),
                )),
            )
            .otherwise(move || {
                let u = username.get();
                column((
                    label("Введите пароль")
                        .font(Font::Title)
                        .bold()
                        .align(TextAlign::Center),
                    label(move || {
                        if u.trim().is_empty() {
                            "Введите пароль для входа".into()
                        } else {
                            format!("Аккаунт: {}", u.trim())
                        }
                    })
                    .font(Font::Subheadline)
                    .color(move || Color::hex(state.accent_color.get()))
                    .align(TextAlign::Center)
                    .padding(Insets { top: 6.0, leading: 32.0, bottom: 20.0, trailing: 32.0 }),
                ))
            }),

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
                    let is_network = msg.contains("error sending request")
                        || msg.contains("Не удалось подключиться")
                        || msg.contains("таймаут")
                        || msg.contains("dns error");

                    if is_network {
                        column((
                            label("Не удалось подключиться к серверу")
                                .font(Font::Headline)
                                .color(colors::ERROR)
                                .align(TextAlign::Center),
                            label("Проверьте интернет или отключите VPN/прокси")
                                .font(Font::Subheadline)
                                .secondary()
                                .align(TextAlign::Center),
                            label(msg)
                                .font(Font::Caption2)
                                .color(colors::ERROR.with_alpha(0.7))
                                .align(TextAlign::Center),
                        ))
                        .spacing(4.0)
                        .padding(Insets { top: 10.0, leading: 14.0, bottom: 10.0, trailing: 14.0 })
                        .background(colors::ERROR.with_alpha(0.12))
                        .corner_radius(12.0)
                        .padding(Insets { top: 0.0, leading: 24.0, bottom: 12.0, trailing: 24.0 })
                        .any()
                    } else {
                        label(msg)
                            .font(Font::Callout)
                            .color(colors::ERROR)
                            .align(TextAlign::Center)
                            .padding(Insets { top: 8.0, leading: 14.0, bottom: 8.0, trailing: 14.0 })
                            .background(colors::ERROR.with_alpha(0.12))
                            .corner_radius(12.0)
                            .padding(Insets { top: 0.0, leading: 24.0, bottom: 12.0, trailing: 24.0 })
                            .any()
                    }
                },
            ),

            // Step 0: Username entry card
            when(
                move || step.get() == 0,
                move || {
                    column((
                        column((
                            label("ЛОГИН ИЛИ EMAIL")
                                .font(Font::Caption2)
                                .bold()
                                .secondary(),
                            text_field(username)
                                .on_submit(move || {
                                    let u = username.get_untracked();
                                    if u.trim().is_empty() {
                                        error_local.set("Введите имя пользователя".into());
                                    } else {
                                        error_local.set(String::new());
                                        state.error_msg.set(String::new());
                                        crate::shared::haptics::tick();
                                        password_focused.set(true);
                                        with_animation(AnimSpec::ease_out(250), move || step.set(1));
                                    }
                                }),
                        ))
                        .spacing(6.0)
                        .padding(Insets { top: 12.0, leading: 16.0, bottom: 12.0, trailing: 16.0 })
                        .background(move || {
                            if day::dark_mode() {
                                Color::rgba(1.0, 1.0, 1.0, 0.07)
                            } else {
                                Color::rgba(0.0, 0.0, 0.0, 0.04)
                            }
                        })
                        .corner_radius(16.0)
                        .padding(Insets { top: 0.0, leading: 20.0, bottom: 16.0, trailing: 20.0 }),

                        // Remember session toggle
                        row((
                            label("Сохранить сессию на устройстве")
                                .font(Font::Subheadline)
                                .grow(),
                            toggle(remember),
                        ))
                        .align(VAlign::Center)
                        .padding(Insets { top: 0.0, leading: 24.0, bottom: 20.0, trailing: 24.0 }),
                    ))
                },
            )
            // Step 1: Password entry card
            .otherwise(move || {
                column((
                    column((
                        label("ПАРОЛЬ")
                            .font(Font::Caption2)
                            .bold()
                            .secondary(),
                        text_field(password)
                            .secure()
                            .focused(password_focused)
                            .id("password-field")
                            .on_submit(move || {
                                if state.loading.get() { return; }
                                let u = username.get_untracked();
                                let p = password.get_untracked();
                                let r = remember.get_untracked();
                                if p.is_empty() {
                                    error_local.set("Введите пароль".into());
                                    return;
                                }
                                error_local.set(String::new());
                                state.error_msg.set(String::new());
                                crate::shared::haptics::pop();
                                features::auth::login(state, &u, &p, r);
                            }),
                    ))
                    .spacing(6.0)
                    .padding(Insets { top: 12.0, leading: 16.0, bottom: 12.0, trailing: 16.0 })
                    .background(move || {
                        if day::dark_mode() {
                            Color::rgba(1.0, 1.0, 1.0, 0.07)
                        } else {
                            Color::rgba(0.0, 0.0, 0.0, 0.04)
                        }
                    })
                    .corner_radius(16.0)
                    .padding(Insets { top: 0.0, leading: 20.0, bottom: 12.0, trailing: 20.0 }),

                    // Back button to change login
                    row((
                        button("← Изменить логин")
                            .action(move || {
                                crate::shared::haptics::tick();
                                error_local.set(String::new());
                                password_focused.set(false);
                                with_animation(AnimSpec::ease_out(250), move || step.set(0));
                            }),
                        spacer().grow(),
                    ))
                    .padding(Insets { top: 0.0, leading: 24.0, bottom: 20.0, trailing: 24.0 }),
                ))
            }),

            spacer().height(24.0),
        ))
        .align(HAlign::Center)
        )
        .on_tap(move || {
            crate::shared::keyboard::dismiss();
        })
        .grow(),

        // Prominent action button at the bottom, lifts with keyboard and responds immediately
        button(move || {
            if state.loading.get() {
                "Вход в систему…"
            } else if step.get() == 0 {
                "Продолжить"
            } else {
                "Войти"
            }
        })
        .prominent()
        .frame(0.0, 50.0)
        .grow_w()
        .corner_radius(25.0)
        .action(move || {
            if state.loading.get() { return; }
            let u = username.get_untracked();
            let p = password.get_untracked();
            let r = remember.get_untracked();

            if step.get() == 0 {
                if u.trim().is_empty() {
                    error_local.set("Введите имя пользователя".into());
                } else {
                    error_local.set(String::new());
                    state.error_msg.set(String::new());
                    crate::shared::haptics::tick();
                    password_focused.set(true);
                    with_animation(AnimSpec::ease_out(250), move || step.set(1));
                }
            } else {
                if p.is_empty() {
                    error_local.set("Введите пароль".into());
                    return;
                }
                error_local.set(String::new());
                state.error_msg.set(String::new());
                crate::shared::haptics::pop();
                features::auth::login(state, &u, &p, r);
            }
        })
        .id("login-btn")
        .padding(Insets { top: 8.0, leading: 20.0, bottom: 0.0, trailing: 20.0 })
        .translation(0.0, move || -kb_h.get())
        .animation(AnimSpec::ease_out(250)),
    ))
    .grow()
}
