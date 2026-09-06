use day::prelude::*;

pub fn render(title: impl Into<String>, icon: impl Into<String>) -> AnyPiece {
    super::ios::button::render(title, icon)
}
