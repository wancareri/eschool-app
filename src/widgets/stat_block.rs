use crate::app::AppState;
use day::prelude::*;

pub fn render(state: AppState, title: &'static str, value_fn: impl Fn() -> String + 'static) -> impl Piece {
    let s = state;
    column((
        // grow_w must sit on the zstack, not the label: GrowLayout stretches its
        // direct child to the flex share, and a stretched UILabel renders its text
        // flush-left on UIKit (TextAlign is a no-op there). The zstack keeps hug
        // content and OverlayLayout's default Center alignment centers it in the
        // grown frame — the column then centers the whole row.
        zstack((
            label(move || value_fn())
                .font(Font::Title2)
                .color(move || Color::hex(s.accent_color.get()))
                .align(TextAlign::Center),
        ))
        .grow_w(),
        zstack((
            label(title)
                .font(Font::Caption)
                .secondary()
                .align(TextAlign::Center),
        ))
        .grow_w(),
    ))
    .align(HAlign::Center)
    .spacing(2.0)
    .padding(Insets { top: 8.0, leading: 8.0, bottom: 8.0, trailing: 8.0 })
    .grow_w()
}
