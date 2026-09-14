use day::prelude::*;

pub fn render(title: &str, subtitle: &str) -> impl Piece {
    let t = title.to_string();
    let s = subtitle.to_string();
    column((
        label(t.clone())
            .font(Font::LargeTitle),
        label(s.clone())
            .font(Font::Subheadline).secondary(),
    ))
    .spacing(6.0)
    .padding(Insets { top: 16.0, leading: 20.0, bottom: 8.0, trailing: 20.0 })
}
