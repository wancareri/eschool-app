use crate::app::AppState;
use crate::shared::{biometric, colors, nslog, pin};
use crate::res;
use day::prelude::*;

const PIN_LENGTH: usize = 4;

pub fn render(state: AppState) -> impl Piece {
    column((
        column((
            label(move || res::str::app_title().format())
                .font(Font::LargeTitle)
                .align(TextAlign::Center),

            label("Введите PIN-код")
                .font(Font::Subheadline)
                .secondary()
                .align(TextAlign::Center)
                .padding(Insets { top: 4.0, leading: 0.0, bottom: 0.0, trailing: 0.0 }),
        ))
        .spacing(4.0)
        .align(HAlign::Center),

        // PIN dots
        {
            let s = state;
            row((
                dot(0, s),
                dot(1, s),
                dot(2, s),
                dot(3, s),
            ))
            .spacing(20.0)
            .padding(Insets { top: 24.0, leading: 0.0, bottom: 8.0, trailing: 0.0 })
        }
        .align(VAlign::Center),

        // Error message
        when(
            move || state.pin_error.get(),
            || label("Неверный PIN-код")
                .font(Font::Caption)
                .color(colors::ERROR)
                .align(TextAlign::Center)
                .padding(Insets { top: 4.0, leading: 0.0, bottom: 0.0, trailing: 0.0 }),
        ),

        // Numpad
        numpad(state),

        // Face ID button
        when(
            move || biometric::is_available() && biometric::is_enabled(),
            move || {
                button("Face ID / Touch ID")
                    .action(move || {
                        state.pin_error.set(false);
                        state.pin_input.set(String::new());
                        crate::shared::biometric::authenticate_async(state);
                    })
                    .id("pin-biometric-btn")
                    .padding(Insets { top: 20.0, leading: 0.0, bottom: 0.0, trailing: 0.0 })
            },
        ),
    ))
    .spacing(8.0)
    .padding(Insets { top: 60.0, leading: 40.0, bottom: 40.0, trailing: 40.0 })
    .grow()
    .any()
}

fn dot(index: usize, state: AppState) -> impl Piece {
    let filled = move || {
        let input = state.pin_input.get();
        input.len() > index
    };
    let error = state.pin_error;

    button(move || {
        if error.get() { "●" } else if filled() { "●" } else { "○" }
    })
    .action(|| {})
    .id(format!("dot-{index}"))
    .frame(20.0, 20.0)
}

fn numpad(state: AppState) -> impl Piece {
    column((
        numpad_row(state, &["1", "2", "3"]),
        numpad_row(state, &["4", "5", "6"]),
        numpad_row(state, &["7", "8", "9"]),
        numpad_row(state, &["", "0", "⌫"]),
    ))
    .spacing(12.0)
    .align(HAlign::Center)
}

fn numpad_row(state: AppState, keys: &[&str]) -> impl Piece {
    let s1 = state;
    let s2 = state;
    let s3 = state;
    let k1 = keys[0];
    let k2 = keys[1];
    let k3 = keys[2];

    row((
        numpad_key(s1, k1),
        numpad_key(s2, k2),
        numpad_key(s3, k3),
    ))
    .spacing(16.0)
}

fn numpad_key(state: AppState, key: &str) -> impl Piece {
    let key_str = key.to_string();
    let key_clone = key_str.clone();
    let key_clone2 = key_str.clone();
    let s = state;

    if key_str.is_empty() {
        spacer().frame(72.0, 52.0).any()
    } else if key_str == "⌫" {
        button("⌫")
            .action(move || {
                let mut input = s.pin_input.get();
                if !input.is_empty() {
                    input.pop();
                    s.pin_input.set(input);
                    s.pin_error.set(false);
                }
            })
            .frame(72.0, 52.0)
            .id("pin-backspace")
            .any()
    } else {
        button(key_clone2.clone())
            .action(move || {
                let mut input = s.pin_input.get();
                if input.len() >= PIN_LENGTH {
                    return;
                }
                input.push_str(&key_clone);
                s.pin_input.set(input.clone());
                s.pin_error.set(false);

                // Auto-check when 4 digits entered
                if input.len() == PIN_LENGTH {
                    if pin::verify(&input) {
                        nslog::nslog("[PIN] Correct, unlocking");
                        s.pin_lock_active.set(false);
                        s.is_authenticated.set(true);
                        s.pin_input.set(String::new());
                    } else {
                        nslog::nslog("[PIN] Wrong");
                        s.pin_error.set(true);
                        let clear = s.pin_input.setter();
                        day::reactive::on_main(move || {
                            clear.set(String::new());
                        });
                    }
                }
            })
            .frame(72.0, 52.0)
            .id(format!("pin-key-{key_clone2}"))
            .any()
    }
}
