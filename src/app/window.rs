use day::prelude::*;
use crate::app::AppState;
use crate::shared::{colors, locale, nslog};
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
    let has_token = day::prefs::get("auth.token")
        .map(|t| !t.is_empty())
        .unwrap_or(false);
    if has_token {
        crate::features::auth::start_background_refresh();
    }

    window_shell(true)
}

fn window_shell(primary: bool) -> impl Piece {
    AppState::scoped(move |_state| {
        day::window_title(move || res::str::app_title().format());
        build_nav(primary)
    })
}

fn build_nav(primary: bool) -> impl Piece {
    let state = AppState::ambient();
    let section = Signal::new(crate::Section::Diary);

    column((
        when(
            move || !state.is_authenticated.get(),
            move || pages::login::render().any(),
        ),
        when(
            move || state.is_authenticated.get(),
            move || {
                let sel = nav(section)
                    .title(res::str::app_title())
                    .sidebar_toggle(true)
                    .item_icon(
                        crate::Section::Diary,
                        res::str::nav_diary(),
                        res::vectors::tab_diary,
                        pages::diary::render,
                    )
                    .icon_tint(colors::NAV_DIARY)
                    .item_icon(
                        crate::Section::Schedule,
                        res::str::nav_schedule(),
                        res::vectors::tab_schedule,
                        pages::schedule::render,
                    )
                    .icon_tint(colors::NAV_SCHEDULE)
                    .item_icon(
                        crate::Section::Teachers,
                        res::str::nav_teachers(),
                        res::vectors::tab_teachers,
                        pages::teachers::render,
                    )
                    .icon_tint(colors::NAV_TEACHERS)
                    .item_icon(
                        crate::Section::Settings,
                        res::str::nav_settings(),
                        res::vectors::tab_settings,
                        pages::settings::render,
                    )
                    .icon_tint(colors::NAV_SETTINGS);

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
