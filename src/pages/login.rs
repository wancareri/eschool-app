use crate::app::AppState;
use crate::app::blur::ApplyGlassBlur;
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
            move || crate::widgets::header::page(res::str::auth_setup_title(), Signal::new(Vec::<String>::new()), setup_password_form(state, show_logs))
                .grow(Dimension::Both)
                .any(),
        ),
        when(
            move || !state.is_authenticated.get() && !state.setup_new_password_active.get(),
            move || auth_form(state, show_logs).any(),
        ),
        when(
            move || show_logs.get(),
            move || debug_log_overlay(show_logs).animation(AnimSpec::ease_out(250)).any(),
        ),
    ))
    .grow(Dimension::Both)
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
            .grow(Dimension::Width),

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
                                    .grow(Dimension::Both),
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

                        section(rows).title("Дети в дневнике").any()
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
        .grow(Dimension::Both),

        when(
            move || show_change_pwd.get(),
            move || change_password_form(state, show_change_pwd).animation(AnimSpec::ease_out(300)).any(),
        ),
    ))
    .grow(Dimension::Both)
}

struct AuthErrorInfo {
    title: String,
    message: String,
}

fn auth_error_code(raw: &str) -> &'static str {
    let lower = raw.to_lowercase();
    if lower.contains("введите логин") || lower.contains("введите имя") { "empty_login" }
    else if lower.contains("введите пароль") { "empty_password" }
    else if lower.contains("403") || lower.contains("заблокир") || lower.contains("forbidden") { "forbidden" }
    else if lower.contains("таймаут") || lower.contains("timeout") || lower.contains("timed out") { "timeout" }
    else if ["500", "502", "503", "504", "internal server"].iter().any(|s| lower.contains(s)) { "server" }
    else if ["error sending request", "не удалось подключиться", "dns", "connection refused", "failed to connect", "network", "connection reset"].iter().any(|s| lower.contains(s)) { "network" }
    else if ["401", "неверн", "unauthorized", "invalid credentials", "invalid password", "invalid login"].iter().any(|s| lower.contains(s)) { "credentials" }
    else { "generic" }
}

fn parse_auth_error(raw: &str) -> AuthErrorInfo {
    let (title, message) = match auth_error_code(raw) {
        "empty_login" => (res::str::auth_error_login_title().format(), res::str::auth_error_login_message().format()),
        "empty_password" => (res::str::auth_error_password_title().format(), res::str::auth_error_password_message().format()),
        "forbidden" => (res::str::auth_error_forbidden_title().format(), res::str::auth_error_forbidden_message().format()),
        "timeout" => (res::str::auth_error_timeout_title().format(), res::str::auth_error_timeout_message().format()),
        "server" => (res::str::auth_error_server_title().format(), res::str::auth_error_server_message().format()),
        "network" => (res::str::auth_error_network_title().format(), res::str::auth_error_network_message().format()),
        "credentials" => (res::str::auth_error_credentials_title().format(), res::str::auth_error_credentials_message().format()),
        _ => (res::str::auth_error_generic_title().format(), res::str::auth_error_generic_message().format()),
    };
    AuthErrorInfo { title, message }
}


pub fn error_notice(raw: &str) -> crate::widgets::notice::Notice {
    let error = parse_auth_error(raw);
    crate::widgets::notice::Notice::error(error.title, error.message)
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
            AuthRoute::Login => res::str::auth_username_title().format(),
            AuthRoute::Password => res::str::auth_password_title().format(),
        }
    }
}

fn auth_action_button<M>(
    title: impl IntoText<M>,
    enabled: impl Fn() -> bool + 'static,
    action: impl Fn() + 'static,
) -> impl Piece {
    let piece = button(title)
        .prominent()
        .enabled(enabled)
        .action(action);

    // Configure the UIButton leaf, before layout-only decorators hide its handle.
    #[cfg(target_os = "ios")]
    let piece = piece.tweak(native_auth_action_button);

    piece
        .id(day::id!(login_btn))
        .height(52.0)
        .grow(Dimension::Width)
        .log_pos("Auth:ActionButton")
        .padding(Insets { top: 0.0, leading: 20.0, bottom: 8.0, trailing: 20.0 })
}

