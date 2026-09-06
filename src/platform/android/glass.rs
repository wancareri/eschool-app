use day::prelude::*;

pub fn header(title: impl Into<String>, subtitle: impl Into<String>) -> AnyPiece {
    // TODO: Material Design 3 glass effect
    super::super::common::glass::header(title, subtitle)
}

pub fn card(title: impl Into<String>, value: impl Into<String>, icon: impl Into<String>) -> AnyPiece {
    // TODO: Material Design 3 card
    super::super::common::glass::card(title, value, icon)
}

pub fn button(title: impl Into<String>, icon: impl Into<String>) -> AnyPiece {
    // TODO: Material Design 3 button
    super::super::common::glass::button(title, icon)
}
