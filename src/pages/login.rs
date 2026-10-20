use crate::app::AppState;
use crate::features;
use crate::shared::colors;
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
            move || !state.is_authenticated.get() && state.setup_new_password_active.get(),
            move || setup_password_form(state).animation(AnimSpec::ease_out(350)).any(),
        ),
        when(
            move || !state.is_authenticated.get() && !state.setup_new_password_active.get(),
            move || auth_form(state).animation(AnimSpec::ease_out(350)).any(),
        ),
    ))
    .grow()
}

fn profile_view(state: AppState) -> impl Piece {
    let remember = crate::shared::secure::load("auth.remember_me")
        .map(|v| v == "true")
        .unwrap_or(false);
    let show_change_pwd = Signal::new(false);

    zstack((
        column((
            column((
                label(move || res::str::settings_title().format())
                    .font(Font::LargeTitle),
                label("Профиль")
                    .font(Font::Subheadline)
                    .secondary(),
            ))
            .spacing(6.0)
            .padding(Insets { top: 16.0, leading: 20.0, bottom: 12.0, trailing: 20.0 })
            .grow_w(),

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
                ).title(move || if state.is_parent.get() { "Родитель" } else { "Ученик" }),

                when(
                    move || state.is_parent.get() && !state.parent_children.get().is_empty(),
                    move || {
                        let children = state.parent_children.get();
                        let rows: Vec<AnyPiece> = children
                            .into_iter()
                            .enumerate()
                            .map(|(this_idx, ch)| {
                                let name = ch.full_name.clone();
                                row((
                                    column((
                                        label(name.clone())
                                            .font(Font::Headline),
                                        when(
                                            move || state.active_child_index.get() == this_idx,
                                            move || label("Выбран для дневника")
                                                .font(Font::Caption)
                                                .color(move || Color::hex(state.accent_color.get()))
                                                .any(),
                                        ),
                                    ))
                                    .grow(),
                                    when(
                                        move || state.active_child_index.get() != this_idx,
                                        move || button("Выбрать")
                                            .action(move || {
                                                crate::shared::haptics::tick();
                                                features::diary::switch_child(state, this_idx);
                                            })
                                            .any(),
                                    ),
                                ))
                                .align(VAlign::Center)
                                .padding(Insets { top: 6.0, leading: 0.0, bottom: 6.0, trailing: 0.0 })
                                .any()
                            })
                            .collect();

                        section(PieceVec(rows)).title("Дети в дневнике").any()
                    },
                ),

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
                        button("Обновить данные")
                            .action(move || {
                                crate::shared::haptics::tick();
                                features::diary::load_all(state);
                            }),
                        when(
                            move || state.is_parent.get(),
                            move || {
                                button("Сменить пароль")
                                    .action(move || {
                                        crate::shared::haptics::tick();
                                        show_change_pwd.set(true);
                                    })
                                    .any()
                            },
                        ),
                        divider(),
                        button("Выйти")
                            .action(move || {
                                crate::shared::haptics::pop();
                                features::auth::logout(state);
                            }),
                    )
                ).title("Действия"),
            ))
            .padding(Insets { top: 0.0, leading: 0.0, bottom: 20.0, trailing: 0.0 }),
        ))
        .spacing(0.0)
        .grow(),

        when(
            move || show_change_pwd.get(),
            move || change_password_form(state, show_change_pwd).animation(AnimSpec::ease_out(300)).any(),
        ),
    ))
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

    scroll(
        column((
            spacer().height(24.0),

            // Greeting header — adapts between Step 0 and Step 1
            when(
                move || step.get() == 0,
                || column((
                    label("Вход в аккаунт")
                        .font(Font::Title)
                        .bold()
                        .align(TextAlign::Center),
                    label("Введите логин ученика или пригласительный код родителя для доступа к дневнику")
                        .font(Font::Subheadline)
                        .secondary()
                        .align(TextAlign::Center)
                        .padding(Insets { top: 6.0, leading: 32.0, bottom: 20.0, trailing: 32.0 }),
                ))
                .grow_w(),
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
                .grow_w()
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
                        .grow_w()
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
                            .grow_w()
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
                            label("ЛОГИН ИЛИ ПРИГЛАСИТЕЛЬНЫЙ КОД")
                                .font(Font::Caption2)
                                .bold()
                                .secondary(),
                            text_field(username)
                                .frame(0.0, 44.0)
                                .grow_w()
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
                        .spacing(4.0)
                        .padding(Insets { top: 10.0, leading: 16.0, bottom: 10.0, trailing: 16.0 })
                        .background(move || {
                            if day::dark_mode() {
                                Color::rgba(1.0, 1.0, 1.0, 0.07)
                            } else {
                                Color::rgba(0.0, 0.0, 0.0, 0.04)
                            }
                        })
                        .corner_radius(16.0)
                        .padding(Insets { top: 0.0, leading: 20.0, bottom: 16.0, trailing: 20.0 })
                        .grow_w(),

                        // Remember session toggle
                        row((
                            label("Сохранить сессию на устройстве")
                                .font(Font::Subheadline)
                                .grow(),
                            toggle(remember),
                        ))
                        .align(VAlign::Center)
                        .padding(Insets { top: 0.0, leading: 24.0, bottom: 20.0, trailing: 24.0 })
                        .grow_w(),
                    ))
                    .grow_w()
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
                            .frame(0.0, 44.0)
                            .grow_w()
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
                                crate::shared::keyboard::dismiss();
                                crate::shared::haptics::pop();
                                features::auth::login(state, &u, &p, r);
                            }),
                    ))
                    .spacing(4.0)
                    .padding(Insets { top: 10.0, leading: 16.0, bottom: 10.0, trailing: 16.0 })
                    .background(move || {
                        if day::dark_mode() {
                            Color::rgba(1.0, 1.0, 1.0, 0.07)
                        } else {
                            Color::rgba(0.0, 0.0, 0.0, 0.04)
                        }
                    })
                    .corner_radius(16.0)
                    .padding(Insets { top: 0.0, leading: 20.0, bottom: 12.0, trailing: 20.0 })
                    .grow_w(),

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
                    .padding(Insets { top: 0.0, leading: 24.0, bottom: 20.0, trailing: 24.0 })
                    .grow_w(),
                ))
                .grow_w()
            }),

            // Prominent action button directly following the input fields
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
                    crate::shared::keyboard::dismiss();
                    crate::shared::haptics::pop();
                    features::auth::login(state, &u, &p, r);
                }
            })
            .id("login-btn")
            .padding(Insets { top: 8.0, leading: 20.0, bottom: 16.0, trailing: 20.0 }),

            when(
                move || kb_h.get() > 0.0,
                move || spacer().frame(0.0, kb_h.get() + 20.0).any(),
            ),
        ))
        .grow_w()
    )
    .on_tap(move || {
        crate::shared::keyboard::dismiss();
    })
    .grow()
}

