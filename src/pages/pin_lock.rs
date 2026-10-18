use crate::app::AppState;
use crate::res;
use crate::shared::{colors, nslog, pin};
use day::prelude::*;

const PIN_LENGTH: usize = 4;

pub fn render(state: AppState) -> impl Piece {
    render_unlock(state)
}

pub fn render_unlock(state: AppState) -> impl Piece {
    column((
        spacer().height(8.0),

        column((
            label(move || res::str::app_title().format())
                .font(Font::LargeTitle)
                .bold()
                .align(TextAlign::Center),

            when(
                move || state.pin_error.get(),
                move || {
                    label("Неверный PIN-код")
                        .font(Font::Subheadline)
                        .color(colors::ERROR)
                        .align(TextAlign::Center)
                        .padding(Insets { top: 6.0, leading: 0.0, bottom: 0.0, trailing: 0.0 })
                },
            )
            .otherwise(move || {
                when(
                    move || state.pin_input.get().len() == PIN_LENGTH,
                    move || {
                        label("Добро пожаловать!")
                            .font(Font::Subheadline)
                            .color(Color::hex(state.accent_color.get()))
                            .align(TextAlign::Center)
                            .padding(Insets { top: 6.0, leading: 0.0, bottom: 0.0, trailing: 0.0 })
                    },
                )
                .otherwise(|| {
                    label("Введите PIN-код")
                        .font(Font::Subheadline)
                        .secondary()
                        .align(TextAlign::Center)
                        .padding(Insets { top: 6.0, leading: 0.0, bottom: 0.0, trailing: 0.0 })
                })
                .any()
            }),
        ))
        .spacing(4.0)
        .align(HAlign::Center),

        // PIN dots — the row rides the wrong-code shake
        {
            let s = state;
            let shake = shake_signal();
            row((
                dot(0, s),
                dot(1, s),
                dot(2, s),
                dot(3, s),
            ))
            .spacing(24.0)
            .padding(Insets { top: 20.0, leading: 0.0, bottom: 8.0, trailing: 0.0 })
            .align(VAlign::Center)
            .translation(move || shake.get(), 0.0)
        },

        // Numpad
        numpad(state),

        spacer().grow(),
    ))
    .spacing(8.0)
    .align(HAlign::Center)
    .padding(Insets {
        top: 16.0,
        leading: 24.0,
        bottom: 24.0,
        trailing: 24.0,
    })
    .grow()
    .any()
}

pub fn render_setup(state: AppState) -> impl Piece {
    let step = Signal::new(0u8); // 0 = first input, 1 = confirm
    let input = Signal::new(String::new());
    let first_pin = Signal::new(String::new());
    let error_msg = Signal::new(String::new());

    column((
        spacer().height(8.0),

        column((
            label("Установка PIN-кода")
                .font(Font::LargeTitle)
                .bold()
                .align(TextAlign::Center),

            when(
                move || !error_msg.get().is_empty(),
                move || {
                    label(move || error_msg.get())
                        .font(Font::Subheadline)
                        .color(colors::ERROR)
                        .align(TextAlign::Center)
                        .padding(Insets { top: 6.0, leading: 0.0, bottom: 0.0, trailing: 0.0 })
                },
            )
            .otherwise(move || {
                label(move || {
                    if step.get() == 0 {
                        "Придумайте 4-значный PIN-код"
                    } else {
                        "Повторите ваш PIN-код"
                    }
                })
                .font(Font::Subheadline)
                .secondary()
                .align(TextAlign::Center)
                .padding(Insets { top: 6.0, leading: 0.0, bottom: 0.0, trailing: 0.0 })
            }),
        ))
        .spacing(4.0)
        .align(HAlign::Center),

        // PIN dots
        {
            let shake = shake_signal();
            let inp = input;
            let err = error_msg;
            row((
                setup_dot(0, state, inp, err),
                setup_dot(1, state, inp, err),
                setup_dot(2, state, inp, err),
                setup_dot(3, state, inp, err),
            ))
            .spacing(24.0)
            .padding(Insets { top: 20.0, leading: 0.0, bottom: 8.0, trailing: 0.0 })
            .align(VAlign::Center)
            .translation(move || shake.get(), 0.0)
        },

        // Setup Numpad
        setup_numpad(state, step, input, first_pin, error_msg),

        // Cancel button
        button("Отмена")
            .action(move || {
                crate::shared::haptics::tick();
                state.pin_setup_active.set(false);
            })
            .id("pin-setup-cancel")
            .padding(Insets { top: 12.0, leading: 24.0, bottom: 0.0, trailing: 24.0 }),

        spacer().grow(),
    ))
    .spacing(8.0)
    .align(HAlign::Center)
    .padding(Insets {
        top: 16.0,
        leading: 24.0,
        bottom: 24.0,
        trailing: 24.0,
    })
    .grow()
    .any()
}

