//! Eschool App, a [Day](https://daybrite.dev) app.

use day::prelude::*;

pub mod app;
pub mod core;
pub mod model;
pub mod native;
pub mod ui;
pub mod util;

pub mod swiftui {
    include!(concat!(env!("OUT_DIR"), "/day_swiftui.rs"));
}

use crate::core::state::ESchoolState;
use crate::model::Scene;

day::day_start!(options: window(), root);

/// The window every entry point opens.
pub fn window() -> day::WindowOptions {
    day::WindowOptions {
        locales: Some((res::locales::DEFAULT, res::locales::CATALOG)),
        title_fn: Some(|| res::str::app_title().format()),
        size: day::prelude::Size::new(400.0, 720.0),
        min_size: Some(day::prelude::Size::new(320.0, 480.0)),
        ..Default::default()
    }
}

day::resources!();

const THEME_KEY: &str = "app.theme";
const LOCALE_KEY: &str = "app.locale";

day::routes! {
    pub(crate) enum Section {
        Login => "login",
        Diary => "diary",
        Schedule => "schedule",
        Teachers => "teachers",
        Settings => "settings",
    }
}

pub fn root() -> impl Piece {
    crate::core::nslog::nslog("[init] Eschool App starting");
    // On first launch, detect the system language and save it so the
    // language picker defaults to the device locale rather than English.
    let has_locale = day::prefs::get(LOCALE_KEY)
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    if !has_locale {
        let sys = crate::core::locale::system_locale();
        day::prefs::set(LOCALE_KEY, sys);
        crate::core::nslog::nslog(&format!("[init] First launch — system locale: {sys}"));
    }
    day_piece_settings::apply_startup(THEME_KEY, LOCALE_KEY);
    day::register_preferences(ui::pages::settings_body);
    day::register_new_window(|| window_shell(false));
    window_shell(true)
}

fn window_shell(primary: bool) -> impl Piece {
    // ESchoolState is scoped OUTSIDE Scene so every page can reach the shared
    // school data without passing it through Scene (which holds template state).
    ESchoolState::scoped(move |_state| {
        Scene::scoped(move |scene| {
            if primary {
                scene.persist();
            }
            day::window_title(move || res::str::app_title().format());
            app::build_nav(scene, primary)
        })
    })
}
