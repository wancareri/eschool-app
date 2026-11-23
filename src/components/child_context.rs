use crate::app::AppState;
use crate::app::preferences::haptics;
use day::prelude::*;

/// Shows and changes the child whose student data is currently displayed.
pub fn child_switcher(state: AppState) -> impl Piece {
    when(
        move || state.is_parent.get(),
        move || {
            let children = state.parent_children.get();
            if children.len() > 1 {
                let pills: Vec<AnyPiece> = children
                    .into_iter()
                    .enumerate()
                    .map(|(idx, child)| {
                        let name = child.full_name.clone();
                        button(move || {
                            if state.active_child_index.get() == idx {
                                format!("● {name}")
                            } else {
                                name.clone()
                            }
                        })
                        .action(move || {
                            haptics::tick();
                            crate::pages::diary::data::switch_child(state, idx);
                        })
                        .padding(Insets {
                            top: 4.0,
                            leading: 10.0,
                            bottom: 4.0,
                            trailing: 10.0,
                        })
                        .any()
                    })
                    .collect();
                row(pills)
                    .spacing(8.0)
                    .align(VAlign::Center)
                    .padding(Insets {
                        top: 2.0,
                        leading: 20.0,
                        bottom: 6.0,
                        trailing: 20.0,
                    })
                    .any()
            } else if let Some(child) = children.first() {
                let name = child.full_name.clone();
                label(move || {
                    format!(
                        "{}: {name}",
                        crate::res::str::parent_child_prefix().format()
                    )
                })
                .font(Font::Subheadline)
                .secondary()
                .align(TextAlign::Center)
                .padding(Insets {
                    top: 0.0,
                    leading: 20.0,
                    bottom: 4.0,
                    trailing: 20.0,
                })
                .any()
            } else {
                spacer().height(0.0).any()
            }
        },
    )
}
