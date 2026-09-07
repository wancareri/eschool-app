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

use crate::model::Scene;

day::day_start!(options: window(), root);

/// The window every entry point opens.
pub fn window() -> day::WindowOptions {
    day::WindowOptions {
        locales: Some((res::locales::DEFAULT, res::locales::CATALOG)),
        title_fn: Some(|| res::str::app_title().format()),
        size: day::prelude::Size::new(960.0, 640.0),
        ..Default::default()
    }
}

day::resources!();

const THEME_KEY: &str = "app.theme";
const LOCALE_KEY: &str = "app.locale";

day::routes! {
    pub(crate) enum Section {
        Welcome => "welcome",
        Diary => "diary",
        Schedule => "schedule",
        Teachers => "teachers",
        Settings => "settings",
    }
}

pub fn root() -> impl Piece {
    info!("Eschool App starting");
    day_piece_settings::apply_startup(THEME_KEY, LOCALE_KEY);
    day::register_preferences(ui::pages::settings_body);
    day::register_new_window(|| window_shell(false));
    app_menu(app::menus());
    window_shell(true)
}

fn window_shell(primary: bool) -> impl Piece {
    Scene::scoped(move |scene| {
        if primary {
            scene.persist();
        }
        day::window_title(
            move || match scene.selected.get().and_then(|id| scene.find(id)) {
                Some(item) if !item.name.is_empty() => item.name,
                _ => res::str::app_title().format(),
            },
        );
        app::build_nav(scene, primary)
    })
}
