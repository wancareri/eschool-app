use crate::app::AppState;
use crate::features;
use crate::shared::colors;
use crate::shared::layout_diag::{debug_log_button, debug_log_overlay, PosLogExt};
use crate::res;
use day::prelude::*;

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
            move || nav_stack(Signal::new(Vec::<String>::new()), setup_password_form(state, show_logs))
                .title("Новый пароль")
                .toolbar(|| auth_toolbar("setup"))
                .grow()
                .any(),
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
    .background(move || {
        if day::dark_mode() {
            Color::hex(0x000000)
        } else {
            Color::hex(0xFFFFFF)
        }
    })
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
                                let auth = get_auth_state();
                                auth.path.set(Vec::new());
                                auth.username.set(String::new());
                                auth.password.set(String::new());
                                auth.error_local.set(String::new());
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

#[allow(dead_code)]
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


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AuthRoute {
    Login,
    Password,
}

impl Route for AuthRoute {
    fn key(&self) -> String {
        match self {
            AuthRoute::Login => "login".into(),
            AuthRoute::Password => "password".into(),
        }
    }
    fn from_key(key: &str) -> Option<Self> {
        match key {
            "login" => Some(AuthRoute::Login),
            "password" => Some(AuthRoute::Password),
            _ => None,
        }
    }
    fn title(&self) -> String {
        match self {
            AuthRoute::Login => "Вход в аккаунт".into(),
            AuthRoute::Password => "Введите пароль".into(),
        }
    }
}

fn auth_action_button(
    title: impl Fn() -> &'static str + 'static,
    action: impl Fn() + 'static,
) -> impl Piece {
    button(title)
        .prominent()
        .height(52.0)
        .grow_w()
        .action(action)
        .id("login-btn")
        .log_pos("Auth:ActionButton")
        .padding(Insets { top: 0.0, leading: 20.0, bottom: 8.0, trailing: 20.0 })
}

#[cfg(target_os = "ios")]
pub fn show_theme_sheet() {
    use objc2::MainThreadMarker;
    use objc2_foundation::NSString;
    use objc2_ui_kit::{
        UIAlertAction, UIAlertActionStyle, UIAlertController, UIAlertControllerStyle,
        UIApplication,
    };
    use block2::RcBlock;
    use core::ptr::NonNull;

    let Some(mtm) = MainThreadMarker::new() else { return; };
    let app = UIApplication::sharedApplication(mtm);
    #[allow(deprecated)]
    let Some(window) = app.keyWindow() else { return; };
    let Some(mut top_vc) = window.rootViewController() else { return; };
    while let Some(presented) = top_vc.presentedViewController() {
        top_vc = presented;
    }

    let alert = UIAlertController::alertControllerWithTitle_message_preferredStyle(
        Some(&NSString::from_str("Оформление")),
        None,
        UIAlertControllerStyle::ActionSheet,
        mtm,
    );

    let h_sys = RcBlock::new(move |_action: NonNull<UIAlertAction>| {
        day::prefs::remove(crate::THEME_KEY);
        day::set_appearance(None);
        crate::shared::haptics::tick();
        restore_auth_focus_after_sheet();
    });
    let a_sys = UIAlertAction::actionWithTitle_style_handler(
        Some(&NSString::from_str("Системная")),
        UIAlertActionStyle::Default,
        Some(&*h_sys),
        mtm,
    );

    let h_light = RcBlock::new(move |_action: NonNull<UIAlertAction>| {
        day::prefs::set(crate::THEME_KEY, "light");
        day::set_appearance(Some(false));
        crate::shared::haptics::tick();
        restore_auth_focus_after_sheet();
    });
    let a_light = UIAlertAction::actionWithTitle_style_handler(
        Some(&NSString::from_str("Светлая")),
        UIAlertActionStyle::Default,
        Some(&*h_light),
        mtm,
    );

    let h_dark = RcBlock::new(move |_action: NonNull<UIAlertAction>| {
        day::prefs::set(crate::THEME_KEY, "dark");
        day::set_appearance(Some(true));
        crate::shared::haptics::tick();
        restore_auth_focus_after_sheet();
    });
    let a_dark = UIAlertAction::actionWithTitle_style_handler(
        Some(&NSString::from_str("Тёмная")),
        UIAlertActionStyle::Default,
        Some(&*h_dark),
        mtm,
    );

    let h_cancel = RcBlock::new(move |_action: NonNull<UIAlertAction>| {
        restore_auth_focus_after_sheet();
    });
    let a_cancel = UIAlertAction::actionWithTitle_style_handler(
        Some(&NSString::from_str("Отмена")),
        UIAlertActionStyle::Cancel,
        Some(&*h_cancel),
        mtm,
    );

    alert.addAction(&a_sys);
    alert.addAction(&a_light);
    alert.addAction(&a_dark);
    alert.addAction(&a_cancel);

    if let Some(popover) = alert.popoverPresentationController() {
        popover.setSourceView(Some(&window));
        let bounds = window.bounds();
        popover.setSourceRect(objc2_core_foundation::CGRect {
            origin: objc2_core_foundation::CGPoint { x: bounds.size.width - 60.0, y: 60.0 },
            size: objc2_core_foundation::CGSize { width: 1.0, height: 1.0 },
        });
    }

    top_vc.presentViewController_animated_completion(&alert, true, None);
}

