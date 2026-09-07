use crate::model::{Item, ItemFields, Scene};
use crate::res;
use crate::ui::components::row::row_view;
use day::prelude::*;

/// The Navigate section's DETAIL — the editor for whichever row the content list has selected.
pub(crate) fn navigate_page() -> impl Piece {
    super::detail::editor_pane(Scene::ambient()).grow()
}

/// The content-list pane: the item list in its own column.
pub(crate) fn item_list_pane() -> impl Piece {
    item_list(Scene::ambient()).grow()
}

/// The list itself — one widget, every layout, driven straight by this window's STORE.
fn item_list(scene: Scene) -> impl Piece {
    list(
        scene.items.rows(move || scene.ordered_keys()),
        move |slot| row_view(scene, slot),
    )
    .row_height(RowHeight::Uniform(58.0))
    .on_selection(move |rows: Vec<Elem<Item>>| match rows.first() {
        Some(it) => scene.open(it.key() as u32),
        None => scene.clear_selection(),
    })
    .selected_rows(move || {
        scene
            .selected
            .get()
            .and_then(|id| scene.ordered_keys().iter().position(|k| *k == id as u64))
            .into_iter()
            .collect()
    })
    .scroll_to_row(scene.scroll_to)
    .reorderable(true)
    .on_reorder(move |from, to| scene.move_row(from, to))
    .deletable(true)
    .delete_label(res::str::cmd_delete().format())
    .on_delete(move |index| {
        if let Some(&k) = scene.ordered_keys().get(index) {
            scene.remove(k as u32);
        }
    })
    .id("item-list")
}

/// The pushed editor's navigation-bar title.
pub(crate) fn detail_title(scene: Scene) -> String {
    let name = scene
        .selected
        .get()
        .filter(|id| scene.find(*id).is_some())
        .map(|id| scene.items.elem(id as u64).name().read())
        .unwrap_or_default();
    if name.is_empty() {
        res::str::item_none().format()
    } else {
        name
    }
}