fn dot(index: usize, state: AppState) -> impl Piece {
    let filled = move || {
        let input = state.pin_input.get();
        input.len() > index
    };
    let error = state.pin_error;
    let s_accent = state;
    let clock = success_clock();

    when(
        move || error.get() || filled(),
        move || {
            circle()
                .fill(move || {
                    if error.get() {
                        colors::ERROR
                    } else {
                        Color::hex(s_accent.accent_color.get())
                    }
                })
                .frame(16.0, 16.0)
                .any()
        },
    )
    .otherwise(move || {
        circle()
            .stroke(move || Color::hex(s_accent.accent_color.get()), 1.5)
            .frame(16.0, 16.0)
            .any()
    })
    .scale(move || success_scale(index, clock.get()))
}

fn setup_dot(index: usize, state: AppState, input: Signal<String>, error_msg: Signal<String>) -> impl Piece {
    let filled = move || input.get().len() > index;
    let s_accent = state;
    let clock = success_clock();

    when(
        move || !error_msg.get().is_empty() || filled(),
        move || {
            circle()
                .fill(move || {
                    if !error_msg.get().is_empty() {
                        colors::ERROR
                    } else {
                        Color::hex(s_accent.accent_color.get())
                    }
                })
                .frame(16.0, 16.0)
                .any()
        },
    )
    .otherwise(move || {
        circle()
            .stroke(move || Color::hex(s_accent.accent_color.get()), 1.5)
            .frame(16.0, 16.0)
            .any()
    })
    .scale(move || success_scale(index, clock.get()))
}

thread_local! {
    static SHAKE: Signal<f64> = Signal::new(0.0);
    static SHAKE_GEN: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    static SUCCESS_CLOCK: Signal<f64> = Signal::new(-1.0);
    static SUCCESS_GEN: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
}

fn shake_signal() -> Signal<f64> {
    SHAKE.with(|s| *s)
}

const SHAKE_MS: u32 = 450;
const SHAKE_FRAME_MS: u32 = 16;
const SHAKE_AMPLITUDE: f64 = 3.0;
const SHAKE_PERIOD_MS: f64 = 80.0;

fn run_shake() {
    let generation = SHAKE_GEN.with(|g| g.fetch_add(1, std::sync::atomic::Ordering::SeqCst)) + 1;
    shake_signal().set(0.0);
    schedule_shake_frame(generation, std::time::Instant::now());
}

fn schedule_shake_frame(generation: u32, started: std::time::Instant) {
    day::reactive::on_main_delayed(SHAKE_FRAME_MS, move || {
        if SHAKE_GEN.with(|g| g.load(std::sync::atomic::Ordering::SeqCst)) != generation {
            return;
        }
        let elapsed = started.elapsed().as_millis() as u32;
        if elapsed >= SHAKE_MS {
            shake_signal().set(0.0);
            return;
        }
        let decay = 1.0 - elapsed as f64 / SHAKE_MS as f64;
        let phase = elapsed as f64 / SHAKE_PERIOD_MS * std::f64::consts::TAU;
        shake_signal().set(SHAKE_AMPLITUDE * decay * phase.sin());
        schedule_shake_frame(generation, started);
    });
}

