use day::prelude::*;
use crate::app::AppState;
use crate::shared::{log_bridge, nslog, locale};
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
    locale::remember_startup_locale();
    let has_locale = day::prefs::get("app.locale")
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    if !has_locale {
        let sys = locale::system_locale();
        day::prefs::set("app.locale", &sys);
        nslog::nslog(&format!("[init] First launch — system locale: {sys}"));
    }
    crate::pages::settings::apply_startup();
    crate::shared::colors::init_accent();
    #[cfg(target_os = "ios")]
    crate::shared::colors::apply_ios_tint(crate::shared::colors::accent_hex());
    #[cfg(target_os = "ios")]
    crate::shared::ios_nav::ios_impl::install_auth_menu_icons();
    #[cfg(target_os = "ios")]
    crate::widgets::modal::ios::install_presentation_hook();
    crate::shared::keyboard::init();
    crate::shared::layout_diag::init();
    day::register_preferences(pages::settings::settings_body);
    day::register_new_window(|| window_shell(false));

    window_shell(true)
}

fn window_shell(primary: bool) -> impl Piece {
    AppState::scoped(move |state| {
        state.register_main();
        day::window_title(move || res::str::app_title().format());

        #[cfg(target_os = "ios")]
        crate::shared::pin::install_lifecycle_observers();

        // Reactive: trigger load_all when is_authenticated transitions to true
        day::reactive::watch(
            move || state.is_authenticated.get(),
            move |&auth, old| {
                if auth && old != Some(&true) {
                    nslog::nslog("[App] is_authenticated changed true, triggering load_all");
                    crate::features::auth::start_background_refresh();
                    crate::features::grade_checker::start_grade_checker();
                    #[cfg(target_os = "ios")]
                    crate::shared::notifications::ios::request_permission();
                    crate::features::diary::load_all(state);
                }
            },
        );

        // `watch` skips its initial value; load an unlocked stored session once.
        if state.is_authenticated.get() {
            crate::features::auth::start_background_refresh();
            crate::features::grade_checker::start_grade_checker();
            #[cfg(target_os = "ios")]
            crate::shared::notifications::ios::request_permission();
            crate::features::diary::load_all(state);
        }

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

        // Keep pin_setup_active and pin_setup_route in sync
        day::reactive::watch(
            move || state.pin_setup_active.get(),
            move |&active, old| {
                if old.is_some() {
                    if active {
                        if state.pin_setup_route.get().is_none() {
                            state.pin_setup_route.set(Some("setup".to_string()));
                        }
                    } else if state.pin_setup_route.get().is_some() {
                        state.pin_setup_route.set(None);
                    }
                }
            },
        );
        day::reactive::watch(
            move || state.pin_setup_route.get(),
            move |route, old| {
                if old.is_some() {
                    let active = route.is_some();
                    if state.pin_setup_active.get() != active {
                        state.pin_setup_active.set(active);
                    }
                }
            },
        );

        #[cfg(target_os = "ios")]
        {
            day::reactive::on_main_delayed(200, move || {
                crate::shared::colors::apply_ios_tint(crate::shared::colors::accent_hex());
            });
            day::reactive::on_main_delayed(800, move || {
                crate::shared::colors::apply_ios_tint(crate::shared::colors::accent_hex());
            });

        }

        build_nav(primary)
    })
}

fn build_nav(primary: bool) -> impl Piece {
    let state = AppState::ambient();

    crate::widgets::notice::bind_error(state, state.error_msg, move |raw| {
        if !state.is_authenticated.get_untracked() && !state.setup_new_password_active.get_untracked() {
            pages::login::error_notice(raw)
        } else {
            let message = if raw.len() <= 240 && !raw.contains("http") && !raw.contains('{') {
                raw.to_owned()
            } else { res::str::notice_error_message().format() };
            crate::widgets::notice::Notice::error(res::str::notice_error_title().format(), message)
        }
    });

    // The status sheet is a cover: day never attaches a cover's view to this
    // page (it lives in its own modal VC, its node measures 0×0), so the
    // subview walk to the tab host still passes through a single-child chain
    // and the page keeps its full-bleed frame — the tab bar never moves. The
    // cover presents OverFullScreen (wancareri/day fork), so the page stays
    // visible beneath its translucent dim.
    zstack((
        nav_body(state, primary),
        crate::pages::pin_lock::modal(state),
        crate::widgets::grade_peek::modal(state),
        crate::widgets::lesson_note::modal(state),
        crate::widgets::lesson_peek::modal(state),
        crate::widgets::bottom_sheet::network_error_sheet(state),
        crate::widgets::notice::modal(state),
    ))
    .grow(Dimension::Both)
    .toolbar(move || crate::widgets::header::toolbar(state))
}