#[cfg(target_os = "ios")]
fn native_auth_action_button(node: day::RNode) {
    use objc2::{AnyThread, rc::Retained, runtime::{AnyObject, ProtocolObject}};
    use objc2_foundation::{NSAttributedString, NSDictionary, NSMutableCopying, NSString};
    use objc2_ui_kit::{
        NSFontAttributeName, UIButton, UIButtonConfiguration, UIButtonConfigurationCornerStyle,
        UIColor, UIFont, UIFontWeightSemibold,
    };
    use std::ptr::NonNull;

    let _ = day_uikit::with_native(node, |view, _, mtm| {
        let Some(button) = view.downcast_ref::<UIButton>() else { return };
        let title = button.configuration().and_then(|config| config.title()).unwrap_or_default();
        let config = UIButtonConfiguration::filledButtonConfiguration(mtm);
        config.setCornerStyle(UIButtonConfigurationCornerStyle::Capsule);
        // Leave the fill tied to the native tint and let UIKit derive the
        // highlighted/disabled colors from these base colors.
        config.setBaseForegroundColor(Some(&UIColor::whiteColor()));
        let font = unsafe { UIFont::systemFontOfSize_weight(17.0, UIFontWeightSemibold) };
        let attributes = NSDictionary::<NSString, AnyObject>::from_slices(
            &[unsafe { NSFontAttributeName }], &[font.as_ref()],
        );
        let attributed = unsafe {
            NSAttributedString::initWithString_attributes(
                NSAttributedString::alloc(), &title, Some(&attributes),
            )
        };
        config.setAttributedTitle(Some(&attributed));

        // Day patches the configuration's plain title for locale/loading changes.
        // Keep its replacement title semibold too, without freezing the text or
        // overriding UIKit's state-dependent foreground color.
        let transformer = block2::RcBlock::new(move |incoming: NonNull<NSDictionary<NSString, AnyObject>>| {
            let attributes = unsafe { incoming.as_ref() }.mutableCopy();
            unsafe {
                attributes.setObject_forKey(font.as_ref(), ProtocolObject::from_ref(NSFontAttributeName));
            }
            let attributes: Retained<NSDictionary<NSString, AnyObject>> = attributes.into_super();
            NonNull::new(Retained::autorelease_ptr(attributes)).expect("retained title attributes")
        });
        unsafe { config.setTitleTextAttributesTransformer(block2::RcBlock::as_ptr(&transformer)) };
        button.setConfiguration(Some(&config));
    });
}

/// One window-level menu shared by every authentication page. Native UIMenu
/// supplies its own anchor; no action sheet coordinates or per-page duplicates.
pub fn auth_toolbar() -> Vec<ToolbarEntry> {
    let locale = day::locale().get().to_lowercase();
    let _ = get_auth_state().settings_revision.get();
    let theme = day::prefs::get(crate::THEME_KEY);
    let dark = day::dark_mode(); // React to appearance changes, including system changes.
    let language = sub_menu(res::str::auth_language().format(), vec![
        menu_item("RU · Русский").id(day::id!(auth_language_ru)).checked(locale.starts_with("ru")).action(|| choose_auth_language("ru")),
        menu_item("BE · Беларуская").id(day::id!(auth_language_be)).checked(locale.starts_with("be")).action(|| choose_auth_language("be")),
        menu_item("EN · English").id(day::id!(auth_language_en)).checked(locale.starts_with("en")).action(|| choose_auth_language("en")),
    ]).image("globe");
    let appearance = sub_menu(res::str::auth_appearance().format(), vec![
        menu_item(res::str::auth_theme_system().format()).id(day::id!(auth_theme_system)).icon(Symbol::Auto)
            .checked(!matches!(theme.as_deref(), Some("light" | "dark"))).action(|| choose_auth_theme(None)),
        menu_item(res::str::auth_theme_light().format()).id(day::id!(auth_theme_light)).icon(Symbol::Light)
            .checked(theme.as_deref() == Some("light")).action(|| choose_auth_theme(Some(false))),
        menu_item(res::str::auth_theme_dark().format()).id(day::id!(auth_theme_dark)).icon(Symbol::Dark)
            .checked(theme.as_deref() == Some("dark")).action(|| choose_auth_theme(Some(true))),
    ]).icon(if dark { Symbol::Dark } else { Symbol::Light });
    let diagnostics = menu_item(res::str::autofill_diagnostics_copy().format())
        .id(day::id!(auth_copy_input_diagnostics))
        .icon(Symbol::Copy)
        .action(|| {
            let report = crate::shared::nslog::autofill_diagnostics();
            let report = if report.is_empty() {
                res::str::autofill_diagnostics_empty().format()
            } else {
                report
            };
            day_part_clipboard::set_text(&report);
            crate::shared::haptics::tick();
        });
    vec![toolbar_menu(day::id!(auth_options), res::str::auth_options(), vec![language, appearance, diagnostics])
        .icon(Symbol::More).placement(ToolbarPlacement::Primary)]
}