#[cfg(not(target_os = "ios"))]
pub fn show_theme_sheet() {
    let cur = day::prefs::get(crate::THEME_KEY);
    match cur.as_deref() {
        None => {
            day::prefs::set(crate::THEME_KEY, "light");
            day::set_appearance(Some(false));
        }
        Some("light") => {
            day::prefs::set(crate::THEME_KEY, "dark");
            day::set_appearance(Some(true));
        }
        _ => {
            day::prefs::remove(crate::THEME_KEY);
            day::set_appearance(None);
        }
    }
}

#[cfg(target_os = "ios")]
pub fn show_lang_sheet() {
    use objc2::MainThreadMarker;
    use objc2_foundation::NSString;
    use objc2_ui_kit::{
        UIAlertAction, UIAlertActionStyle, UIAlertController, UIAlertControllerStyle,
        UIApplication,
    };
    use block2::RcBlock;
    use core::ptr::NonNull;

    let Some(mtm) = MainThreadMarker::new() else { return; };
    let app = UIApplication::sharedApplication(mtm);
    #[allow(deprecated)]
    let Some(window) = app.keyWindow() else { return; };
    let Some(mut top_vc) = window.rootViewController() else { return; };
    while let Some(presented) = top_vc.presentedViewController() {
        top_vc = presented;
    }

    let alert = UIAlertController::alertControllerWithTitle_message_preferredStyle(
        Some(&NSString::from_str("Язык")),
        None,
        UIAlertControllerStyle::ActionSheet,
        mtm,
    );

    let h_ru = RcBlock::new(move |_action: NonNull<UIAlertAction>| {
        day::prefs::set(crate::LOCALE_KEY, "ru");
        set_locale("ru");
        crate::shared::haptics::tick();
        restore_auth_focus_after_sheet();
    });
    let a_ru = UIAlertAction::actionWithTitle_style_handler(
        Some(&NSString::from_str("Русский (RU)")),
        UIAlertActionStyle::Default,
        Some(&*h_ru),
        mtm,
    );

    let h_be = RcBlock::new(move |_action: NonNull<UIAlertAction>| {
        day::prefs::set(crate::LOCALE_KEY, "be");
        set_locale("be");
        crate::shared::haptics::tick();
        restore_auth_focus_after_sheet();
    });
    let a_be = UIAlertAction::actionWithTitle_style_handler(
        Some(&NSString::from_str("Беларуская (BE)")),
        UIAlertActionStyle::Default,
        Some(&*h_be),
        mtm,
    );

    let h_en = RcBlock::new(move |_action: NonNull<UIAlertAction>| {
        day::prefs::set(crate::LOCALE_KEY, "en");
        set_locale("en");
        crate::shared::haptics::tick();
        restore_auth_focus_after_sheet();
    });
    let a_en = UIAlertAction::actionWithTitle_style_handler(
        Some(&NSString::from_str("English (EN)")),
        UIAlertActionStyle::Default,
        Some(&*h_en),
        mtm,
    );

    let h_cancel = RcBlock::new(move |_action: NonNull<UIAlertAction>| {
        restore_auth_focus_after_sheet();
    });
    let a_cancel = UIAlertAction::actionWithTitle_style_handler(
        Some(&NSString::from_str("Отмена")),
        UIAlertActionStyle::Cancel,
        Some(&*h_cancel),
        mtm,
    );

    alert.addAction(&a_ru);
    alert.addAction(&a_be);
    alert.addAction(&a_en);
    alert.addAction(&a_cancel);

    if let Some(popover) = alert.popoverPresentationController() {
        popover.setSourceView(Some(&window));
        let bounds = window.bounds();
        popover.setSourceRect(objc2_core_foundation::CGRect {
            origin: objc2_core_foundation::CGPoint { x: bounds.size.width - 120.0, y: 60.0 },
            size: objc2_core_foundation::CGSize { width: 1.0, height: 1.0 },
        });
    }

    top_vc.presentViewController_animated_completion(&alert, true, None);
}

