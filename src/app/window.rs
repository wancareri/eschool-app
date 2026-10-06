use day::prelude::*;
use crate::app::AppState;
use crate::shared::{locale, log_bridge, nslog};
use crate::pages;
use crate::res;

pub fn window() -> day::WindowOptions {
    log_bridge::install();
    day::WindowOptions {
        locales: Some((res::locales::DEFAULT, res::locales::CATALOG)),
        title_fn: Some(|| res::str::app_title().format()),
        size: day::prelude::Size::new(400.0, 720.0),
        min_size: Some(day::prelude::Size::new(320.0, 480.0)),
        ..Default::default()
    }
}

pub fn root() -> impl Piece {
    log_bridge::install();
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

        // Haptics: a settings page change (strip tap OR pager swipe — the strip
        // tick alone missed swipes).
        day::reactive::watch(
            move || state.settings_tab.get(),
            move |&t, old| {
                if old.is_some() && old != Some(&t) {
                    crate::shared::haptics::tick();
                }
            },
        );

        // Face ID when the lock drops (auto-lock): cold start prompts in the
        // block below; the PIN screen itself no longer carries a button.
        day::reactive::watch(
            move || state.pin_lock_active.get(),
            move |&locked, old| {
                if locked
                    && old == Some(&false)
                    && crate::shared::biometric::is_available()
                    && crate::shared::biometric::is_enabled()
                {
                    crate::shared::biometric::authenticate_async(state);
                }
            },
        );

        // Haptics: a tab-bar page change, an appearance flip, a language switch.
        {
            use crate::shared::haptics;
            day::reactive::watch(
                move || state.current_section.get(),
                move |sec, old| {
                    if old.is_some() && old != Some(&sec) {
                        haptics::tick();
                    }
                },
            );
            day::reactive::watch(
                move || day::dark_mode(),
                move |&_, old| {
                    if old.is_some() {
                        haptics::tick();
                    }
                },
            );
            day::reactive::watch(
                move || day::locale().get(),
                move |tag, old| {
                    if old.is_some() && old != Some(tag) {
                        haptics::tick();
                    }
                },
            );
        }

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

    // The status sheet is a cover: day never attaches a cover's view to this
    // page (it lives in its own modal VC, its node measures 0×0), so the
    // subview walk to the tab host still passes through a single-child chain
    // and the page keeps its full-bleed frame — the tab bar never moves. The
    // cover presents OverFullScreen (wancareri/day fork), so the page stays
    // visible beneath its translucent dim.
    zstack((
        nav_body(state, primary),
        crate::widgets::grade_peek::modal(state),
        crate::widgets::bottom_sheet::network_error_sheet(state),
    ))
    .grow()
}

fn nav_body(state: AppState, primary: bool) -> impl Piece {
    when(
        move || state.is_authenticated.get(),
        move || render_nav_content(state, primary),
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
    #[cfg(target_os = "ios")]
    crate::shared::colors::apply_ios_tint(state.accent_color.get());

    let sel = nav(section)
        .style(day::prelude::NavStyle::Tabs)
        .background(Color::clear())
        .blur(20.0)
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
        .item_icon(
            crate::Section::Schedule,
            res::str::nav_schedule(),
            res::vectors::tab_schedule,
            pages::schedule::render,
        )
        .item_icon(
            crate::Section::Teachers,
            res::str::nav_teachers(),
            res::vectors::tab_teachers,
            pages::teachers::render,
        )
        .item_icon(
            crate::Section::Settings,
            res::str::nav_settings(),
            res::vectors::tab_settings,
            pages::settings::render,
        );

    if primary {
        sel.id("nav").any()
    } else {
        sel.id("nav").local().any()
    }
}