fn choose_auth_language(locale: &'static str) {
    if day::locale().get_untracked().starts_with(locale) { return; }
    #[cfg(target_os = "ios")]
    crate::shared::autofill::ios::sync_values();
    day::prefs::set(crate::LOCALE_KEY, locale);
    set_locale(locale);
    #[cfg(target_os = "ios")]
    crate::shared::autofill::ios::refresh_active();
    crate::shared::haptics::tick();
}

fn choose_auth_theme(appearance: Option<bool>) {
    match appearance {
        Some(true) => { day::prefs::set(crate::THEME_KEY, "dark"); }
        Some(false) => { day::prefs::set(crate::THEME_KEY, "light"); }
        None => { day::prefs::remove(crate::THEME_KEY); }
    }
    day::set_appearance(appearance);
    get_auth_state().settings_revision.update(|revision| *revision += 1);
    crate::shared::haptics::tick();
}

fn intro_view(on_start: impl Fn() + 'static) -> impl Piece {
    let piece = column((
        spacer().height(16.0),
        label("ESchool")
            .font(Font::LargeTitle)
            .bold()
            .align(TextAlign::Center),
        spacer().height(6.0),
        label(res::str::auth_intro_subtitle())
            .font(Font::Headline)
            .secondary()
            .align(TextAlign::Center),
        spacer().height(10.0),
        label(res::str::auth_intro_description())
            .font(Font::Subheadline)
            .secondary()
            .align(TextAlign::Center)
            .padding(Insets { top: 0.0, leading: 32.0, bottom: 0.0, trailing: 32.0 }),
        spacer().grow(Dimension::Both),
        row((
            spacer().grow(Dimension::Both),
            crate::widgets::lottie_view::duck_intro_animation(240.0),
            spacer().grow(Dimension::Both),
        )),
        spacer().grow(Dimension::Both),
        auth_action_button(res::str::auth_start(), || true, on_start),
        spacer().height(8.0),
    ))
    .grow(Dimension::Both)
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

// Native field configuration and flexible sizing must share this boundary.
#[cfg(any(target_os = "ios", test))]
fn native_auth_input(
    field: impl Decorate,
    configure: impl FnOnce(day::RNode) + 'static,
) -> impl Piece {
    // Layout decorators can return nodes without native handles. Configure
    // the leaf before wrapping it, otherwise with_native silently returns None.
    field.tweak(configure).grow(Dimension::Width)
}

fn username_step_view(
    username: Signal<String>,
    password: Signal<String>,
    remember: Signal<bool>,
    username_focused: Signal<bool>,
    state: AppState,
    on_submit: impl Fn() + Clone + 'static,
) -> impl Piece {
    let sticker = row((
        spacer().grow(Dimension::Both),
        crate::widgets::lottie_view::duck_login_animation(230.0),
        spacer().grow(Dimension::Both),
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
        .input_purpose(InputPurpose::Username)
        .submit_label(SubmitLabel::Next)
        .placeholder(res::str::auth_username_placeholder())
        .focused(username_focused)
        .log_pos("Auth:UsernameField")
        .on_submit(on_sub);

    #[cfg(target_os = "ios")]
    let input_field = native_auth_input(input_field, move |node| {
        let _ = day_uikit::with_native(node, move |view, _, _| {
            crate::shared::autofill::ios::username(view, username);
        });
    });
    #[cfg(not(target_os = "ios"))]
    let input_field = input_field.grow(Dimension::Width);

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

    // The user explicitly requested keeping two steps with a capture input.
    // Mount only while this step is active so it never competes with the real
    // password field on the destination page. It has no hit/accessibility target.
    #[cfg(target_os = "ios")]
    let credential_capture = {
        when(move || get_auth_state().path.get().last() == Some(&AuthRoute::Login), move || {
            native_auth_input(text_field(password).secure(true)
                .input_purpose(InputPurpose::Password)
                .submit_label(SubmitLabel::Go), move |node| {
                let _ = day_uikit::with_native(node, move |view, _, _| {
                    crate::shared::autofill::ios::capture_password(view, password);
                });
            }).height(1.0).any()
        })
    };
    #[cfg(not(target_os = "ios"))]
    let credential_capture = { let _ = password; spacer().height(0.0) };

    let content = column((
        input_card
            .padding(Insets { top: 0.0, leading: 20.0, bottom: 12.0, trailing: 20.0 })
            .grow(Dimension::Width),
        credential_capture,

        // Minimalist native session remember toggle
        row((
            label(res::str::auth_remember())
                .font(Font::Subheadline)
                .secondary()
                .grow(Dimension::Both),
            toggle(remember),
        ))
        .align(VAlign::Center)
        .padding(Insets { top: 0.0, leading: 24.0, bottom: 8.0, trailing: 24.0 })
        .grow(Dimension::Width),
    ));


    let bottom_section = column((
        label(res::str::auth_step_username()).font(Font::Caption).secondary()
            .padding(Insets { top: 0.0, leading: 20.0, bottom: 12.0, trailing: 20.0 }),
        content,
        auth_action_button(res::str::auth_continue(),
            move || !username.get().trim().is_empty() && !state.loading.get(), on_submit),
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
        spacer().grow(Dimension::Both),
        sticker,
        spacer().grow(Dimension::Both),
        bottom_section,
    ))
    .grow(Dimension::Both)
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
            crate::shared::ios_nav::ios_impl::bind_auth_page_title(view, || res::str::auth_username_title().format());
            view.setBackgroundColor(Some(&objc2_ui_kit::UIColor::systemBackgroundColor()));
            if let Some(w) = view.window() {
                w.setBackgroundColor(Some(&objc2_ui_kit::UIColor::systemBackgroundColor()));
            }
        })
    };

    piece
}

