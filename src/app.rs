use crate::core::colors;
use crate::core::state::ESchoolState;
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

/// Nav item keys for the reactive sidebar.
#[derive(Clone, PartialEq, Eq, Debug)]
enum NavEntry {
    Login,
    Diary,
    Schedule,
    Teachers,
    Settings,
}

/// Build the navigation selector — fully reactive to auth state.
pub(crate) fn build_nav(scene: Scene, primary: bool) -> AnyPiece {
    let state = ESchoolState::ambient();

    let sel = selector(scene.section)
        .title(res::str::app_title())
        .sidebar_toggle(true)
        .items(
            move || {
                let mut entries = vec![NavEntry::Login];
                if state.is_authenticated.get() {
                    entries.push(NavEntry::Diary);
                    entries.push(NavEntry::Schedule);
                    entries.push(NavEntry::Teachers);
                }
                if !has_menu_bar() {
                    entries.push(NavEntry::Settings);
                }
                entries
            },
            |entry: &NavEntry| match entry {
                NavEntry::Login => {
                    let is_auth = ESchoolState::ambient().is_authenticated.get();
                    item(Section::Login, res::str::nav_login())
                        .icon(res::vectors::tab_login)
                        .icon_tint(if is_auth { colors::SUCCESS } else { colors::PRIMARY })
                }
                NavEntry::Diary => {
                    item(Section::Diary, res::str::nav_diary())
                        .icon(res::vectors::tab_diary)
                        .icon_tint(colors::NAV_DIARY)
                }
                NavEntry::Schedule => {
                    item(Section::Schedule, res::str::nav_schedule())
                        .icon(res::vectors::tab_schedule)
                        .icon_tint(colors::NAV_SCHEDULE)
                }
                NavEntry::Teachers => {
                    item(Section::Teachers, res::str::nav_teachers())
                        .icon(res::vectors::tab_teachers)
                        .icon_tint(colors::NAV_TEACHERS)
                }
                NavEntry::Settings => {
                    item(Section::Settings, res::str::nav_settings())
                        .icon(res::vectors::tab_settings)
                        .icon_tint(colors::NAV_SETTINGS)
                }
            },
        )
        .destination(|section: &Section| match section {
            Section::Login => login_page().any(),
            Section::Diary => diary_page().any(),
            Section::Schedule => schedule_page().any(),
            Section::Teachers => teachers_page().any(),
            Section::Settings => settings_page().any(),
        });

    if primary {
        sel.id("nav").restore("app.section").any()
    } else {
        sel.id("nav").local().any()
    }
}
