use day::prelude::*;
use crate::app::blur::ApplyGlassBlur;

pub mod button {
    use super::*;
    pub fn render(title: impl Into<String>, _icon: impl Into<String>) -> AnyPiece {
        day::prelude::button(title.into()).apply_glass_blur().any()
    }
}

pub mod card {
    use super::*;
    pub fn render(title: impl Into<String>, value: impl Into<String>, _icon: impl Into<String>) -> AnyPiece {
        let t = title.into();
        let v = value.into();
        column((
                label(v).font(Font::Title),
                label(t).font(Font::Caption),
            ))
            .spacing(4.0)
            .padding(12.0)
        .apply_glass_blur()
        .any()
    }
}

pub mod header {
    use super::*;
    pub fn render(title: impl Into<String>, subtitle: impl Into<String>) -> AnyPiece {
        column((label(title.into()).font(Font::Title), label(subtitle.into()).font(Font::Subheadline))).apply_glass_blur().any()
    }
}
