use crate::model::{ItemFields, KINDS, Scene};
use crate::res;
use crate::ui::components::stepper::stepper;
use crate::util::{color_of, date_of, hex_of, iso_of};
use day::prelude::*;

/// The detail pane on a desktop: the selected row's editor, or the empty state.
pub(crate) fn editor_pane(scene: Scene) -> impl Piece {
    column((
        when(
            move || {
                scene
                    .selected
                    .get()
                    .is_none_or(|id| scene.find(id).is_none())
            },
            || {
                column((
                    spacer(),
                    label(res::str::item_none())
                        .font(Font::Title3)
                        .secondary()
                        .align(TextAlign::Center),
                    spacer(),
                ))
                .align(HAlign::Center)
                .grow()
                .padding(24.0)
                .grow()
            },
        ),
        each(
            items(
                move || {
                    scene
                        .selected
                        .get()
                        .filter(|id| scene.find(*id).is_some())
                        .into_iter()
                        .collect::<Vec<u32>>()
                },
                |id: &u32| *id,
            ),
            move |slot: ItemSlot<u32, u32>| editor(scene, slot.key()),
        ),
    ))
    .grow()
}

/// The editor: every standard two-way binding over one item, in a native form.
pub(crate) fn editor(scene: Scene, id: u32) -> impl Piece {
    let it = scene.items.elem(id as u64);

    form((
        section((
            labeled(
                res::str::field_name(),
                text_field(it.name())
                    .placeholder(res::str::field_name_hint())
                    .id("field-name"),
            ),
            labeled(res::str::field_count(), stepper(it.count())),
            labeled(
                res::str::field_date(),
                day_piece_datetime::date_picker(it.date().map(date_of, iso_of)).id("field-date"),
            ),
        ))
        .title(res::str::section_basics()),
        section((
            labeled(
                res::str::field_kind(),
                picker(KINDS.iter().map(|k| tr(k).format()), it.kind())
                    .segmented()
                    .id("field-kind"),
            ),
            labeled(res::str::field_done(), toggle(it.done()).id("field-done")),
            labeled(
                res::str::field_rating(),
                day_piece_rating::rating(it.rating())
                    .max(5)
                    .id("field-rating"),
            ),
            labeled(
                res::str::field_color(),
                day_piece_colorpicker::color_picker(it.color().map(color_of, hex_of))
                    .id("field-color"),
            ),
        ))
        .title(res::str::section_details()),
        section((text_area(it.notes()).min_lines(5).id("field-notes"),))
            .title(res::str::section_notes()),
    ))
    .padding(Insets {
        top: 0.0,
        leading: 16.0,
        bottom: 0.0,
        trailing: 16.0,
    })
}
