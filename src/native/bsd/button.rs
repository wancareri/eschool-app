use day::prelude::*;

pub fn render(title: impl Into<String>, icon: impl Into<String>) -> AnyPiece {
    super::super::linux::button::render(title, icon)
}
