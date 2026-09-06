use day::prelude::*;

/// A number field with its own increment/decrement pair.
pub(crate) fn stepper(value: impl Binding<i64> + Copy) -> impl Piece {
    row((
        button("−")
            .action(move || value.write((value.peek() - 1).max(0)))
            .id("field-count-dec"),
        label(move || value.read().to_string())
            .tabular()
            .reserving("000")
            .id("field-count"),
        button("+")
            .action(move || value.write((value.peek() + 1).min(999)))
            .id("field-count-inc"),
    ))
    .spacing(8.0)
}
