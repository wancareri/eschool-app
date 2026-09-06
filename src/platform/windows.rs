use day::prelude::*;

pub fn header(title: impl Into<String>, subtitle: impl Into<String>) -> AnyPiece {
    // TODO: WinUI 3 glass effect
    super::common::header(title, subtitle)
}

pub fn card(title: impl Into<String>, value: impl Into<String>, icon: impl Into<String>) -> AnyPiece {
    // TODO: WinUI 3 card
    super::common::card(title, value, icon)
}

pub fn button(title: impl Into<String>, icon: impl Into<String>) -> AnyPiece {
    // TODO: WinUI 3 button
    super::common::button(title, icon)
}