#[cfg(not(target_os = "ios"))]
pub fn show_lang_sheet() {
    let cur = day::locale().get_untracked();
    if cur.starts_with("ru") {
        day::prefs::set(crate::LOCALE_KEY, "be");
        set_locale("be");
    } else if cur.starts_with("be") {
        day::prefs::set(crate::LOCALE_KEY, "en");
        set_locale("en");
    } else {
        day::prefs::set(crate::LOCALE_KEY, "ru");
        set_locale("ru");
    }
}

pub fn glass_lang_button() -> impl Piece {
    let lang_str = move || {
        let cur = day::locale().get();
        let lower = cur.to_lowercase();
        if lower.starts_with("be") {
            "BE"
        } else if lower.starts_with("en") {
            "EN"
        } else {
            "RU"
        }
    };

    row((
        vector(res::vectors::globe)
            .frame(16.0, 16.0)
            .tint(move || {
                if day::dark_mode() {
                    Color::hex(0xFFFFFF)
                } else {
                    Color::hex(0x000000)
                }
            }),
        spacer().width(6.0),
        label(lang_str)
            .font(Font::Subheadline)
            .bold(),
    ))
    .align(VAlign::Center)
    .padding(Insets { top: 7.0, leading: 12.0, bottom: 7.0, trailing: 12.0 })
    .background(move || {
        if day::dark_mode() {
            Color::rgba(1.0, 1.0, 1.0, 0.12)
        } else {
            Color::rgba(0.0, 0.0, 0.0, 0.07)
        }
    })
    .glass_radius(18.0)
    .on_tap(move || {
        crate::shared::haptics::tick();
        show_lang_sheet();
    })
}

pub fn glass_theme_button() -> impl Piece {
    let is_dark = move || {
        let cur = day::prefs::get(crate::THEME_KEY);
        match cur.as_deref() {
            Some("light") => false,
            Some("dark") => true,
            _ => day::dark_mode(),
        }
    };

    row((
        when(is_dark, || {
            vector(res::vectors::moon)
                .frame(18.0, 18.0)
                .tint(Color::hex(0xFFFFFF))
                .any()
        })
        .otherwise(|| {
            vector(res::vectors::sun)
                .frame(18.0, 18.0)
                .tint(Color::hex(0x000000))
                .any()
        }),
    ))
    .align(VAlign::Center)
    .frame(36.0, 36.0)
    .background(move || {
        if day::dark_mode() {
            Color::rgba(1.0, 1.0, 1.0, 0.12)
        } else {
            Color::rgba(0.0, 0.0, 0.0, 0.07)
        }
    })
    .glass_radius(18.0)
    .on_tap(move || {
        crate::shared::haptics::tick();
        show_theme_sheet();
    })
}

fn auth_toolbar(page: &str) -> Vec<ToolbarEntry> {
    let language = day::locale().get().to_lowercase();
    let code = if language.starts_with("be") { "BE" }
        else if language.starts_with("en") { "EN" } else { "RU" };
    vec![
        toolbar_button(format!("auth-{page}-language"), code)
            .tooltip(res::str::auth_language())
            .placement(ToolbarPlacement::Primary)
            .action(show_lang_sheet),
        toolbar_button(format!("auth-{page}-theme"), res::str::auth_appearance())
            .icon(if day::dark_mode() { Symbol::Dark } else { Symbol::Light })
            .placement(ToolbarPlacement::Primary)
            .action(show_theme_sheet),
    ]
}

