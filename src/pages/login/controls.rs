use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum AuthRoute {
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

pub(super) fn auth_action_button<M>(
    title: impl IntoText<M>,
    enabled: impl Fn() -> bool + 'static,
    action: impl Fn() + 'static,
) -> impl Piece {
    let piece = button(title)
        .prominent()
        .enabled(enabled)
        .action(action)
        .animation(AnimSpec::ease_out(200));

    // Configure the UIButton leaf, before layout-only decorators hide its handle.
    #[cfg(feature = "uikit")]
    let piece = piece.tweak(native_auth_action_button);

    piece
        .id(day::id!(login_btn))
        .height(52.0)
        .grow(Dimension::Width)
        .corner_radius(26.0)
        .log_pos("Auth:ActionButton")
        .padding(Insets {
            top: 0.0,
            leading: 20.0,
            bottom: 8.0,
            trailing: 20.0,
        })
}

#[cfg(feature = "uikit")]
pub(super) fn native_auth_action_button(node: day::RNode) {
    use objc2::{
        MainThreadOnly,
        rc::Retained,
        runtime::{AnyObject, ProtocolObject},
    };
    use objc2_foundation::{NSDictionary, NSMutableCopying, NSString};
    use objc2_ui_kit::{
        NSFontAttributeName, NSForegroundColorAttributeName, UIButton, UIButtonConfiguration,
        UIButtonConfigurationCornerStyle, UIColor, UIFont,
    };
    use std::ptr::NonNull;

    // One transformer per main thread, retained for its lifetime rather than
    // recreated for each button or configuration update.
    thread_local! {
        static TITLE_ATTRIBUTES: block2::RcBlock<dyn Fn(
            NonNull<NSDictionary<NSString, AnyObject>>,
        ) -> NonNull<NSDictionary<NSString, AnyObject>>> = {
            let font = UIFont::boldSystemFontOfSize(17.0);
            block2::RcBlock::new(move |incoming: NonNull<NSDictionary<NSString, AnyObject>>| {
                let attributes = unsafe { incoming.as_ref() }.mutableCopy();
                unsafe {
                    attributes.setObject_forKey(font.as_ref(), ProtocolObject::from_ref(NSFontAttributeName));
                    attributes.setObject_forKey(UIColor::whiteColor().as_ref(),
                        ProtocolObject::from_ref(NSForegroundColorAttributeName));
                }
                let attributes: Retained<NSDictionary<NSString, AnyObject>> = attributes.into_super();
                NonNull::new(Retained::autorelease_ptr(attributes)).expect("retained title attributes")
            })
        };
    }

    fn apply_style(button: &UIButton, config: &UIButtonConfiguration) {
        config.setCornerStyle(UIButtonConfigurationCornerStyle::Capsule);
        let bg = config.background();
        bg.setCornerRadius(26.0);
        config.setBackground(&bg);
        config.setBaseForegroundColor(Some(&UIColor::whiteColor()));
        TITLE_ATTRIBUTES.with(|transformer| unsafe {
            config.setTitleTextAttributesTransformer(block2::RcBlock::as_ptr(transformer));
        });
        button.setConfiguration(Some(config));
        button.layer().setCornerRadius(26.0);
        button.layer().setMasksToBounds(true);
        button.setClipsToBounds(true);
        if let Some(label) = button.titleLabel() {
            unsafe { label.setFont(Some(&UIFont::boldSystemFontOfSize(17.0))) };
        }
    }

    let _ = day_uikit::with_native(node, |view, _, mtm| {
        let Some(button) = view.downcast_ref::<UIButton>() else {
            return;
        };
        let title = button
            .configuration()
            .and_then(|config| config.title())
            .unwrap_or_default();
        let config = UIButtonConfiguration::borderedProminentButtonConfiguration(mtm);
        config.setTitle(Some(&title));
        apply_style(button, &config);

        // Reapply after every native update, including Day's title/state patches.
        // Guard synchronous reentry from setConfiguration without retaining UIButton.
        let updating = std::cell::Cell::new(false);
        let update = block2::RcBlock::new(move |button: NonNull<UIButton>| {
            if updating.replace(true) {
                return;
            }
            let button = unsafe { button.as_ref() };
            let config = button.configuration().unwrap_or_else(|| {
                UIButtonConfiguration::borderedProminentButtonConfiguration(button.mtm())
            });
            let is_loading = crate::app::AppState::get_main()
                .map(|state| state.loading.get())
                .unwrap_or(false);
            config.setShowsActivityIndicator(is_loading);
            apply_style(button, &config);
            updating.set(false);
            if day::testing::fast_animations() {
                return;
            }
            use objc2_quartz_core::{
                CAMediaTiming, CAMediaTimingFunction, CATransition, kCAMediaTimingFunctionEaseOut,
            };
            if button.window().is_none() {
                return;
            }
            let fade = CATransition::new();
            fade.setDuration(0.2);
            unsafe {
                fade.setTimingFunction(Some(&CAMediaTimingFunction::functionWithName(
                    kCAMediaTimingFunctionEaseOut,
                )));
            }
            button
                .layer()
                .addAnimation_forKey(&fade, Some(&NSString::from_str("eschool.auth-action")));
        });
        unsafe { button.setConfigurationUpdateHandler(block2::RcBlock::as_ptr(&update)) };
    });
}

/// One window-level menu shared by every authentication page. Native UIMenu
/// supplies its own anchor; no action sheet coordinates or per-page duplicates.
pub fn auth_toolbar() -> Vec<ToolbarEntry> {
    let locale = day::locale().get().to_lowercase();
    let _ = get_auth_state().settings_revision.get();
    let theme = day::prefs::get(crate::THEME_KEY);
    let dark = day::dark_mode(); // React to appearance changes, including system changes.
    let language = sub_menu(
        res::str::auth_language().format(),
        vec![
            menu_item("RU · Русский")
                .id(day::id!(auth_language_ru))
                .checked(locale.starts_with("ru"))
                .action(|| choose_auth_language("ru")),
            menu_item("BE · Беларуская")
                .id(day::id!(auth_language_be))
                .checked(locale.starts_with("be"))
                .action(|| choose_auth_language("be")),
            menu_item("EN · English")
                .id(day::id!(auth_language_en))
                .checked(locale.starts_with("en"))
                .action(|| choose_auth_language("en")),
        ],
    )
    .image("globe");
    let appearance = sub_menu(
        res::str::auth_appearance().format(),
        vec![
            menu_item(res::str::auth_theme_system().format())
                .id(day::id!(auth_theme_system))
                .icon(Symbol::Auto)
                .checked(!matches!(theme.as_deref(), Some("light" | "dark")))
                .action(|| choose_auth_theme(None)),
            menu_item(res::str::auth_theme_light().format())
                .id(day::id!(auth_theme_light))
                .icon(Symbol::Light)
                .checked(theme.as_deref() == Some("light"))
                .action(|| choose_auth_theme(Some(false))),
            menu_item(res::str::auth_theme_dark().format())
                .id(day::id!(auth_theme_dark))
                .icon(Symbol::Dark)
                .checked(theme.as_deref() == Some("dark"))
                .action(|| choose_auth_theme(Some(true))),
        ],
    )
    .icon(if dark { Symbol::Dark } else { Symbol::Light });
    vec![
        toolbar_menu(
            day::id!(auth_options),
            res::str::auth_options(),
            vec![language, appearance],
        )
        .icon(Symbol::More)
        .placement(ToolbarPlacement::Primary),
    ]
}

pub(super) fn choose_auth_language(locale: &'static str) {
    if day::locale().get_untracked().starts_with(locale) {
        return;
    }
    #[cfg(feature = "uikit")]
    crate::ios::autofill::ios::sync_values();
    day::prefs::set(crate::LOCALE_KEY, locale);
    set_locale(locale);
    #[cfg(feature = "uikit")]
    crate::ios::autofill::ios::refresh_active();
    crate::app::preferences::haptics::tick();
}

pub(super) fn choose_auth_theme(appearance: Option<bool>) {
    match appearance {
        Some(true) => {
            day::prefs::set(crate::THEME_KEY, "dark");
        }
        Some(false) => {
            day::prefs::set(crate::THEME_KEY, "light");
        }
        None => {
            day::prefs::remove(crate::THEME_KEY);
        }
    }
    day::set_appearance(appearance);
    get_auth_state()
        .settings_revision
        .update(|revision| *revision += 1);
    crate::app::preferences::haptics::tick();
}

pub(super) fn intro_view(on_start: impl Fn() + 'static) -> impl Piece {
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
            .padding(Insets {
                top: 0.0,
                leading: 32.0,
                bottom: 0.0,
                trailing: 32.0,
            }),
        spacer().grow(Dimension::Both),
        row((
            spacer().grow(Dimension::Both),
            crate::pages::login::lottie_view::duck_intro_animation(240.0),
            spacer().grow(Dimension::Both),
        )),
        spacer().grow(Dimension::Both),
        auth_keyboard_dock(auth_action_button(
            res::str::auth_start(),
            || true,
            on_start,
        )),
    ))
    .spacing(0.0)
    .grow(Dimension::Both)
    .background(move || {
        if day::dark_mode() {
            Color::hex(0x000000)
        } else {
            Color::hex(0xFFFFFF)
        }
    });

    #[cfg(feature = "uikit")]
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
#[cfg(any(feature = "uikit", test))]
pub(super) fn native_auth_input(
    field: impl Decorate,
    configure: impl FnOnce(day::RNode) + 'static,
) -> impl Piece {
    // Layout decorators can return nodes without native handles. Configure
    // the leaf before wrapping it, otherwise with_native silently returns None.
    field.tweak(configure).grow(Dimension::Width)
}

