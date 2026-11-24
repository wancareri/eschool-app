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
    // Keep this a plain UIKit button until the final native tweak.  Calling
    // `.prominent()` here makes Day install its normal filled configuration
    // after the glass configuration during initial realization.
    let piece = button(title)
        .enabled(enabled)
        .action(action);
    // Native callbacks must attach to the button itself, before Day's
    // sizing/padding wrappers. Otherwise no UIKit button is reached.
    #[cfg(feature = "uikit")]
    let piece = piece.tweak(native_glass_auth_button);

    piece
        .id(day::id!(login_btn))
        .height(52.0)
        .grow(Dimension::Width)
        // Keep the declarative capsule for non-UIKit toolkits.  On UIKit the
        // final glass tweak releases the layer clip for the native effect.
        .corner_radius(26.0)
        .log_pos("Auth:ActionButton")
        .padding(Insets {
            top: 0.0,
            leading: 20.0,
            bottom: 8.0,
            trailing: 20.0,
        })
        .animation(AnimSpec::ease_out(200))
}

#[cfg(feature = "uikit")]
fn native_glass_auth_button(node: day::RNode) {
    // This runs after every Day sizing/decorating operation, so it is the
    // configuration UIKit actually displays rather than a style Day replaces.
    crate::components::modal::native_glass_button(node, true);
}

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
    crate::ios::autofill::sync_values();
    day::prefs::set(crate::LOCALE_KEY, locale);
    set_locale(locale);
    crate::ios::autofill::refresh_active();
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

pub(super) fn native_auth_input(
    field: impl Decorate,
    configure: impl FnOnce(day::RNode) + 'static,
) -> impl Piece {
    field.tweak(configure).grow(Dimension::Width)
}

pub(super) fn auth_keyboard_dock(action: impl Piece) -> impl Piece {
    let dock = column((action,)).spacing(0.0);
    #[cfg(feature = "uikit")]
    let dock = {
        use day_uikit::UiKitExt;
        dock.uikit(|view, _, _| crate::ios::keyboard::register_follower_view(view))
    };
    dock.grow(Dimension::Width)
}

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
    Scope::child().on_cleanup(move || cleanup.set(cleanup.get_untracked()));
    tint
}

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
    Scope::child().on_cleanup(move || cleanup.set(cleanup.get_untracked()));
    opacity
}
