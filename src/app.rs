use crate::core::colors;
use crate::model::Scene;
use crate::res;
use crate::Section;
use crate::ui::pages::*;
use day::prelude::*;

/// Whether this platform has a menu bar.
pub(crate) fn has_menu_bar() -> bool {
    capability(Cap::AppMenu) != Support::Unsupported
}

/// Run a command on the window that currently has FOCUS.
pub(crate) fn front(f: impl Fn(Scene) + 'static) -> impl Fn() + 'static {
    move || {
        if let Some(scene) = Scene::focused() {
            f(scene)
        }
    }
}

/// The desktop menu bar.
pub(crate) fn menus() -> Vec<MenuEntry> {
    vec![
        sub_menu(
            res::str::menu_file().format(),
            vec![
                menu_role(MenuRole::NewWindow),
                menu_separator(),
                menu_role(MenuRole::CloseWindow),
            ],
        ),
        sub_menu(
            res::str::menu_edit().format(),
            vec![
                menu_role(MenuRole::Cut),
                menu_role(MenuRole::Copy),
                menu_role(MenuRole::Paste),
                menu_role(MenuRole::SelectAll),
            ],
        ),
    ]
}

/// Build the navigation selector — four school tabs + optional settings.
pub(crate) fn build_nav(scene: Scene, primary: bool) -> AnyPiece {
    let sel = selector(scene.section)
        .title(res::str::app_title())
        .sidebar_toggle(true)
        .item_icon(
            Section::Diary,
            res::str::nav_diary(),
            res::vectors::tab_diary,
            diary_page,
        )
        .icon_tint(colors::NAV_DIARY)
        .item_icon(
            Section::Schedule,
            res::str::nav_schedule(),
            res::vectors::tab_schedule,
            schedule_page,
        )
        .icon_tint(colors::NAV_SCHEDULE)
        .item_icon(
            Section::Teachers,
            res::str::nav_teachers(),
            res::vectors::tab_teachers,
            teachers_page,
        )
        .icon_tint(colors::NAV_TEACHERS)
        .items(
            move || {
                if has_menu_bar() {
                    Vec::new()
                } else {
                    vec![Section::Settings]
                }
            },
            |s: &Section| {
                item(*s, res::str::nav_settings())
                    .icon(res::vectors::tab_settings)
                    .icon_tint(colors::NAV_SETTINGS)
            },
        )
        .destination(|_: &Section| settings_page())
        .id("nav");
    if primary {
        sel.restore("app.section").any()
    } else {
        sel.local().any()
    }
}