/// Only the action follows the keyboard. The compact sticker and input card
/// retain their top positions throughout keyboard presentation and dismissal.
pub(super) fn auth_keyboard_dock(action: impl Piece) -> impl Piece {
    let dock = column((action,)).spacing(0.0);
    #[cfg(feature = "uikit")]
    let dock = {
        use day_uikit::UiKitExt;
        dock.uikit(|view, _, _| crate::ios::keyboard::register_follower_view(view))
    };
    dock.grow(Dimension::Width)
}

/// Day 0.7's UIKit image tint patch ignores implicit animation intent.
/// Interpolate the color itself, leaving the field and form layout untouched.
pub(super) fn animated_focus_tint(
    focused: Signal<bool>,
    accent: Signal<u32>,
) -> day::tween::Tweened<Color> {
    let target = move || {
        Color::hex(if focused.get() {
            accent.get()
        } else {
            0x8E8E93
        })
    };
    let tint = day::tween::Tweened::new(target());
    let animated = tint.clone();
    day::reactive::watch(target, move |color, _| {
        animated.animate_to(*color, AnimSpec::ease_out(200));
    });
    let cleanup = tint.clone();
    // Day disposes child scopes before freeing this scope's signals. A cleanup
    // on the current scope would read an already-disposed tween during login.
    Scope::child().on_cleanup(move || cleanup.set(cleanup.get_untracked()));
    tint
}

/// Both eye images stay mounted; interrupted toggles reverse from the current opacity.
pub(super) fn animated_visibility(visible: Signal<bool>) -> day::tween::Tweened<f64> {
    let opacity = day::tween::Tweened::new(if visible.get() { 1.0 } else { 0.0 });
    let animated = opacity.clone();
    day::reactive::watch(
        move || visible.get(),
        move |visible, _| {
            animated.animate_to(if *visible { 1.0 } else { 0.0 }, AnimSpec::ease_out(200));
        },
    );
    let cleanup = opacity.clone();
    // Cancel while the owning scope's tween signal is still alive.
    Scope::child().on_cleanup(move || cleanup.set(cleanup.get_untracked()));
    opacity
}
