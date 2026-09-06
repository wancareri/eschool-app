use day::prelude::*;

pub fn render(title: impl Into<String>, _icon: impl Into<String>) -> AnyPiece {
    let title_str = title.into();
    day::prelude::button(title_str).any()
}
