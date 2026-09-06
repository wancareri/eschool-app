use day::prelude::*;

pub fn render(title: impl Into<String>, icon: impl Into<String>) -> AnyPiece {
    // TODO: WinUI 3 button
    super::super::common::button::render(title, icon)
}