fn shake_total_ms() -> u32 {
    SHAKE_MS + 80
}

const SUC_START_MS: f64 = 60.0;
const SUC_STAGGER_MS: f64 = 70.0;
const SUC_DOT_MS: f64 = 480.0;
const SUC_TOTAL_MS: f64 = SUC_START_MS + 3.0 * SUC_STAGGER_MS + SUC_DOT_MS;
const SUC_BUMP: f64 = 0.35;

fn success_clock() -> Signal<f64> {
    SUCCESS_CLOCK.with(|s| *s)
}

fn success_scale(index: usize, t: f64) -> f64 {
    if t < 0.0 {
        return 1.0;
    }
    let start = SUC_START_MS + index as f64 * SUC_STAGGER_MS;
    let p = (t - start) / SUC_DOT_MS;
    if !(0.0..1.0).contains(&p) {
        return 1.0;
    }
    1.0 + SUC_BUMP * (std::f64::consts::PI * p).sin()
}

fn run_success_pulse() {
    let generation = SUCCESS_GEN.with(|g| g.fetch_add(1, std::sync::atomic::Ordering::SeqCst)) + 1;
    success_clock().set(0.0);
    schedule_success_frame(generation, std::time::Instant::now());
}

fn schedule_success_frame(generation: u32, started: std::time::Instant) {
    day::reactive::on_main_delayed(16, move || {
        if SUCCESS_GEN.with(|g| g.load(std::sync::atomic::Ordering::SeqCst)) != generation {
            return;
        }
        let t = started.elapsed().as_millis() as f64;
        success_clock().set(t);
        if t < SUC_TOTAL_MS {
            schedule_success_frame(generation, started);
        }
    });
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
    .spacing(24.0)
    .align(VAlign::Center)
}

fn numpad_key(state: AppState, key: &str) -> impl Piece {
    let key_str = key.to_string();
    let key_clone = key_str.clone();
    let key_clone2 = key_str.clone();
    let s = state;

    if key_str.is_empty() {
        spacer().frame(72.0, 72.0).any()
    } else if key_str == "⌫" {
        button("⌫")
            .action(move || {
                crate::shared::haptics::tick();
                let mut input = s.pin_input.get();
                if input.len() >= PIN_LENGTH {
                    return;
                }
                if !input.is_empty() {
                    input.pop();
                    s.pin_input.set(input);
                    s.pin_error.set(false);
                }
            })
            .id("pin-backspace")
            .frame(72.0, 72.0)
            .any()
    } else {
        button(move || key_clone2.clone())
            .action(move || {
                crate::shared::haptics::tick();
                let mut input = s.pin_input.get();
                if input.len() >= PIN_LENGTH {
                    return;
                }
                input.push_str(&key_clone);
                s.pin_input.set(input.clone());
                s.pin_error.set(false);

                if input.len() == PIN_LENGTH {
                    if pin::verify(&input) {
                        nslog::nslog("[PIN] Correct, unlocking");
                        crate::shared::haptics::pop();
                        run_success_pulse();
                        let unlock = s.pin_lock_active.setter();
                        let auth = s.is_authenticated.setter();
                        let clear = s.pin_input.setter();
                        day::reactive::on_main_delayed(800, move || {
                            unlock.set(false);
                            auth.set(true);
                            clear.set(String::new());
                        });
                    } else {
                        nslog::nslog("[PIN] Wrong");
                        s.pin_error.set(true);
                        run_shake();
                        let clear = s.pin_input.setter();
                        let err = s.pin_error.setter();
                        day::reactive::on_main_delayed(shake_total_ms(), move || {
                            clear.set(String::new());
                            err.set(false);
                        });
                    }
                }
            })
            .id(format!("pin-key-{}", key_str))
            .frame(72.0, 72.0)
            .any()
    }
}

