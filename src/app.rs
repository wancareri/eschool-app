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
                menu_item(res::str::cmd_add().format())
                    .shortcut(Shortcut::new("n").shift())
                    .action(front(|scene| scene.new_item())),
                menu_separator(),
                menu_role(MenuRole::CloseWindow),
            ],
        ),
        sub_menu(
            res::str::menu_edit().format(),
            vec![
                menu_item(res::str::cmd_delete().format())
                    .shortcut(Shortcut::new("Delete"))
                    .action(front(|scene| scene.delete_selected())),
                menu_item(res::str::cmd_done().format())
                    .shortcut(Shortcut::new("d"))
                    .action(front(|scene| scene.done_selected())),
                menu_item(res::str::cmd_show_done().format())
                    .shortcut(Shortcut::new("h"))
                    .action(front(|scene| scene.show_done.update(|v| *v = !*v))),
                menu_separator(),
                menu_role(MenuRole::Cut),
                menu_role(MenuRole::Copy),
                menu_role(MenuRole::Paste),
                menu_role(MenuRole::SelectAll),
            ],
        ),
    ]
}

/// Build the navigation selector, already restored or local.
pub(crate) fn build_nav(scene: Scene, primary: bool) -> AnyPiece {
    let sel = selector(scene.section)
        .title(res::str::app_title())
        .content_list(item_list_pane)
        .content_list_width(320.0)
        .content_list_for(|s: &Section| matches!(s, Section::Navigate))
        .detail_visible(scene.detail_open)
        .detail_title(move || detail_title(scene))
        .sidebar_toggle(true)
        .toolbar(move || {
            vec![
                toolbar_toggle("tb-show-done", res::str::cmd_show_done(), scene.show_done)
                    .icon(Symbol::Filter)
                    .tooltip(res::str::cmd_show_done()),
                toolbar_button("tb-done", res::str::cmd_done())
                    .icon(Symbol::Check)
                    .tooltip(res::str::cmd_done())
                    .action(move || scene.done_selected()),
                toolbar_button("tb-add", res::str::cmd_add())
                    .icon(Symbol::Add)
                    .tooltip(res::str::cmd_add())
                    .action(move || scene.new_item()),
            ]
        })
        .item_icon(
            Section::Welcome,
            res::str::nav_welcome(),
            res::vectors::tab_welcome,
            welcome_page,
        )
        .icon_tint(Color::hex(0xF59E0B))
        .item_icon(
            Section::Navigate,
            res::str::nav_navigate(),
            res::vectors::tab_navigate,
            navigate_page,
        )
        .icon_tint(Color::hex(0x3B82F6))
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
                    .icon_tint(Color::hex(0x10B981))
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
