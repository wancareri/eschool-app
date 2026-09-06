//! Android-specific Material Design components.

use day::prelude::*;

pub fn glass_header(title: impl Into<String>, subtitle: impl Into<String>) -> AnyPiece {
    // TODO: Material Design 3 glass effect
    super::common::glass_header(title, subtitle)
}

pub fn glass_card(title: impl Into<String>, value: impl Into<String>, icon: impl Into<String>) -> AnyPiece {
    // TODO: Material Design 3 card
    super::common::glass_card(title, value, icon)
}

pub fn glass_button(title: impl Into<String>, icon: impl Into<String>) -> AnyPiece {
    // TODO: Material Design 3 button
    super::common::glass_button(title, icon)
}