fn setup_numpad(
    state: AppState,
    step: Signal<u8>,
    input: Signal<String>,
    first_pin: Signal<String>,
    error_msg: Signal<String>,
) -> impl Piece {
    column((
        setup_numpad_row(state, step, input, first_pin, error_msg, &["1", "2", "3"]),
        setup_numpad_row(state, step, input, first_pin, error_msg, &["4", "5", "6"]),
        setup_numpad_row(state, step, input, first_pin, error_msg, &["7", "8", "9"]),
        setup_numpad_row(state, step, input, first_pin, error_msg, &["", "0", "⌫"]),
    ))
    .spacing(12.0)
    .align(HAlign::Center)
}

fn setup_numpad_row(
    state: AppState,
    step: Signal<u8>,
    input: Signal<String>,
    first_pin: Signal<String>,
    error_msg: Signal<String>,
    keys: &[&str],
) -> impl Piece {
    row((
        setup_numpad_key(state, step, input, first_pin, error_msg, keys[0]),
        setup_numpad_key(state, step, input, first_pin, error_msg, keys[1]),
        setup_numpad_key(state, step, input, first_pin, error_msg, keys[2]),
    ))
    .spacing(24.0)
    .align(VAlign::Center)
}

fn setup_numpad_key(
    state: AppState,
    step: Signal<u8>,
    input: Signal<String>,
    first_pin: Signal<String>,
    error_msg: Signal<String>,
    key: &str,
) -> impl Piece {
    let key_str = key.to_string();
    let key_clone = key_str.clone();
    let key_clone2 = key_str.clone();

    if key_str.is_empty() {
        spacer().frame(72.0, 72.0).any()
    } else if key_str == "⌫" {
        button("⌫")
            .action(move || {
                crate::shared::haptics::tick();
                let mut v = input.get();
                if !v.is_empty() {
                    v.pop();
                    input.set(v);
                    error_msg.set(String::new());
                }
            })
            .id("pin-setup-backspace")
            .frame(72.0, 72.0)
            .any()
    } else {
        button(move || key_clone2.clone())
            .action(move || {
                crate::shared::haptics::tick();
                let mut v = input.get();
                if v.len() >= PIN_LENGTH {
                    return;
                }
                v.push_str(&key_clone);
                input.set(v.clone());
                error_msg.set(String::new());

                if v.len() == PIN_LENGTH {
                    if step.get() == 0 {
                        // First entry complete, advance to confirmation
                        crate::shared::haptics::pop();
                        first_pin.set(v);
                        step.set(1);
                        input.set(String::new());
                    } else {
                        // Confirm entry
                        let expected = first_pin.get();
                        if v == expected {
                            // Match! Save pin and finish
                            crate::shared::haptics::pop();
                            run_success_pulse();
                            pin::save_pin(&v);
                            nslog::nslog("[PIN] New PIN set successfully");
                            let setup_active = state.pin_setup_active.setter();
                            day::reactive::on_main_delayed(800, move || {
                                setup_active.set(false);
                            });
                        } else {
                            // Mismatch! Shake and reset to step 0
                            nslog::nslog("[PIN] Mismatch in confirmation");
                            error_msg.set("PIN-коды не совпадают".into());
                            run_shake();
                            let inp = input.setter();
                            let st = step.setter();
                            let fp = first_pin.setter();
                            let err = error_msg.setter();
                            day::reactive::on_main_delayed(shake_total_ms(), move || {
                                inp.set(String::new());
                                fp.set(String::new());
                                st.set(0);
                                err.set(String::new());
                            });
                        }
                    }
                }
            })
            .id(format!("pin-setup-key-{}", key_str))
            .frame(72.0, 72.0)
            .any()
    }
}
