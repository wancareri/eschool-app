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
        state.register_main();
        day::window_title(move || res::str::app_title().format());

        // Auto-lock: save background timestamp periodically + re-lock after 5 min
        #[cfg(target_os = "ios")]
        {
            use std::sync::atomic::{AtomicBool, Ordering};
            static BG_WATCHER_STARTED: AtomicBool = AtomicBool::new(false);
            if !BG_WATCHER_STARTED.swap(true, Ordering::Relaxed) {
                nslog::nslog("[App] Starting background timestamp saver (30s)");
                let lock_sig = state.pin_lock_active.setter();
                let auth_sig = state.is_authenticated.setter();
                std::thread::spawn(move || {
                    loop {
                        std::thread::sleep(std::time::Duration::from_secs(30));
                        crate::shared::pin::save_last_background();
                        if crate::shared::pin::is_enabled()
                            && crate::shared::pin::should_auto_lock()
                        {
                            nslog::nslog("[PIN] Auto-lock triggered");
                            auth_sig.set(false);
                            lock_sig.set(true);
                        }
                    }
                });
            }
        }

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
                    state.pin_lock_active.set(false);
                    state.is_authenticated.set(true);
                }
            },
        );

        // Biometric lock: if token exists + biometric enabled, prompt Face ID on startup
        // Works for both pure biometric lock and PIN+biometric combo
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
    let show_modal = state.show_network_modal;
    let s_modal = state;

    // Necessary: multi-child root zstack makes iOS content_frame pad by the home
    // inset and lifts the tab bar; single-child body keeps full-bleed layout.
    when(
        move || !show_modal.get(),
        move || nav_body(state, primary).any(),
    )
    .otherwise(move || {
        zstack((
            nav_body(s_modal, primary),
            crate::widgets::bottom_sheet::network_error_sheet(s_modal),
        ))
        .grow()
        .any()
    })
    .grow()
}

fn nav_body(state: AppState, primary: bool) -> impl Piece {
    let s_auth = state;
    let s_even = state;
    let s_odd = state;

    when(
        move || s_auth.is_authenticated.get(),
        move || {
            let s_tok1 = s_even;
            let s_tok2 = s_odd;
            column((
                when(
                    move || s_tok1.ui_reload_token.get() % 2 == 0,
                    move || render_nav_content(s_even, primary),
                ),
                when(
                    move || s_tok2.ui_reload_token.get() % 2 == 1,
                    move || render_nav_content(s_odd, primary),
                ),
            ))
            .grow()
            .any()
        },
    )
    .otherwise(move || {
        if state.pin_lock_active.get() {
            pages::pin_lock::render(state).any()
        } else {
            pages::login::render().any()
        }
    })
    .grow()
}

fn render_nav_content(state: AppState, primary: bool) -> impl Piece {
    let section = state.current_section;
    let accent = Color::hex(state.accent_color.get());
    #[cfg(target_os = "ios")]
    crate::shared::colors::apply_ios_tint(state.accent_color.get());

    day::reactive::watch(
        move || section.get(),
        move |&s, _| {
            let name = match s {
                crate::Section::Diary => "diary",
                crate::Section::Schedule => "schedule",
                crate::Section::Teachers => "teachers",
                crate::Section::Settings => "settings",
                _ => "diary",
            };
            day::prefs::set("app.section", name);
        },
    );

    let sel = nav(section)
        .style(day::prelude::NavStyle::Tabs)
        .title(move || {
            if state.loading.get() {
                format!("{}  ⏳", res::str::app_title().format())
            } else {
                res::str::app_title().format()
            }
        })
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
}