fn password_step_view(
    username: Signal<String>,
    password: Signal<String>,
    password_focused: Signal<bool>,
    state: AppState,
    on_submit: impl Fn() + Clone + 'static,
) -> impl Piece {
    let sticker = row((
        spacer().grow(Dimension::Both),
        crate::widgets::lottie_view::duck_password_animation(230.0),
        spacer().grow(Dimension::Both),
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

    let pw_field = text_field(password)
        .input_purpose(InputPurpose::Password)
        .submit_label(SubmitLabel::Go)
        .secure(move || !show_password.get())
        .placeholder(res::str::auth_password_placeholder())
        .focused(password_focused)
        .id(day::id!(password_field))
        .log_pos("Auth:PasswordField")
        .on_submit(on_submit.clone());

    #[cfg(target_os = "ios")]
    let pw_field = native_auth_input(pw_field, move |node| {
        let _ = day_uikit::with_native(node, move |view, _, _| {
            crate::shared::autofill::ios::password(view, password);
        });
    });

    #[cfg(not(target_os = "ios"))]
    let pw_field = pw_field.grow(Dimension::Width);

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
            if password_focused.get() || !password.get().is_empty() {
                Color::rgba(1.0, 1.0, 1.0, 0.12)
            } else {
                Color::rgba(1.0, 1.0, 1.0, 0.07)
            }
        } else {
            if password_focused.get() || !password.get().is_empty() {
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
            .grow(Dimension::Width),
    ));


    let btn_title = move || {
        if state.loading.get() {
            res::str::auth_signing_in().format()
        } else {
            res::str::auth_sign_in().format()
        }
    };

    let bottom_section = column((
        label(res::str::auth_step_password()).font(Font::Caption).secondary(),
        label(move || username.get()).font(Font::Subheadline).bold()
            .padding(Insets { top: 4.0, leading: 20.0, bottom: 12.0, trailing: 20.0 }),
        content,
        auth_action_button(btn_title,
            move || !password.get().trim().is_empty() && !state.loading.get(), on_submit),
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
        spacer().grow(Dimension::Both),
        sticker,
        spacer().grow(Dimension::Both),
        bottom_section,
    ))
    .grow(Dimension::Both)
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
            crate::shared::ios_nav::ios_impl::bind_auth_page_title(view, || res::str::auth_password_title().format());
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
    settings_revision: Signal<u64>,
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
                settings_revision: Signal::global(0),
            };
            *b = Some(s);
            s
        }
    })
}