fn intro_view(on_start: impl Fn() + 'static) -> impl Piece {
    let piece = column((
        spacer().height(16.0),
        label("ESchool")
            .font(Font::LargeTitle)
            .bold()
            .align(TextAlign::Center),
        spacer().height(6.0),
        label("Электронный дневник")
            .font(Font::Headline)
            .secondary()
            .align(TextAlign::Center),
        spacer().height(10.0),
        label("Быстрый и удобный доступ к расписанию,\nоценкам и домашним заданиям")
            .font(Font::Subheadline)
            .secondary()
            .align(TextAlign::Center)
            .padding(Insets { top: 0.0, leading: 32.0, bottom: 0.0, trailing: 32.0 }),
        spacer().grow(),
        row((
            spacer().grow(),
            crate::widgets::lottie_view::duck_intro_animation(240.0),
            spacer().grow(),
        )),
        spacer().grow(),
        auth_action_button(|| "Начать", on_start),
        spacer().height(8.0),
    ))
    .grow()
    .background(move || {
        if day::dark_mode() {
            Color::hex(0x000000)
        } else {
            Color::hex(0xFFFFFF)
        }
    });

    #[cfg(target_os = "ios")]
    let piece = {
        use day_uikit::UiKitExt;
        piece.uikit(move |view, _class, _mtm| {
            view.setBackgroundColor(Some(&objc2_ui_kit::UIColor::systemBackgroundColor()));
            if let Some(w) = view.window() {
                w.setBackgroundColor(Some(&objc2_ui_kit::UIColor::systemBackgroundColor()));
            }
        })
    };

    piece
}

fn username_step_view(
    username: Signal<String>,
    _password: Signal<String>,
    remember: Signal<bool>,
    username_focused: Signal<bool>,
    error_local: Signal<String>,
    state: AppState,
    on_submit: impl Fn() + Clone + 'static,
) -> impl Piece {
    let sticker = row((
        spacer().grow(),
        crate::widgets::lottie_view::duck_login_animation(230.0),
        spacer().grow(),
    ));

    #[cfg(target_os = "ios")]
    let sticker = {
        use day_uikit::UiKitExt;
        sticker.uikit(|view, _class, _mtm| {
            crate::shared::keyboard::register_sticker_view(view);
        })
    };

    let on_sub = on_submit.clone();
    let input_field = text_field(username)
        .placeholder("Логин или код")
        .focused(username_focused)
        .grow_w()
        .log_pos("Auth:UsernameField")
        .on_submit(on_sub);

    let input_card = row((
        vector(res::vectors::tab_login)
            .frame(20.0, 20.0)
            .tint(move || {
                if username_focused.get() {
                    Color::hex(state.accent_color.get())
                } else {
                    Color::hex(0x8E8E93)
                }
            }),
        spacer().width(12.0),
        input_field,
    ))
    .align(VAlign::Center)
    .padding(Insets { top: 12.0, leading: 16.0, bottom: 12.0, trailing: 16.0 })
    .background(move || {
        if day::dark_mode() {
            if username_focused.get() {
                Color::rgba(1.0, 1.0, 1.0, 0.12)
            } else {
                Color::rgba(1.0, 1.0, 1.0, 0.07)
            }
        } else {
            if username_focused.get() {
                Color::rgba(0.0, 0.0, 0.0, 0.06)
            } else {
                Color::rgba(0.0, 0.0, 0.0, 0.035)
            }
        }
    })
    .glass_radius(16.0);

    let content = column((
        input_card
            .padding(Insets { top: 0.0, leading: 20.0, bottom: 12.0, trailing: 20.0 })
            .grow_w(),

        // Minimalist native session remember toggle
        row((
            label("Запомнить вход")
                .font(Font::Subheadline)
                .secondary()
                .grow(),
            toggle(remember),
        ))
        .align(VAlign::Center)
        .padding(Insets { top: 0.0, leading: 24.0, bottom: 8.0, trailing: 24.0 })
        .grow_w(),
    ));

    let err_display = when(
        move || !error_local.get().is_empty() || !state.error_msg.get().is_empty(),
        move || {
            let local_err = error_local.get();
            let state_err = state.error_msg.get();
            let (code, msg) = if !local_err.is_empty() {
                let err = parse_auth_error(&local_err);
                (err.code, err.message)
            } else {
                let err = parse_auth_error(&state_err);
                (err.code, err.message)
            };
            column((
                label(msg)
                    .font(Font::Subheadline)
                    .color(colors::ERROR)
                    .align(TextAlign::Center),
                spacer().height(2.0),
                label(format!("Код ошибки: [{code}]"))
                    .font(Font::Caption2)
                    .bold()
                    .color(colors::ERROR.with_alpha(0.8))
                    .align(TextAlign::Center),
            ))
            .spacing(2.0)
            .padding(Insets { top: 0.0, leading: 24.0, bottom: 8.0, trailing: 24.0 })
            .grow_w()
            .any()
        },
    );

    let bottom_section = column((
        content,
        err_display,
        auth_action_button(|| "Продолжить", on_submit),
        spacer().height(8.0),
    ));

    #[cfg(target_os = "ios")]
    let bottom_section = {
        use day_uikit::UiKitExt;
        bottom_section.uikit(|view, _class, _mtm| {
            crate::shared::keyboard::register_content_view(view);
        })
    };

    let piece = column((
        spacer().grow(),
        sticker,
        spacer().grow(),
        bottom_section,
    ))
    .grow()
    .background(move || {
        if day::dark_mode() {
            Color::hex(0x000000)
        } else {
            Color::hex(0xFFFFFF)
        }
    });

    #[cfg(target_os = "ios")]
    let piece = {
        use day_uikit::UiKitExt;
        piece.uikit(move |view, _class, _mtm| {
            view.setBackgroundColor(Some(&objc2_ui_kit::UIColor::systemBackgroundColor()));
            if let Some(w) = view.window() {
                w.setBackgroundColor(Some(&objc2_ui_kit::UIColor::systemBackgroundColor()));
            }
        })
    };

    piece
}

