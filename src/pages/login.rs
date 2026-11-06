use crate::app::AppState;
use crate::features;
use crate::shared::colors;
use crate::shared::layout_diag::{debug_log_button, debug_log_overlay, PosLogExt};
use crate::res;
use day::prelude::*;
#[cfg(target_os = "ios")]
use day_uikit::UiKitExt;

pub fn render() -> impl Piece {
    let state = AppState::ambient();
    let show_logs = Signal::new(false);

    zstack((
        when(
            move || state.is_authenticated.get(),
            move || profile_view(state).animation(AnimSpec::ease_out(350)).any(),
        ),
        when(
            move || !state.is_authenticated.get() && state.setup_new_password_active.get(),
            move || setup_password_form(state, show_logs).animation(AnimSpec::ease_out(350)).any(),
        ),
        when(
            move || !state.is_authenticated.get() && !state.setup_new_password_active.get(),
            move || auth_form(state, show_logs).animation(AnimSpec::ease_out(350)).any(),
        ),
        when(
            move || show_logs.get(),
            move || debug_log_overlay(show_logs).animation(AnimSpec::ease_out(250)).any(),
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

struct AuthErrorInfo {
    code: &'static str,
    title: &'static str,
    message: String,
}

fn parse_auth_error(raw: &str) -> AuthErrorInfo {
    let lower = raw.to_lowercase();
    if lower.contains("введите логин") || lower.contains("введите имя") {
        AuthErrorInfo {
            code: "AUTH_EMPTY_LOGIN",
            title: "Не указан логин",
            message: "Пожалуйста, введите ваш логин или код.".into(),
        }
    } else if lower.contains("введите пароль") {
        AuthErrorInfo {
            code: "AUTH_EMPTY_PASSWORD",
            title: "Не указан пароль",
            message: "Пожалуйста, введите пароль от вашего аккаунта.".into(),
        }
    } else if lower.contains("401") || lower.contains("неверн") || lower.contains("парол") || lower.contains("unauthorized") || lower.contains("invalid") {
        AuthErrorInfo {
            code: "AUTH_INVALID_CREDENTIALS",
            title: "Неверные данные",
            message: "Неверный логин или пароль. Проверьте правильность ввода.".into(),
        }
    } else if lower.contains("403") || lower.contains("заблокир") || lower.contains("forbidden") {
        AuthErrorInfo {
            code: "AUTH_FORBIDDEN",
            title: "Доступ ограничен",
            message: "Доступ к аккаунту заблокирован или запрещён.".into(),
        }
    } else if lower.contains("таймаут") || lower.contains("timeout") || lower.contains("timed out") {
        AuthErrorInfo {
            code: "NET_TIMEOUT",
            title: "Таймаут соединения",
            message: "Сервер электронного дневника не ответил вовремя.".into(),
        }
    } else if lower.contains("error sending request")
        || lower.contains("не удалось подключиться")
        || lower.contains("dns error")
        || lower.contains("connection refused")
        || lower.contains("failed to connect")
    {
        AuthErrorInfo {
            code: "NET_NO_CONNECTION",
            title: "Ошибка сети",
            message: "Не удалось подключиться. Проверьте интернет или отключите VPN.".into(),
        }
    } else if lower.contains("500") || lower.contains("502") || lower.contains("503") || lower.contains("504") || lower.contains("internal") {
        AuthErrorInfo {
            code: "SRV_UNAVAILABLE",
            title: "Сбой сервера",
            message: "На сервере дневника проводятся работы. Попробуйте позже.".into(),
        }
    } else {
        AuthErrorInfo {
            code: "AUTH_FAILED",
            title: "Ошибка входа",
            message: if raw.trim().is_empty() {
                "Не удалось выполнить вход. Попробуйте снова.".into()
            } else if raw.len() <= 60 && !raw.contains('{') && !raw.contains("http") {
                raw.to_string()
            } else {
                "Произошла ошибка при попытке входа. Попробуйте снова.".into()
            },
        }
    }
}

fn error_banner(error_local: Signal<String>, state: AppState) -> impl Piece {
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
            let raw_msg = if !local_err.is_empty() { local_err } else { state_err };
            let err = parse_auth_error(&raw_msg);

            column((
                row((
                    label(err.title)
                        .font(Font::Headline)
                        .color(colors::ERROR),
                    spacer().grow(),
                    label(err.code)
                        .font(Font::Caption2)
                        .bold()
                        .color(colors::ERROR)
                        .padding(Insets { top: 3.0, leading: 7.0, bottom: 3.0, trailing: 7.0 })
                        .background(colors::ERROR.with_alpha(0.14))
                        .corner_radius(6.0),
                ))
                .align(VAlign::Center)
                .grow_w(),

                spacer().height(4.0),

                label(err.message)
                    .font(Font::Subheadline)
                    .secondary()
                    .align(TextAlign::Leading),
            ))
            .padding(Insets { top: 12.0, leading: 16.0, bottom: 12.0, trailing: 16.0 })
            .background(colors::ERROR.with_alpha(0.08))
            .corner_radius(14.0)
            .padding(Insets { top: 0.0, leading: 20.0, bottom: 12.0, trailing: 20.0 })
            .grow_w()
            .any()
        },
    )
}

#[cfg(target_os = "ios")]
unsafe extern "C" {
    fn eschool_style_liquid_glass_button(button_ptr: *mut std::ffi::c_void);
    fn eschool_glue_button_to_keyboard(button_ptr: *mut std::ffi::c_void);
}

fn liquid_glass_back_button(action: impl Fn() + 'static) -> impl Piece {
    let act = std::rc::Rc::new(action);
    let act_clone = act.clone();

    #[cfg(target_os = "ios")]
    {
        button("")
            .frame(42.0, 42.0)
            .action(move || {
                act_clone();
            })
            .uikit(|view, _class, _mtm| {
                let ptr = (&**view as *const objc2_ui_kit::UIView) as *mut std::ffi::c_void;
                unsafe {
                    eschool_style_liquid_glass_button(ptr);
                }
            })
            .id("liquid-glass-back-arrow")
    }

    #[cfg(not(target_os = "ios"))]
    {
        button("‹")
            .frame(42.0, 42.0)
            .corner_radius(21.0)
            .glass_radius(21.0)
            .background(move || {
                if day::dark_mode() {
                    Color::rgba(1.0, 1.0, 1.0, 0.16)
                } else {
                    Color::rgba(0.0, 0.0, 0.0, 0.08)
                }
            })
            .action(move || {
                act_clone();
            })
            .id("liquid-glass-back-arrow")
    }
}

fn auth_action_button(
    title: impl Fn() -> &'static str + 'static,
    action: impl Fn() + 'static,
) -> impl Piece {
    let btn = button(title)
        .prominent()
        .frame(0.0, 50.0)
        .grow_w()
        .action(action)
        .id("login-btn")
        .log_pos("Auth:ActionButton")
        .padding(Insets { top: 0.0, leading: 24.0, bottom: 12.0, trailing: 24.0 });

    #[cfg(target_os = "ios")]
    {
        btn.uikit(|view, _class, _mtm| {
            let ptr = (&**view as *const objc2_ui_kit::UIView) as *mut std::ffi::c_void;
            unsafe {
                eschool_glue_button_to_keyboard(ptr);
            }
        })
    }

    #[cfg(not(target_os = "ios"))]
    {
        btn
    }
}

fn prompt_login_confirmation(
    username: String,
    on_confirm: impl Fn() + 'static,
    on_edit: impl Fn() + 'static,
) {
    #[cfg(target_os = "ios")]
    {
        use objc2::MainThreadMarker;
        use objc2_foundation::NSString;
        use objc2_ui_kit::{
            UIApplication, UIAlertAction, UIAlertActionStyle, UIAlertController, UIAlertControllerStyle,
        };
        use std::ptr::NonNull;

        let Some(mtm) = MainThreadMarker::new() else { return };
        let app = UIApplication::sharedApplication(mtm);
        let Some(window) = app.keyWindow() else { return };
        let Some(root_vc) = window.rootViewController() else { return };

        let mut top = root_vc;
        while let Some(presented) = top.presentedViewController() {
            top = presented;
        }

        let title = NSString::from_str(&username);
        let message = NSString::from_str("Это верный логин?");

        let alert = unsafe {
            UIAlertController::alertControllerWithTitle_message_preferredStyle(
                Some(&title),
                Some(&message),
                UIAlertControllerStyle::Alert,
                mtm,
            )
        };

        let on_edit_rc = std::rc::Rc::new(on_edit);
        let on_edit_clone = on_edit_rc.clone();
        let edit_action = unsafe {
            let title_edit = NSString::from_str("Изменить");
            let handler = block2::RcBlock::new(move |_: NonNull<UIAlertAction>| {
                on_edit_clone();
            });
            UIAlertAction::actionWithTitle_style_handler(
                Some(&title_edit),
                UIAlertActionStyle::Cancel,
                Some(&handler),
                mtm,
            )
        };

        let on_confirm_rc = std::rc::Rc::new(on_confirm);
        let confirm_action = unsafe {
            let title_yes = NSString::from_str("Да");
            let handler = block2::RcBlock::new(move |_: NonNull<UIAlertAction>| {
                on_confirm_rc();
            });
            UIAlertAction::actionWithTitle_style_handler(
                Some(&title_yes),
                UIAlertActionStyle::Default,
                Some(&handler),
                mtm,
            )
        };

        unsafe {
            alert.addAction(&edit_action);
            alert.addAction(&confirm_action);
            top.presentViewController_animated_completion(&alert, true, None);
        }
    }

    #[cfg(not(target_os = "ios"))]
    {
        let _ = username;
        let _ = on_edit;
        on_confirm();
    }
}

fn auth_form(state: AppState, _show_logs: Signal<bool>) -> impl Piece {
    let step = Signal::new(0u8); // 0 = Intro, 1 = Login/Username, 2 = Password
    let username = Signal::new(String::new());
    let password = Signal::new(String::new());
    let username_focused = Signal::new(false);
    let password_focused = Signal::new(false);
    let remember = Signal::new(true);
    let error_local = Signal::new(String::new());
    let _kb_h = crate::shared::keyboard::keyboard_height_signal();

    column((
            // Step 0: Intro screen (Telegram style: Logo + title + subtitle + spacer)
            when(
                move || step.get() == 0,
                move || {
                    column((
                        spacer().grow(),
                        row((
                            spacer().grow(),
                            crate::widgets::lottie_view::login_animation_view_sized(130.0),
                            spacer().grow(),
                        )),
                        spacer().height(24.0),
                        label("ESchool")
                            .font(Font::LargeTitle)
                            .bold()
                            .align(TextAlign::Center),
                        spacer().height(8.0),
                        label("Электронный дневник")
                            .font(Font::Headline)
                            .secondary()
                            .align(TextAlign::Center),
                        spacer().height(12.0),
                        label("Быстрый и удобный доступ к расписанию,\nоценкам и домашним заданиям")
                            .font(Font::Subheadline)
                            .secondary()
                            .align(TextAlign::Center)
                            .padding(Insets { top: 0.0, leading: 32.0, bottom: 0.0, trailing: 32.0 }),
                        spacer().grow(),
                    ))
                    .grow_w()
                    .any()
                },
            ),

            // Step 1: Login / Username screen
            when(
                move || step.get() == 1,
                move || {
                    let top_inset = (crate::shared::keyboard::get_safe_area_top()).max(24.0) + 8.0;
                    column((
                        spacer().height(top_inset),
                        row((
                            liquid_glass_back_button(move || {
                                crate::shared::haptics::tick();
                                crate::shared::keyboard::dismiss();
                                username_focused.set(false);
                                with_animation(AnimSpec::ease_out(250), move || step.set(0));
                            }),
                            spacer().grow(),
                        ))
                        .padding(Insets { top: 0.0, leading: 20.0, bottom: 6.0, trailing: 20.0 }),

                        column((
                            label("Вход в аккаунт")
                                .font(Font::Title)
                                .bold()
                                .align(TextAlign::Center),
                            label("Введите логин ученика или пригласительный код родителя для доступа к дневнику")
                                .font(Font::Subheadline)
                                .secondary()
                                .align(TextAlign::Center)
                                .padding(Insets { top: 4.0, leading: 24.0, bottom: 12.0, trailing: 24.0 }),
                        ))
                        .grow_w(),

                        error_banner(error_local, state),

                        column((
                            label("ЛОГИН ИЛИ ПРИГЛАСИТЕЛЬНЫЙ КОД")
                                .font(Font::Caption2)
                                .bold()
                                .secondary(),
                            zstack((
                                // Password field placed underneath with real 44pt height and non-zero alpha,
                                // allowing iOS AutoFill to discover both credentials on Step 1 and dual-fill
                                // with a SINGLE Face ID scan!
                                text_field(password)
                                    .secure()
                                    .frame(0.0, 44.0)
                                    .grow_w()
                                    .opacity(0.02),
                                text_field(username)
                                    .focused(username_focused)
                                    .frame(0.0, 44.0)
                                    .grow_w()
                                    .log_pos("Auth:UsernameField")
                                    .on_submit(move || {
                                        let u = username.get_untracked();
                                        if u.trim().is_empty() {
                                            error_local.set("Введите логин".into());
                                        } else {
                                            error_local.set(String::new());
                                            state.error_msg.set(String::new());
                                            crate::shared::haptics::tick();
                                            prompt_login_confirmation(
                                                u.trim().to_string(),
                                                move || {
                                                    crate::shared::haptics::tick();
                                                    username_focused.set(false);
                                                    password_focused.set(true);
                                                    with_animation(AnimSpec::ease_out(250), move || step.set(2));
                                                },
                                                move || {
                                                    crate::shared::haptics::tick();
                                                    username_focused.set(true);
                                                },
                                            );
                                        }
                                    }),
                            )),
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
                        .padding(Insets { top: 0.0, leading: 20.0, bottom: 14.0, trailing: 20.0 })
                        .grow_w(),

                        // Remember session toggle
                        row((
                            label("Сохранить сессию на устройстве")
                                .font(Font::Subheadline)
                                .grow(),
                            toggle(remember),
                        ))
                        .align(VAlign::Center)
                        .padding(Insets { top: 0.0, leading: 24.0, bottom: 12.0, trailing: 24.0 })
                        .grow_w(),
                    ))
                    .grow_w()
                    .any()
                },
            ),

            // Step 2: Password screen
            when(
                move || step.get() == 2,
                move || {
                    let top_inset = (crate::shared::keyboard::get_safe_area_top()).max(24.0) + 8.0;
                    column((
                        spacer().height(top_inset),
                        // Real Apple Liquid Glass circular back bubble with native SF Symbol chevron
                        row((
                            liquid_glass_back_button(move || {
                                crate::shared::haptics::tick();
                                error_local.set(String::new());
                                password_focused.set(false);
                                username_focused.set(true);
                                with_animation(AnimSpec::ease_out(250), move || step.set(1));
                            }),
                            spacer().grow(),
                        ))
                        .padding(Insets { top: 0.0, leading: 20.0, bottom: 6.0, trailing: 20.0 }),

                        column((
                            label("Введите пароль")
                                .font(Font::Title)
                                .bold()
                                .align(TextAlign::Center),
                            label("Введите пароль для входа в дневник")
                                .font(Font::Subheadline)
                                .secondary()
                                .align(TextAlign::Center)
                                .padding(Insets { top: 4.0, leading: 24.0, bottom: 12.0, trailing: 24.0 }),
                        ))
                        .grow_w(),

                        error_banner(error_local, state),

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
                                .log_pos("Auth:PasswordField")
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
                        .padding(Insets { top: 0.0, leading: 20.0, bottom: 10.0, trailing: 20.0 })
                        .grow_w(),
                    ))
                    .grow_w()
                    .any()
                },
            ),

            spacer().grow(),

            // Telegram-style solid rounded action button pinned at bottom, lifts directly above keyboard
            auth_action_button(
                move || {
                    if state.loading.get() {
                        "Вход в систему…"
                    } else {
                        match step.get() {
                            0 => "Начать",
                            1 => "Продолжить",
                            _ => "Войти",
                        }
                    }
                },
                move || {
                    if state.loading.get() { return; }
                    match step.get_untracked() {
                        0 => {
                            crate::shared::haptics::tick();
                            error_local.set(String::new());
                            state.error_msg.set(String::new());
                            username_focused.set(true);
                            with_animation(AnimSpec::ease_out(250), move || step.set(1));
                        }
                        1 => {
                            let u = username.get_untracked();
                            if u.trim().is_empty() {
                                error_local.set("Введите логин".into());
                            } else {
                                error_local.set(String::new());
                                state.error_msg.set(String::new());
                                crate::shared::haptics::tick();
                                prompt_login_confirmation(
                                    u.trim().to_string(),
                                    move || {
                                        crate::shared::haptics::tick();
                                        username_focused.set(false);
                                        password_focused.set(true);
                                        with_animation(AnimSpec::ease_out(250), move || step.set(2));
                                    },
                                    move || {
                                        crate::shared::haptics::tick();
                                        username_focused.set(true);
                                    },
                                );
                            }
                        }
                        _ => {
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
                        }
                    }
                },
            ),
    ))
    .grow()
    .on_tap(move || {
        crate::shared::keyboard::dismiss();
    })
}

fn setup_password_form(state: AppState, show_logs: Signal<bool>) -> impl Piece {
    let new_password = Signal::new(String::new());
    let confirm_password = Signal::new(String::new());
    let confirm_focused = Signal::new(false);
    let error_local = Signal::new(String::new());
    let remember = Signal::new(true);
    let kb_h = crate::shared::keyboard::keyboard_height_signal();

    column((
        debug_log_button(show_logs),
        spacer().height(16.0),

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
            .log_pos("Setup:TitleColumn")
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
                    .log_pos("Setup:NewPasswordField")
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
                    .log_pos("Setup:ConfirmPasswordField")
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

        spacer().grow(),

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
        .log_pos("Setup:SubmitButton")
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
        .padding(Insets { top: 0.0, leading: 24.0, bottom: 8.0, trailing: 24.0 })
        .grow_w(),

        button("← Назад к входу")
            .log_pos("Setup:BackButton")
            .action(move || {
                crate::shared::haptics::tick();
                crate::shared::keyboard::dismiss();
                state.setup_new_password_active.set(false);
                state.setup_temp_password_data.set(None);
            })
            .padding(Insets { top: 0.0, leading: 24.0, bottom: 8.0, trailing: 24.0 }),

        dyn_piece(
            move || {
                let h = kb_h.get();
                if h > 0.0 {
                    let safe_b = crate::shared::keyboard::get_safe_area_bottom();
                    (h - safe_b).max(0.0) + 12.0
                } else {
                    4.0
                }
            },
            move |lift| spacer().height(lift).any(),
        ),
    ))
    .grow()
    .on_tap(move || {
        crate::shared::keyboard::dismiss();
    })
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
                        .grow_w()
                        .log_pos("ChangePwd:OldPasswordField"),
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
                        .log_pos("ChangePwd:NewPasswordField")
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
                        .grow_w()
                        .log_pos("ChangePwd:ConfirmPasswordField"),
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
                .log_pos("ChangePwd:SubmitButton")
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

                dyn_piece(
                    move || {
                        let h = kb_h.get();
                        if h > 0.0 {
                            let safe_b = crate::shared::keyboard::get_safe_area_bottom();
                            (h - safe_b).max(0.0) + 20.0
                        } else {
                            0.0
                        }
                    },
                    move |lift| spacer().frame(0.0, lift).any(),
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