/// Clear every auth draft before showing the welcome page again.
pub fn reset_auth_flow() {
    #[cfg(target_os = "ios")]
    crate::shared::autofill::ios::reset();
    let auth = get_auth_state();
    day::reactive::batch(|| {
        auth.username_focused.set(false);
        auth.password_focused.set(false);
        auth.username.set(String::new());
        auth.password.set(String::new());
        auth.error_local.set(String::new());
        auth.remember.set(true);
        auth.path.set(Vec::new());
    });
}

fn focus_auth_route(route: AuthRoute) {
    let ticket = crate::shared::session::ticket();
    day::reactive::on_main(move || {
        let allowed = move || {
            if !ticket.is_current() { return false; }
            let Some(state) = AppState::get_main() else { return false };
            !state.is_authenticated.get_untracked()
                && !state.setup_new_password_active.get_untracked()
                && get_auth_state().path.get_untracked().last() == Some(&route)
        };
        #[cfg(target_os = "ios")]
        crate::shared::autofill::ios::focus_after_navigation(
            match route {
                AuthRoute::Login => crate::shared::autofill::ios::Role::Username,
                AuthRoute::Password => crate::shared::autofill::ios::Role::Password,
            }, allowed,
        );
        #[cfg(not(target_os = "ios"))]
        if allowed() {
            let auth = get_auth_state();
            match route {
                AuthRoute::Login => auth.username_focused.set(true),
                AuthRoute::Password => auth.password_focused.set(true),
            }
        }
    });
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
    crate::widgets::notice::bind_error(state, error_local, error_notice);

    let root = intro_view({
        let path = path;
        let error_local = error_local;
        let state = state;
        move || {
            crate::shared::haptics::tick();
            error_local.set(String::new());
            state.error_msg.set(String::new());
            path.set(vec![AuthRoute::Login]);
            focus_auth_route(AuthRoute::Login);
        }
    });

    let proceed_username = {
        let username = username;
        let error_local = error_local;
        let state = state;
        let path = path;
        move || {
            if state.loading.get() { return; }
            #[cfg(target_os = "ios")]
            crate::shared::autofill::ios::sync_values();
            let u = username.get_untracked();
            if u.trim().is_empty() {
                error_local.set("Введите логин".into());
            } else {
                error_local.set(String::new());
                state.error_msg.set(String::new());
                crate::shared::haptics::tick();
                path.set(vec![AuthRoute::Login, AuthRoute::Password]);
                focus_auth_route(AuthRoute::Password);
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
            if p.trim().is_empty() {
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

    let stack = crate::widgets::header::page("ESchool", path, root)
        .destination(move |route: &AuthRoute| {
            match route {
                AuthRoute::Login => username_step_view(
                    username,
                    password,
                    remember,
                    username_focused,
                    state,
                    proceed_username_sub.clone(),
                )
                .any(),
                AuthRoute::Password => password_step_view(
                    username,
                    password,
                    password_focused,
                    state,
                    do_login_sub.clone(),
                )
                .any(),
            }
        })
        .id(day::id!(auth_stack))
        .grow(Dimension::Both)
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
                    focus_auth_route(AuthRoute::Login);
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
    crate::widgets::notice::bind_error(state, error_local, |message| {
        crate::widgets::notice::Notice::error(res::str::auth_setup_title().format(), message)
    });

    column((
        debug_log_button(show_logs),
        spacer().height(16.0),

            column((
                label("Создание постоянного пароля")
                    .font(Font::Title)
                    .bold()
                    .align(TextAlign::Center),
                label("Вы вошли по временному коду. Придумайте надёжный постоянный пароль для кабинета родителя (12–15 символов, буквы, цифры и спецсимволы)")
                .font(Font::Subheadline)
                .secondary()
                .align(TextAlign::Center)
                .padding(Insets { top: 6.0, leading: 24.0, bottom: 16.0, trailing: 24.0 }),
            ))
            .log_pos("Setup:TitleColumn")
            .grow(Dimension::Width),

            column((
                label("НОВЫЙ ПАРОЛЬ")
                    .font(Font::Caption2)
                    .bold()
                    .secondary(),
                text_field(new_password)
                    .secure(true)
                    .input_purpose(InputPurpose::NewPassword)
                    .submit_label(SubmitLabel::Next)
                    .tweak(crate::shared::autofill::new_password)
                    .grow(Dimension::Width)
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
            .grow(Dimension::Width),

            column((
                label("ПОВТОРИТЕ ПАРОЛЬ")
                    .font(Font::Caption2)
                    .bold()
                    .secondary(),
                text_field(confirm_password)
                    .secure(true)
                    .input_purpose(InputPurpose::NewPassword)
                    .submit_label(SubmitLabel::Done)
                    .tweak(crate::shared::autofill::new_password)
                    .focused(confirm_focused)
                    .grow(Dimension::Width)
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
            .grow(Dimension::Width),

        spacer().grow(Dimension::Both),

        button(move || {
            if state.loading.get() {
                "Сохранение…"
            } else {
                "Создать пароль и войти"
            }
        })
        .prominent()
        .height(50.0)
        .grow(Dimension::Width)
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
        .grow(Dimension::Width),

        button("← Назад к входу")
            .log_pos("Setup:BackButton")
            .action(move || {
                crate::shared::haptics::tick();
                crate::shared::keyboard::dismiss();
                state.setup_new_password_active.set(false);
                state.setup_temp_password_data.set(None);
            })
            .padding(Insets { top: 0.0, leading: 24.0, bottom: 8.0, trailing: 24.0 }),

        spacer().height(move || {
                let h = kb_h.get();
                if h > 0.0 {
                    let safe_b = crate::shared::keyboard::get_safe_area_bottom();
                    (h - safe_b).max(0.0) + 12.0
                } else {
                    4.0
                }
            }),
    ))
    .grow(Dimension::Both)
}

fn change_password_form(state: AppState, open: Signal<bool>) -> impl Piece {
    let old_password = Signal::new(String::new());
    let new_password = Signal::new(String::new());
    let confirm_password = Signal::new(String::new());
    let confirm_focused = Signal::new(false);
    let error_local = Signal::new(String::new());
    let notices = crate::widgets::notice::presenter(state);
    crate::widgets::notice::bind_error(state, error_local, |message| {
        crate::widgets::notice::Notice::error(res::str::notice_error_title().format(), message)
    });
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
                .grow(Dimension::Width),

                column((
                    label("ТЕКУЩИЙ ПАРОЛЬ")
                        .font(Font::Caption2)
                        .bold()
                        .secondary(),
                    text_field(old_password)
                        .secure(true)
                        .input_purpose(InputPurpose::Password)
                        .submit_label(SubmitLabel::Go)
                        .tweak(crate::shared::autofill::current_password)
                        .grow(Dimension::Width)
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
                .grow(Dimension::Width),

                column((
                    label("НОВЫЙ ПАРОЛЬ")
                        .font(Font::Caption2)
                        .bold()
                        .secondary(),
                    text_field(new_password)
                        .secure(true)
                        .input_purpose(InputPurpose::NewPassword)
                        .submit_label(SubmitLabel::Next)
                        .tweak(crate::shared::autofill::new_password)
                        .grow(Dimension::Width)
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
                .grow(Dimension::Width),

                column((
                    label("ПОВТОРИТЕ НОВЫЙ ПАРОЛЬ")
                        .font(Font::Caption2)
                        .bold()
                        .secondary(),
                    text_field(confirm_password)
                        .secure(true)
                        .input_purpose(InputPurpose::NewPassword)
                        .submit_label(SubmitLabel::Done)
                        .tweak(crate::shared::autofill::new_password)
                        .focused(confirm_focused)
                        .grow(Dimension::Width)
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
                .grow(Dimension::Width),

                button(move || {
                    if state.loading.get() {
                        "Сохранение…"
                    } else {
                        "Сохранить новый пароль"
                    }
                })
                .prominent()
                .height(50.0)
                .grow(Dimension::Width)
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
                    features::auth::change_logged_in_parent_password(
                        state,
                        &op_c,
                        &np_c,
                        move || {
                            crate::shared::haptics::pop();
                            notices.show(crate::widgets::notice::Notice {
                                title: res::str::notice_success_title().format(),
                                message: res::str::notice_password_updated().format(),
                                kind: crate::widgets::notice::NoticeKind::Success,
                            });
                            set_open.set(false);
                        },
                    );
                })
                .padding(Insets { top: 8.0, leading: 20.0, bottom: 12.0, trailing: 20.0 })
                .grow(Dimension::Width),

                button("Отмена")
                    .action(move || {
                        crate::shared::haptics::tick();
                        crate::shared::keyboard::dismiss();
                        open.set(false);
                    })
                    .padding(Insets { top: 0.0, leading: 24.0, bottom: 16.0, trailing: 24.0 }),

                spacer().frame(0.0, move || {
                        let h = kb_h.get();
                        if h > 0.0 {
                            let safe_b = crate::shared::keyboard::get_safe_area_bottom();
                            (h - safe_b).max(0.0) + 20.0
                        } else {
                            0.0
                        }
                    }),
            ))
            .grow(Dimension::Width)
        )
        .on_tap(move || {
            crate::shared::keyboard::dismiss();
        })
        .grow(Dimension::Both),
    ))
    .background(move || {
        if day::dark_mode() {
            Color::hex(0x000000)
        } else {
            Color::hex(0xF2F2F7)
        }
    })
    .grow(Dimension::Both)
}

#[cfg(test)]
mod auth_error_tests {
    use super::auth_error_code;

    #[cfg(feature = "mock")]
    #[test]
    fn auth_step_buttons_follow_trimmed_input_and_loading() {
        use day::prelude::*;
        for password_step in [false, true] {
            day_core::uninstall_tree();
            let value = Signal::global(String::new());
            let loading = Signal::global(false);
            let (mock, probe) = day_mock::MockToolkit::new();
            day_core::launch_with(mock, day_spec::WindowOptions::default(), move || {
                let mut state = crate::app::AppState::create();
                state.loading = loading;
                if password_step {
                    super::password_step_view(Signal::new("account".into()), value,
                        Signal::new(false), state, || {}).any()
                } else {
                    super::username_step_view(value, Signal::new(String::new()),
                        Signal::new(true), Signal::new(false), state, || {}).any()
                }
            });
            let buttons = probe.find_by_kind(day_spec::kinds::BUTTON);
            assert_eq!(buttons.len(), 1);
            let handle = buttons[0].0;
            for (text, busy, enabled) in [
                ("", false, false), (" \t\n\u{2003}", false, false),
                (" account ", false, true), ("account", true, false),
                ("account", false, true), ("", false, false),
            ] {
                value.set(text.into());
                loading.set(busy);
                day::reactive::flush_sync();
                let button = probe.widget(handle);
                assert_eq!(button.enabled, enabled,
                    "password_step={password_step}, busy={busy}, expected enabled={enabled}");
                assert_eq!(value.get(), text, "validation must not rewrite credentials");
                assert!(!button.text.is_empty(), "disabled native buttons keep their label");
            }
            day_core::uninstall_tree();
        }
    }

    #[test]
    fn reset_discards_credentials_focus_and_navigation() {
        use super::{AuthRoute, get_auth_state, reset_auth_flow};
        let auth = get_auth_state();
        auth.username.set("test-account".into());
        auth.password.set("test-secret".into());
        auth.path.set(vec![AuthRoute::Login, AuthRoute::Password]);
        auth.username_focused.set(true);
        auth.password_focused.set(true);
        auth.error_local.set("previous error".into());
        auth.remember.set(false);
        reset_auth_flow();
        assert!(auth.username.get().is_empty());
        assert!(auth.password.get().is_empty());
        assert!(auth.path.get().is_empty());
        assert!(!auth.username_focused.get());
        assert!(!auth.password_focused.get());
        assert!(auth.error_local.get().is_empty());
        assert!(auth.remember.get());
        reset_auth_flow();
        assert!(auth.path.get().is_empty());
    }

    #[cfg(feature = "mock")]
    #[test]
    fn credential_configuration_receives_the_native_field_before_layout_wrappers() {
        use day::prelude::*;
        use std::{cell::RefCell, rc::Rc};
        let handles = Rc::new(RefCell::new(Vec::new()));
        let observed = handles.clone();
        let shown = Signal::global(false);
        let (mock, probe) = day_mock::MockToolkit::new();
        day_core::uninstall_tree();
        day_core::launch_with(mock, day_spec::WindowOptions::default(), move || {
            let input = |purpose: InputPurpose, submit: SubmitLabel, capture: bool| {
                let observed = observed.clone();
                let value = Signal::new(String::from("fixture"));
                let focus = Signal::new(purpose == InputPurpose::Password && !capture);
                let field = text_field(value)
                    .focused(focus)
                    .input_purpose(purpose)
                    .submit_label(submit)
                    .secure(move || purpose != InputPurpose::Username && (capture || !shown.get()));
                super::native_auth_input(field, move |node| {
                    day_core::with_tree(|tree| {
                        let handle = tree.node_handle_any(node)
                            .expect("credential configuration must receive a native handle");
                        assert_eq!(tree.node_kind(node), Some(day_spec::kinds::TEXT_FIELD));
                        observed.borrow_mut().push(*handle.downcast::<day_mock::MockHandle>().unwrap());
                    });
                }).height(if capture { 1.0 } else { 44.0 }).any()
            };
            column((
                input(InputPurpose::Username, SubmitLabel::Next, false),
                input(InputPurpose::Password, SubmitLabel::Go, false),
                input(InputPurpose::Password, SubmitLabel::Go, true),
                input(InputPurpose::NewPassword, SubmitLabel::Done, false),
            )).any()
        });
        let handles = handles.borrow().clone();
        assert_eq!(handles.len(), 4, "all credential roles and capture must configure");
        for (handle, (purpose, submit_label, secure)) in handles.iter().zip([
            (InputPurpose::Username, SubmitLabel::Next, false),
            (InputPurpose::Password, SubmitLabel::Go, true),
            (InputPurpose::Password, SubmitLabel::Go, true),
            (InputPurpose::NewPassword, SubmitLabel::Done, true),
        ]) {
            let traits = probe.widget(*handle).input_traits.unwrap();
            assert_eq!(traits.purpose, purpose);
            assert_eq!(traits.submit_label, submit_label);
            assert_eq!(traits.secure, secure);
        }
        let password = probe.widget(handles[1]);
        assert!(password.focused);
        shown.set(true);
        day::reactive::flush_sync();
        let visible = probe.widget(handles[1]);
        assert!(visible.focused, "masking must preserve focus");
        assert_eq!(visible.node, password.node, "masking must preserve field identity");
        assert_eq!(visible.text, password.text, "masking must preserve the captured value");
        let traits = visible.input_traits.unwrap();
        assert!(!traits.secure);
        assert_eq!(traits.purpose, InputPurpose::Password);
        assert_eq!(traits.submit_label, SubmitLabel::Go);
        assert!(probe.widget(handles[2]).input_traits.unwrap().secure,
            "the separate capture field keeps its secure role");
        shown.set(false);
        day::reactive::flush_sync();
        assert!(probe.widget(handles[1]).input_traits.unwrap().secure);
        day_core::uninstall_tree();
    }

    #[test]
    fn transport_errors_are_not_bad_passwords() {
        assert_eq!(auth_error_code("password login: error sending request: DNS error"), "network");
        assert_eq!(auth_error_code("invalid password: request timed out"), "timeout");
        assert_eq!(auth_error_code("password endpoint: HTTP 503"), "server");
        assert_eq!(auth_error_code("HTTP 403: invalid credentials"), "forbidden");
        assert_eq!(auth_error_code("HTTP 401 Unauthorized"), "credentials");
        assert_eq!(auth_error_code("Введите пароль"), "empty_password");
    }
}