fn setup_password_form(state: AppState) -> impl Piece {
    let new_password = Signal::new(String::new());
    let confirm_password = Signal::new(String::new());
    let confirm_focused = Signal::new(false);
    let error_local = Signal::new(String::new());
    let remember = Signal::new(true);
    let kb_h = crate::shared::keyboard::keyboard_height_signal();

    scroll(
        column((
            spacer().height(24.0),

            column((
                label("Создание постоянного пароля")
                    .font(Font::Title)
                    .bold()
                    .align(TextAlign::Center),
                label("Вы вошли по временному коду. Придумайте надёжный постоянный пароль для кабинета родителя (12–15 символов, строчные и заглавные латинские буквы, цифры и спецсимволы)")
                    .font(Font::Subheadline)
                    .secondary()
                    .align(TextAlign::Center)
                    .padding(Insets { top: 6.0, leading: 24.0, bottom: 16.0, trailing: 24.0 }),
            ))
            .grow_w(),

            when(
                move || {
                    let local = error_local.get();
                    let state_err = state.error_msg.get();
                    !local.is_empty() || !state_err.is_empty()
                },
                move || {
                    let local = error_local.get();
                    let state_err = state.error_msg.get();
                    let msg = if !local.is_empty() { local } else { state_err };
                    label(msg)
                        .font(Font::Callout)
                        .color(colors::ERROR)
                        .align(TextAlign::Center)
                        .padding(Insets { top: 8.0, leading: 14.0, bottom: 8.0, trailing: 14.0 })
                        .background(colors::ERROR.with_alpha(0.12))
                        .corner_radius(12.0)
                        .padding(Insets { top: 0.0, leading: 20.0, bottom: 12.0, trailing: 20.0 })
                        .grow_w()
                        .any()
                },
            ),

            column((
                label("НОВЫЙ ПАРОЛЬ")
                    .font(Font::Caption2)
                    .bold()
                    .secondary(),
                text_field(new_password)
                    .secure()
                    .frame(0.0, 44.0)
                    .grow_w()
                    .on_submit(move || {
                        confirm_focused.set(true);
                    }),
            ))
            .spacing(4.0)
            .padding(Insets { top: 10.0, leading: 16.0, bottom: 10.0, trailing: 16.0 })
            .background(move || {
                if day::dark_mode() {
                    Color::rgba(1.0, 1.0, 1.0, 0.07)
                } else {
                    Color::rgba(0.0, 0.0, 0.0, 0.04)
                }
            })
            .corner_radius(16.0)
            .padding(Insets { top: 0.0, leading: 20.0, bottom: 12.0, trailing: 20.0 })
            .grow_w(),

            column((
                label("ПОВТОРИТЕ ПАРОЛЬ")
                    .font(Font::Caption2)
                    .bold()
                    .secondary(),
                text_field(confirm_password)
                    .secure()
                    .focused(confirm_focused)
                    .frame(0.0, 44.0)
                    .grow_w()
                    .on_submit(move || {
                        if state.loading.get() { return; }
                        let np = new_password.get_untracked();
                        let cp = confirm_password.get_untracked();
                        let r = remember.get_untracked();

                        if np.is_empty() {
                            error_local.set("Введите новый пароль".into());
                            return;
                        }
                        if np != cp {
                            error_local.set("Пароли не совпадают. Проверьте правильность ввода".into());
                            return;
                        }
                        if let Err(e) = eschool_api::auth::validate_parent_password(&np) {
                            error_local.set(e);
                            return;
                        }

                        if let Some((puuid, old_pwd, uname)) = state.setup_temp_password_data.get_untracked() {
                            error_local.set(String::new());
                            state.error_msg.set(String::new());
                            crate::shared::keyboard::dismiss();
                            crate::shared::haptics::pop();
                            features::auth::complete_parent_new_password(state, &puuid, &old_pwd, &np, &uname, r);
                        }
                    }),
            ))
            .spacing(4.0)
            .padding(Insets { top: 10.0, leading: 16.0, bottom: 10.0, trailing: 16.0 })
            .background(move || {
                if day::dark_mode() {
                    Color::rgba(1.0, 1.0, 1.0, 0.07)
                } else {
                    Color::rgba(0.0, 0.0, 0.0, 0.04)
                }
            })
            .corner_radius(16.0)
            .padding(Insets { top: 0.0, leading: 20.0, bottom: 12.0, trailing: 20.0 })
            .grow_w(),

            button(move || {
                if state.loading.get() {
                    "Сохранение…"
                } else {
                    "Создать пароль и войти"
                }
            })
            .prominent()
            .frame(0.0, 50.0)
            .grow_w()
            .corner_radius(25.0)
            .action(move || {
                if state.loading.get() { return; }
                let np = new_password.get_untracked();
                let cp = confirm_password.get_untracked();
                let r = remember.get_untracked();

                if np.is_empty() {
                    error_local.set("Введите новый пароль".into());
                    return;
                }
                if np != cp {
                    error_local.set("Пароли не совпадают. Проверьте правильность ввода".into());
                    return;
                }
                if let Err(e) = eschool_api::auth::validate_parent_password(&np) {
                    error_local.set(e);
                    return;
                }

                if let Some((puuid, old_pwd, uname)) = state.setup_temp_password_data.get_untracked() {
                    error_local.set(String::new());
                    state.error_msg.set(String::new());
                    crate::shared::keyboard::dismiss();
                    crate::shared::haptics::pop();
                    features::auth::complete_parent_new_password(state, &puuid, &old_pwd, &np, &uname, r);
                }
            })
            .padding(Insets { top: 8.0, leading: 20.0, bottom: 12.0, trailing: 20.0 })
            .grow_w(),

            button("← Назад к входу")
                .action(move || {
                    crate::shared::haptics::tick();
                    crate::shared::keyboard::dismiss();
                    state.setup_new_password_active.set(false);
                    state.setup_temp_password_data.set(None);
                })
                .padding(Insets { top: 0.0, leading: 24.0, bottom: 16.0, trailing: 24.0 }),

            when(
                move || kb_h.get() > 0.0,
                move || spacer().frame(0.0, kb_h.get() + 20.0).any(),
            ),
        ))
        .grow_w()
    )
    .on_tap(move || {
        crate::shared::keyboard::dismiss();
    })
    .grow()
}