fn password_step_view(
    _username: Signal<String>,
    password: Signal<String>,
    password_focused: Signal<bool>,
    error_local: Signal<String>,
    state: AppState,
    on_submit: impl Fn() + Clone + 'static,
) -> impl Piece {
    let sticker = row((
        spacer().grow(),
        crate::widgets::lottie_view::duck_password_animation(230.0),
        spacer().grow(),
    ));

    #[cfg(target_os = "ios")]
    let sticker = {
        use day_uikit::UiKitExt;
        sticker.uikit(|view, _class, _mtm| {
            crate::shared::keyboard::register_sticker_view(view);
        })
    };

    let show_password = Signal::new(false);

    let toggle_eye = row((
        when(
            move || show_password.get(),
            || vector(res::vectors::eye_off).frame(20.0, 20.0).tint(Color::hex(0x8E8E93)).any(),
        )
        .otherwise(|| vector(res::vectors::eye).frame(20.0, 20.0).tint(Color::hex(0x8E8E93)).any()),
    ))
    .align(VAlign::Center)
    .frame(36.0, 36.0)
    .on_tap({
        let show_password = show_password;
        move || {
            crate::shared::haptics::tick();
            show_password.set(!show_password.get());
        }
    });

    let on_sub_plain = on_submit.clone();
    let on_sub_sec = on_submit.clone();

    let pw_field = when(
        move || show_password.get(),
        {
            let on_sub = on_sub_plain.clone();
            move || {
                text_field(password)
                    .placeholder("Пароль")
                    .focused(password_focused)
                    .grow_w()
                    .id("password-field-plain")
                    .log_pos("Auth:PasswordField")
                    .on_submit(on_sub.clone())
                    .any()
            }
        },
    )
    .otherwise({
        let on_sub = on_sub_sec.clone();
        move || {
            text_field(password)
                .secure()
                .placeholder("Пароль")
                .focused(password_focused)
                .grow_w()
                .id("password-field")
                .log_pos("Auth:PasswordField")
                .on_submit(on_sub.clone())
                .any()
        }
    })
    .grow();

    let input_card = row((
        vector(res::vectors::lock)
            .frame(20.0, 20.0)
            .tint(move || {
                if password_focused.get() {
                    Color::hex(state.accent_color.get())
                } else {
                    Color::hex(0x8E8E93)
                }
            }),
        spacer().width(12.0),
        pw_field,
        toggle_eye,
    ))
    .align(VAlign::Center)
    .padding(Insets { top: 6.0, leading: 16.0, bottom: 6.0, trailing: 10.0 })
    .background(move || {
        if day::dark_mode() {
            if password_focused.get() {
                Color::rgba(1.0, 1.0, 1.0, 0.12)
            } else {
                Color::rgba(1.0, 1.0, 1.0, 0.07)
            }
        } else {
            if password_focused.get() {
                Color::rgba(0.0, 0.0, 0.0, 0.06)
            } else {
                Color::rgba(0.0, 0.0, 0.0, 0.035)
            }
        }
    })
    .glass_radius(16.0);

    let content = column((
        input_card
            .padding(Insets { top: 0.0, leading: 20.0, bottom: 8.0, trailing: 20.0 })
            .grow_w(),
    ));

    let err_display = when(
        move || !error_local.get().is_empty() || !state.error_msg.get().is_empty(),
        move || {
            let local_err = error_local.get();
            let state_err = state.error_msg.get();
            let (code, msg) = if !local_err.is_empty() {
                let err = parse_auth_error(&local_err);
                (err.code, err.message)
            } else {
                let err = parse_auth_error(&state_err);
                (err.code, err.message)
            };
            column((
                label(msg)
                    .font(Font::Subheadline)
                    .color(colors::ERROR)
                    .align(TextAlign::Center),
                spacer().height(2.0),
                label(format!("Код ошибки: [{code}]"))
                    .font(Font::Caption2)
                    .bold()
                    .color(colors::ERROR.with_alpha(0.8))
                    .align(TextAlign::Center),
            ))
            .spacing(2.0)
            .padding(Insets { top: 0.0, leading: 24.0, bottom: 8.0, trailing: 24.0 })
            .grow_w()
            .any()
        },
    );

    let btn_title = move || {
        if state.loading.get() {
            "Вход в систему…"
        } else {
            "Войти"
        }
    };

    let bottom_section = column((
        content,
        err_display,
        auth_action_button(btn_title, on_submit),
        spacer().height(8.0),
    ));

    #[cfg(target_os = "ios")]
    let bottom_section = {
        use day_uikit::UiKitExt;
        bottom_section.uikit(|view, _class, _mtm| {
            crate::shared::keyboard::register_content_view(view);
        })
    };

    let piece = column((
        spacer().grow(),
        sticker,
        spacer().grow(),
        bottom_section,
    ))
    .grow()
    .background(move || {
        if day::dark_mode() {
            Color::hex(0x000000)
        } else {
            Color::hex(0xFFFFFF)
        }
    });

    #[cfg(target_os = "ios")]
    let piece = {
        use day_uikit::UiKitExt;
        piece.uikit(move |view, _class, _mtm| {
            view.setBackgroundColor(Some(&objc2_ui_kit::UIColor::systemBackgroundColor()));
            if let Some(w) = view.window() {
                w.setBackgroundColor(Some(&objc2_ui_kit::UIColor::systemBackgroundColor()));
            }
        })
    };

    piece
}