fn nav_body(state: AppState, primary: bool) -> impl Piece {
    #[derive(Clone, PartialEq)]
    enum Screen { Unlock, Tabs, Login }

    // One owner replaces the complete controller tree. Nested auth conditions
    // can tear down the login stack before its replacement is attached on UIKit.
    switch(
        move || {
            if state.pin_lock_active.get() { Screen::Unlock }
            else if state.is_authenticated.get() { Screen::Tabs }
            else { Screen::Login }
        },
        move |screen| match screen {
            Screen::Unlock => pages::pin_lock::render_unlock(state).any(),
            Screen::Tabs => render_nav_content(state, primary).any(),
            Screen::Login => pages::login::render().any(),
        },
    )
    .grow(Dimension::Both)
    .background(move || {
        if day::dark_mode() {
            Color::hex(0x000000)
        } else {
            Color::hex(0xFFFFFF)
        }
    })
}

fn diary_tab() -> impl Piece {
    let path = Signal::new(Vec::<String>::new());
    crate::widgets::header::page(res::str::diary_title(), path, pages::diary::render())
}

fn schedule_tab() -> impl Piece {
    let path = Signal::new(Vec::<String>::new());
    crate::widgets::header::page(res::str::schedule_title(), path, pages::schedule::render())
}

fn teachers_tab() -> impl Piece {
    let path = Signal::new(Vec::<String>::new());
    crate::widgets::header::page(res::str::teachers_title(), path, pages::teachers::render())
}

fn settings_tab() -> impl Piece {
    let path = Signal::new(Vec::<String>::new());
    crate::widgets::header::page(res::str::settings_title(), path, pages::settings::render())
}

fn render_nav_content(state: AppState, primary: bool) -> impl Piece {
    let section = state.current_section;
    #[cfg(target_os = "ios")]
    crate::shared::colors::apply_ios_tint(state.accent_color.get());

    let sel = nav(section)
        .style(day::prelude::NavStyle::Tabs)
        .items(item(crate::Section::Diary, res::str::nav_diary()).icon(res::vectors::tab_diary).page(diary_tab))
        .items(item(crate::Section::Schedule, res::str::nav_schedule()).icon(res::vectors::tab_schedule).page(schedule_tab))
        .items(item(crate::Section::Teachers, res::str::nav_teachers()).icon(res::vectors::tab_teachers).page(teachers_tab))
        .items(item(crate::Section::Settings, res::str::nav_settings()).icon(res::vectors::tab_settings).page(settings_tab));

    if primary {
        sel.title(move || crate::widgets::header::section_title(section.get())).id(day::id!(nav)).background(Color::rgba(0.0, 0.0, 0.0, 0.0)).any()
    } else {
        sel.local().title(move || crate::widgets::header::section_title(section.get())).id(day::id!(nav)).background(Color::rgba(0.0, 0.0, 0.0, 0.0)).any()
    }
}

#[cfg(all(test, feature = "mock"))]
mod tests {
    use super::*;
    use day_spec::props::NavPresentation;

    #[test]
    fn authentication_replaces_login_and_pin_trees_with_diary_tabs() {
        day_core::uninstall_tree();
        let (mock, probe) = day_mock::MockToolkit::new();
        day_core::launch_with(mock, day_spec::WindowOptions::default(), || {
            AppState::scoped(|state| {
                state.is_authenticated.set(false);
                state.pin_lock_active.set(false);
                nav_body(state, true)
            }).any()
        });
        let state = AppState::get_main().unwrap();
        let login_hosts: Vec<_> = probe.find_by_kind(day_spec::kinds::NAV)
            .into_iter().map(|(handle, _)| handle).collect();
        assert!(!login_hosts.is_empty());

        state.current_section.set(crate::Section::Diary);
        state.is_authenticated.set(true);
        day::reactive::flush_sync();
        let tabs = probe.find_by_kind(day_spec::kinds::NAV)
            .into_iter().filter(|(_, w)| w.presentation == Some(NavPresentation::Tabs))
            .collect::<Vec<_>>();
        assert_eq!(tabs.len(), 1, "login must mount the tab controller without restarting");
        assert_eq!(tabs[0].1.selected_page, Some(0), "Diary must be selected");
        for handle in login_hosts {
            assert!(!probe.state.borrow().widgets.contains_key(&handle.0),
                "outgoing login controllers must be removed");
        }

        state.pin_lock_active.set(true);
        day::reactive::flush_sync();
        assert!(!probe.state.borrow().widgets.contains_key(&tabs[0].0.0));
        state.pin_lock_active.set(false);
        day::reactive::flush_sync();
        assert_eq!(probe.find_by_kind(day_spec::kinds::NAV).iter()
            .filter(|(_, w)| w.presentation == Some(NavPresentation::Tabs)).count(), 1);

        state.is_authenticated.set(false);
        day::reactive::flush_sync();
        assert!(probe.find_by_kind(day_spec::kinds::NAV).iter()
            .all(|(_, w)| w.presentation != Some(NavPresentation::Tabs)));
        assert!(!probe.find_by_kind(day_spec::kinds::NAV).is_empty());
        day_core::uninstall_tree();
    }
}