fn change_password_form(state: AppState, open: Signal<bool>) -> impl Piece {
    let old_password = Signal::new(String::new());
    let new_password = Signal::new(String::new());
    let confirm_password = Signal::new(String::new());
    let confirm_focused = Signal::new(false);
    let error_local = Signal::new(String::new());
    let success_msg = Signal::new(String::new());
    let kb_h = crate::shared::keyboard::keyboard_height_signal();

    column((
        scroll(
            column((
                spacer().height(20.0),

                column((
                    label("Смена пароля")
                        .font(Font::Title)
                        .bold()
                        .align(TextAlign::Center),
                    label("Введите текущий пароль и новый надёжный пароль (12–15 символов, латинские заглавные и строчные буквы, цифры, спецсимволы)")
                        .font(Font::Subheadline)
                        .secondary()
                        .align(TextAlign::Center)
                        .padding(Insets { top: 6.0, leading: 24.0, bottom: 16.0, trailing: 24.0 }),
                ))
                .grow_w(),

                when(
                    move || !success_msg.get().is_empty(),
                    move || {
                        label(move || success_msg.get())
                            .font(Font::Callout)
                            .color(Color::hex(0x34C759))
                            .align(TextAlign::Center)
                            .padding(Insets { top: 8.0, leading: 14.0, bottom: 8.0, trailing: 14.0 })
                            .background(Color::hex(0x34C759).with_alpha(0.12))
                            .corner_radius(12.0)
                            .padding(Insets { top: 0.0, leading: 20.0, bottom: 12.0, trailing: 20.0 })
                            .grow_w()
                            .any()
                    },
                ),

                when(
                    move || {
                        let local = error_local.get();
                        let state_err = state.error_msg.get();
                        !local.is_empty() || !state_err.is_empty()
                    },
                    move || {
                        let local = error_local.get();
                        let state_err = state.error_msg.get();
                        let msg = if !local.is_empty() { local } else { state_err };
                        label(msg)
                            .font(Font::Callout)
                            .color(colors::ERROR)
                            .align(TextAlign::Center)
                            .padding(Insets { top: 8.0, leading: 14.0, bottom: 8.0, trailing: 14.0 })
                            .background(colors::ERROR.with_alpha(0.12))
                            .corner_radius(12.0)
                            .padding(Insets { top: 0.0, leading: 20.0, bottom: 12.0, trailing: 20.0 })
                            .grow_w()
                            .any()
                    },
                ),

                column((
                    label("ТЕКУЩИЙ ПАРОЛЬ")
                        .font(Font::Caption2)
                        .bold()
                        .secondary(),
                    text_field(old_password)
                        .secure()
                        .frame(0.0, 44.0)
                        .grow_w(),
                ))
                .spacing(4.0)
                .padding(Insets { top: 10.0, leading: 16.0, bottom: 10.0, trailing: 16.0 })
                .background(move || {
                    if day::dark_mode() {
                        Color::rgba(1.0, 1.0, 1.0, 0.07)
                    } else {
                        Color::rgba(0.0, 0.0, 0.0, 0.04)
                    }
                })
                .corner_radius(16.0)
                .padding(Insets { top: 0.0, leading: 20.0, bottom: 12.0, trailing: 20.0 })
                .grow_w(),

                column((
                    label("НОВЫЙ ПАРОЛЬ")
                        .font(Font::Caption2)
                        .bold()
                        .secondary(),
                    text_field(new_password)
                        .secure()
                        .frame(0.0, 44.0)
                        .grow_w()
                        .on_submit(move || {
                            confirm_focused.set(true);
                        }),
                ))
                .spacing(4.0)
                .padding(Insets { top: 10.0, leading: 16.0, bottom: 10.0, trailing: 16.0 })
                .background(move || {
                    if day::dark_mode() {
                        Color::rgba(1.0, 1.0, 1.0, 0.07)
                    } else {
                        Color::rgba(0.0, 0.0, 0.0, 0.04)
                    }
                })
                .corner_radius(16.0)
                .padding(Insets { top: 0.0, leading: 20.0, bottom: 12.0, trailing: 20.0 })
                .grow_w(),

                column((
                    label("ПОВТОРИТЕ НОВЫЙ ПАРОЛЬ")
                        .font(Font::Caption2)
                        .bold()
                        .secondary(),
                    text_field(confirm_password)
                        .secure()
                        .focused(confirm_focused)
                        .frame(0.0, 44.0)
                        .grow_w(),
                ))
                .spacing(4.0)
                .padding(Insets { top: 10.0, leading: 16.0, bottom: 10.0, trailing: 16.0 })
                .background(move || {
                    if day::dark_mode() {
                        Color::rgba(1.0, 1.0, 1.0, 0.07)
                    } else {
                        Color::rgba(0.0, 0.0, 0.0, 0.04)
                    }
                })
                .corner_radius(16.0)
                .padding(Insets { top: 0.0, leading: 20.0, bottom: 12.0, trailing: 20.0 })
                .grow_w(),

                button(move || {
                    if state.loading.get() {
                        "Сохранение…"
                    } else {
                        "Сохранить новый пароль"
                    }
                })
                .prominent()
                .frame(0.0, 50.0)
                .grow_w()
                .corner_radius(25.0)
                .action(move || {
                    if state.loading.get() { return; }
                    let op = old_password.get_untracked();
                    let np = new_password.get_untracked();
                    let cp = confirm_password.get_untracked();

                    if op.is_empty() {
                        error_local.set("Введите текущий пароль".into());
                        return;
                    }
                    if np.is_empty() {
                        error_local.set("Введите новый пароль".into());
                        return;
                    }
                    if np != cp {
                        error_local.set("Пароли не совпадают. Проверьте правильность ввода".into());
                        return;
                    }
                    if let Err(e) = eschool_api::auth::validate_parent_password(&np) {
                        error_local.set(e);
                        return;
                    }

                    error_local.set(String::new());
                    state.error_msg.set(String::new());
                    crate::shared::keyboard::dismiss();
                    crate::shared::haptics::pop();

                    let op_c = op.clone();
                    let np_c = np.clone();
                    let set_open = open.setter();
                    let set_success = success_msg.setter();
                    features::auth::change_logged_in_parent_password(
                        state,
                        &op_c,
                        &np_c,
                        move || {
                            crate::shared::haptics::pop();
                            set_success.set("Пароль успешно обновлён!".into());
                            set_open.set(false);
                        },
                    );
                })
                .padding(Insets { top: 8.0, leading: 20.0, bottom: 12.0, trailing: 20.0 })
                .grow_w(),

                button("Отмена")
                    .action(move || {
                        crate::shared::haptics::tick();
                        crate::shared::keyboard::dismiss();
                        open.set(false);
                    })
                    .padding(Insets { top: 0.0, leading: 24.0, bottom: 16.0, trailing: 24.0 }),

                when(
                    move || kb_h.get() > 0.0,
                    move || spacer().frame(0.0, kb_h.get() + 20.0).any(),
                ),
            ))
            .grow_w()
        )
        .on_tap(move || {
            crate::shared::keyboard::dismiss();
        })
        .grow(),
    ))
    .background(move || {
        if day::dark_mode() {
            Color::hex(0x000000)
        } else {
            Color::hex(0xF2F2F7)
        }
    })
    .grow()
}