#[derive(Clone, Copy)]
struct AuthState {
    path: Signal<Vec<AuthRoute>>,
    username: Signal<String>,
    password: Signal<String>,
    username_focused: Signal<bool>,
    password_focused: Signal<bool>,
    remember: Signal<bool>,
    error_local: Signal<String>,
}

fn get_auth_state() -> AuthState {
    thread_local! {
        static STATE: std::cell::RefCell<Option<AuthState>> = const { std::cell::RefCell::new(None) };
    }
    STATE.with(|cell| {
        let mut b = cell.borrow_mut();
        if let Some(s) = b.as_ref() {
            *s
        } else {
            let s = AuthState {
                path: Signal::global(Vec::<AuthRoute>::new()),
                username: Signal::global(String::new()),
                password: Signal::global(String::new()),
                username_focused: Signal::global(false),
                password_focused: Signal::global(false),
                remember: Signal::global(true),
                error_local: Signal::global(String::new()),
            };
            *b = Some(s);
            s
        }
    })
}

fn focus_auth_route_after(route: AuthRoute, delay: u32) {
    let ticket = crate::shared::session::ticket();
    day::reactive::on_main_delayed(delay, move || {
        if !ticket.is_current() { return; }
        let Some(state) = AppState::get_main() else { return };
        if state.is_authenticated.get() || state.setup_new_password_active.get() { return; }
        let auth = get_auth_state();
        if auth.path.get_untracked().last() != Some(&route) { return; }
        match route {
            AuthRoute::Login => auth.username_focused.set(true),
            AuthRoute::Password => auth.password_focused.set(true),
        }
    });
}

#[cfg(target_os = "ios")]
fn restore_auth_focus_after_sheet() {
    if let Some(route) = get_auth_state().path.get_untracked().last().copied() {
        focus_auth_route_after(route, 350);
    }
}

