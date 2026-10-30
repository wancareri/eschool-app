use day::prelude::*;

pub mod button {
    use super::*;
    pub fn render(title: impl Into<String>, _icon: impl Into<String>) -> AnyPiece {
        day::prelude::button(title.into()).glass().any()
    }
}

pub mod card {
    use super::*;
    pub fn render(title: impl Into<String>, value: impl Into<String>, _icon: impl Into<String>) -> AnyPiece {
        let t = title.into();
        let v = value.into();
        card(
            column((
                label(v).font(Font::Title),
                label(t).font(Font::Caption),
            ))
            .spacing(4.0)
            .padding(12.0),
        )
        .glass()
        .any()
    }
}

pub mod header {
    use super::*;
    pub fn render(title: impl Into<String>, subtitle: impl Into<String>) -> AnyPiece {
        header(title.into(), subtitle.into()).glass().any()
    }
}
