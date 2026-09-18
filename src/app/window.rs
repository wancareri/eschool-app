use day::prelude::*;
use crate::app::AppState;
use crate::shared::{locale, nslog};
use crate::pages;
use crate::res;

pub fn window() -> day::WindowOptions {
    day::WindowOptions {
        locales: Some((res::locales::DEFAULT, res::locales::CATALOG)),
        title_fn: Some(|| res::str::app_title().format()),
        size: day::prelude::Size::new(400.0, 720.0),
        min_size: Some(day::prelude::Size::new(320.0, 480.0)),
        ..Default::default()
    }
}

pub fn root() -> impl Piece {
    nslog::nslog("[init] Eschool App starting");
    let has_locale = day::prefs::get("app.locale")
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    if !has_locale {
        let sys = locale::system_locale();
        day::prefs::set("app.locale", sys);
        nslog::nslog(&format!("[init] First launch — system locale: {sys}"));
    }
    day_piece_settings::apply_startup("app.theme", "app.locale");
    crate::shared::colors::init_accent();
    day::register_preferences(pages::settings::settings_body);
    day::register_new_window(|| window_shell(false));

    // Start background token refresh if authenticated
    let has_token = crate::shared::secure::load("auth.token").is_some();
    if has_token {
        crate::features::auth::start_background_refresh();
        crate::features::grade_checker::start_grade_checker();
        #[cfg(target_os = "ios")]
        crate::shared::notifications::ios::request_permission();
    }

    window_shell(true)
}

fn window_shell(primary: bool) -> impl Piece {
    AppState::scoped(move |state| {
        day::window_title(move || res::str::app_title().format());

        // Reactive: trigger load_all when is_authenticated transitions to true
        day::reactive::watch(
            move || state.is_authenticated.get(),
            move |&auth, old| {
                if auth && old != Some(&true) {
                    nslog::nslog("[App] is_authenticated changed true, triggering load_all");
                    crate::features::diary::load_all(state);
                }
            },
        );

        // Watch biometric signal — set by Setter from Face ID reply block
        day::reactive::watch(
            move || state.biometric_ok.get(),
            move |&ok, old| {
                if ok && old != Some(&true) {
                    nslog::nslog("[App] biometric_ok became true, setting is_authenticated");
                    state.is_authenticated.set(true);
                }
            },
        );

        // Biometric lock: if token exists + biometric enabled, prompt Face ID on startup
        if !state.is_authenticated.get()
            && crate::shared::secure::load("auth.token").is_some()
            && crate::shared::biometric::is_available()
            && crate::shared::biometric::is_enabled()
        {
            nslog::nslog("[App] Biometric lock: prompting Face ID on startup");
            crate::shared::biometric::authenticate_async(state);
        }

        build_nav(primary)
    })
}

fn build_nav(primary: bool) -> impl Piece {
    let state = AppState::ambient();
    let section = Signal::new(crate::Section::Diary);

    column((
        // Not authenticated → show login
        when(
            move || !state.is_authenticated.get(),
            move || pages::login::render().any(),
        ),
        // Authenticated → always show nav (even while loading)
        when(
            move || state.is_authenticated.get() && {
                let _ = state.accent_color.get();
                true
            },
            move || {
                let accent = Color::hex(state.accent_color.get());
                #[cfg(target_os = "ios")]
                crate::shared::colors::apply_ios_tint(state.accent_color.get());
                let sel = nav(section)
                    .title(move || {
                        if state.loading.get() {
                            format!("{}  ⏳", res::str::app_title().format())
                        } else {
                            res::str::app_title().format()
                        }
                    })
                    .sidebar_toggle(true)
                    .item_icon(
                        crate::Section::Diary,
                        res::str::nav_diary(),
                        res::vectors::tab_diary,
                        pages::diary::render,
                    )
                    .icon_tint(accent)
                    .item_icon(
                        crate::Section::Schedule,
                        res::str::nav_schedule(),
                        res::vectors::tab_schedule,
                        pages::schedule::render,
                    )
                    .icon_tint(accent)
                    .item_icon(
                        crate::Section::Teachers,
                        res::str::nav_teachers(),
                        res::vectors::tab_teachers,
                        pages::teachers::render,
                    )
                    .icon_tint(accent)
                    .item_icon(
                        crate::Section::Settings,
                        res::str::nav_settings(),
                        res::vectors::tab_settings,
                        pages::settings::render,
                    )
                    .icon_tint(accent);

                if primary {
                    sel.id("nav").restore("app.section").any()
                } else {
                    sel.id("nav").local().any()
                }
            },
        ),
    ))
    .grow()
    .any()
}