fn auth_form(state: AppState, _show_logs: Signal<bool>) -> impl Piece {
    let auth = get_auth_state();
    let path = auth.path;
    let username = auth.username;
    let password = auth.password;
    let username_focused = auth.username_focused;
    let password_focused = auth.password_focused;
    let remember = auth.remember;
    let error_local = auth.error_local;

    let root = intro_view({
        let path = path;
        let error_local = error_local;
        let state = state;
        move || {
            crate::shared::haptics::tick();
            error_local.set(String::new());
            state.error_msg.set(String::new());
            path.set(vec![AuthRoute::Login]);
            focus_auth_route_after(AuthRoute::Login, 350);
        }
    });

    let proceed_username = {
        let username = username;
        let error_local = error_local;
        let state = state;
        let path = path;
        move || {
            let u = username.get_untracked();
            if u.trim().is_empty() {
                error_local.set("Введите логин".into());
            } else {
                error_local.set(String::new());
                state.error_msg.set(String::new());
                crate::shared::haptics::tick();
                path.set(vec![AuthRoute::Login, AuthRoute::Password]);
                focus_auth_route_after(AuthRoute::Password, 350);
            }
        }
    };

    let do_login = {
        let username = username;
        let password = password;
        let remember = remember;
        let error_local = error_local;
        let state = state;
        move || {
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
        }
    };

    let proceed_username_sub = proceed_username.clone();
    let do_login_sub = do_login.clone();

    let stack = nav_stack(path, root)
        .title("ESchool")
        .toolbar(|| auth_toolbar("intro"))
        .destination(move |route: &AuthRoute| {
            match route {
                AuthRoute::Login => username_step_view(
                    username,
                    password,
                    remember,
                    username_focused,
                    error_local,
                    state,
                    proceed_username_sub.clone(),
                )
                .toolbar(|| auth_toolbar("username"))
                .any(),
                AuthRoute::Password => password_step_view(
                    username,
                    password,
                    password_focused,
                    error_local,
                    state,
                    do_login_sub.clone(),
                )
                .toolbar(|| auth_toolbar("password"))
                .any(),
            }
        })
        .id("auth-stack")
        .grow()
        .background(move || {
            if day::dark_mode() {
                Color::hex(0x000000)
            } else {
                Color::hex(0xFFFFFF)
            }
        });

    day::reactive::watch(
        move || path.get(),
        move |p, old| {
            if p.is_empty() {
                username_focused.set(false);
                password_focused.set(false);
                crate::shared::keyboard::dismiss();
            } else if old.is_some_and(|old| old.len() > p.len()) {
                // Day reports only completed native pops. A cancelled swipe
                // never changes the path and must not dismiss/refocus fields.
                if p.last() == Some(&AuthRoute::Login) {
                    password_focused.set(false);
                    focus_auth_route_after(AuthRoute::Login, 0);
                }
            }
        },
    );

    stack
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
                label(move || {
                    let local = error_local.get();
                    let state_err = state.error_msg.get();
                    if !local.is_empty() {
                        local
                    } else if !state_err.is_empty() {
                        let err = parse_auth_error(&state_err);
                        err.message
                    } else {
                        "Вы вошли по временному коду. Придумайте надёжный постоянный пароль для кабинета родителя (12–15 символов, буквы, цифры и спецсимволы)".to_string()
                    }
                })
                .font(Font::Subheadline)
                .color(move || {
                    let has_err = !error_local.get().is_empty() || !state.error_msg.get().is_empty();
                    if has_err { colors::ERROR } else { colors::SECONDARY }
                })
                .align(TextAlign::Center)
                .padding(Insets { top: 6.0, leading: 24.0, bottom: 16.0, trailing: 24.0 }),
            ))
            .log_pos("Setup:TitleColumn")
            .grow_w(),

            column((
                label("НОВЫЙ ПАРОЛЬ")
                    .font(Font::Caption2)
                    .bold()
                    .secondary(),
                text_field(new_password)
                    .secure()
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
        .height(50.0)
        .grow_w()
        .padding(Insets { top: 0.0, leading: 24.0, bottom: 12.0, trailing: 24.0 })
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
                .height(50.0)
                .grow_w()
                .padding(Insets { top: 0.0, leading: 24.0, bottom: 12.0, trailing: 24.0 })
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
