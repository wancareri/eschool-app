use crate::app::AppState;
use crate::res;
use crate::shared::{colors, nslog, pin};
use crate::shared::layout_diag::{debug_log_overlay, PosLogExt};
use day::prelude::*;
#[cfg(target_os = "ios")]
use day_uikit::UiKitExt;

pub fn ios_back_arrow_button(action: impl Fn() + 'static) -> impl Piece {
    let act = std::rc::Rc::new(action);
    let act_clone = act.clone();

    #[cfg(target_os = "ios")]
    {
        button("‹")
            .frame(44.0, 44.0)
            .corner_radius(22.0)
            .background(move || {
                if day::dark_mode() {
                    Color::rgba(1.0, 1.0, 1.0, 0.12)
                } else {
                    Color::rgba(0.0, 0.0, 0.0, 0.06)
                }
            })
            .action(move || {
                crate::shared::haptics::tick();
                act_clone();
            })
            .uikit(|view, _class, _mtm| {
                use objc2_foundation::NSString;
                use objc2_ui_kit::{UIButton, UIImage, UIControlState};
                if let Some(btn) = view.downcast_ref::<UIButton>() {
                    let name = NSString::from_str("chevron.backward");
                    if let Some(img) = UIImage::systemImageNamed(&name) {
                        btn.setImage_forState(Some(&img), UIControlState::Normal);
                        btn.setTitle_forState(None, UIControlState::Normal);
                    }
                }
            })
            .id("ios-back-arrow")
    }

    #[cfg(not(target_os = "ios"))]
    {
        button("‹ Назад")
            .frame(44.0, 44.0)
            .corner_radius(22.0)
            .background(move || {
                if day::dark_mode() {
                    Color::rgba(1.0, 1.0, 1.0, 0.12)
                } else {
                    Color::rgba(0.0, 0.0, 0.0, 0.06)
                }
            })
            .action(move || {
                crate::shared::haptics::tick();
                act_clone();
            })
            .id("ios-back-arrow")
    }
}

const PIN_LENGTH: usize = 4;

pub fn open_setup(state: AppState) {
    state.pin_setup_active.set(true);
    state.pin_setup_route.set(Some("setup".to_string()));
}

pub fn close_setup(state: AppState) {
    state.pin_setup_active.set(false);
    state.pin_setup_route.set(None);
}

pub fn modal(state: AppState) -> impl Piece {
    cover(
        state.pin_setup_route,
        move |_| {
            render_setup(state)
        },
    )
    .background(|_| {
        if day::dark_mode() {
            Color::hex(0x000000)
        } else {
            Color::hex(0xFFFFFF)
        }
    })
}

pub fn render(state: AppState) -> impl Piece {
    render_unlock(state)
}

pub fn render_unlock(state: AppState) -> impl Piece {
    let shake = Signal::new(0.0);
    let success_clock = Signal::new(-1.0);

    column((
        spacer().grow(),

        column((
            label(move || res::str::app_title().format())
                .font(Font::LargeTitle)
                .bold()
                .align(TextAlign::Center),

            when(
                move || state.pin_error.get(),
                move || {
                    let msg = if let Some(secs) = pin::remaining_lockout_secs() {
                        format!("Блокировка ввода ({secs} с)")
                    } else {
                        "Неверный PIN-код".to_string()
                    };
                    label(move || msg.clone())
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
            let clk = success_clock;
            row((
                dot(0, s, clk),
                dot(1, s, clk),
                dot(2, s, clk),
                dot(3, s, clk),
            ))
            .spacing(24.0)
            .padding(Insets { top: 20.0, leading: 0.0, bottom: 8.0, trailing: 0.0 })
            .align(VAlign::Center)
            .translation(move || shake.get(), 0.0)
            .log_pos("PinUnlock:DotsRow")
        },

        // Numpad
        numpad(state, shake, success_clock).log_pos("PinUnlock:Numpad"),

        spacer().grow(),
    ))
    .spacing(8.0)
    .align(HAlign::Center)
    .padding(Insets {
        top: 24.0,
        leading: 24.0,
        bottom: 24.0,
        trailing: 24.0,
    })
    .log_pos("PinUnlock:Root")
    .grow()
    .any()
}

pub fn render_setup(state: AppState) -> impl Piece {
    let step = Signal::new(0u8); // 0 = first input, 1 = confirm
    let input = Signal::new(String::new());
    let first_pin = Signal::new(String::new());
    let error_msg = Signal::new(String::new());
    let show_logs = Signal::new(false);
    let shake = Signal::new(0.0);
    let success_clock = Signal::new(-1.0);

    zstack((
        column((
            row((
                ios_back_arrow_button(move || {
                    crate::shared::haptics::tick();
                    close_setup(state);
                }),
                spacer().grow(),
                button("📋 Логи")
                    .action(move || {
                        crate::shared::haptics::tick();
                        show_logs.set(true);
                    })
                    .padding(Insets { top: 4.0, leading: 8.0, bottom: 4.0, trailing: 8.0 })
                    .background(Color::rgba(0.5, 0.5, 0.5, 0.2))
                    .corner_radius(12.0),
            ))
            .align(VAlign::Center)
            .grow_w()
            .log_pos("PinSetup:TopBar"),

            spacer().grow(),

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
            .align(HAlign::Center)
            .log_pos("PinSetup:Title"),

            // PIN dots
            {
                let inp = input;
                let err = error_msg;
                let s = state;
                let clk = success_clock;
                row((
                    setup_dot(0, s, inp, err, clk),
                    setup_dot(1, s, inp, err, clk),
                    setup_dot(2, s, inp, err, clk),
                    setup_dot(3, s, inp, err, clk),
                ))
                .spacing(24.0)
                .padding(Insets { top: 20.0, leading: 0.0, bottom: 8.0, trailing: 0.0 })
                .align(VAlign::Center)
                .translation(move || shake.get(), 0.0)
                .log_pos("PinSetup:DotsRow")
            },

            // Setup Numpad
            setup_numpad(state, step, input, first_pin, error_msg, shake, success_clock)
                .log_pos("PinSetup:Numpad"),

            // Cancel button
            button("Отмена")
                .action(move || {
                    crate::shared::haptics::tick();
                    close_setup(state);
                })
                .id("pin-setup-cancel")
                .log_pos("PinSetup:Cancel")
                .padding(Insets { top: 12.0, leading: 24.0, bottom: 0.0, trailing: 24.0 }),

            spacer().grow(),
        ))
        .spacing(8.0)
        .align(HAlign::Center)
        .padding(Insets {
            top: 24.0,
            leading: 24.0,
            bottom: 24.0,
            trailing: 24.0,
        })
        .background(move || {
            if day::dark_mode() {
                Color::hex(0x000000)
            } else {
                Color::hex(0xFFFFFF)
            }
        })
        .grow_w()
        .grow()
        .log_pos("PinSetup:Root"),

        when(
            move || show_logs.get(),
            move || debug_log_overlay(show_logs).animation(AnimSpec::ease_out(250)).any(),
        ),
    ))
    .grow()
    .any()
}

fn dot(index: usize, state: AppState, clock: Signal<f64>) -> impl Piece {
    let filled = move || {
        let input = state.pin_input.get();
        input.len() > index
    };
    let error = state.pin_error;
    let s_accent = state;

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

fn setup_dot(
    index: usize,
    state: AppState,
    input: Signal<String>,
    error_msg: Signal<String>,
    clock: Signal<f64>,
) -> impl Piece {
    let filled = move || input.get().len() > index;
    let s_accent = state;

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

static UNLOCK_SHAKE_GEN: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static UNLOCK_SUCCESS_GEN: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static SETUP_SHAKE_GEN: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static SETUP_SUCCESS_GEN: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

const SHAKE_MS: u32 = 450;
const SHAKE_FRAME_MS: u32 = 16;
const SHAKE_AMPLITUDE: f64 = 3.0;
const SHAKE_PERIOD_MS: f64 = 80.0;

fn run_shake(shake: Signal<f64>, gen_counter: &'static std::sync::atomic::AtomicU32) {
    let generation = gen_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
    shake.set(0.0);
    schedule_shake_frame(shake.setter(), gen_counter, generation, std::time::Instant::now());
}

fn schedule_shake_frame(
    shake: Setter<f64>,
    gen_counter: &'static std::sync::atomic::AtomicU32,
    generation: u32,
    started: std::time::Instant,
) {
    day::reactive::on_main_delayed(SHAKE_FRAME_MS, move || {
        if gen_counter.load(std::sync::atomic::Ordering::SeqCst) != generation {
            return;
        }
        let elapsed = started.elapsed().as_millis() as u32;
        if elapsed >= SHAKE_MS {
            shake.set(0.0);
            return;
        }
        let decay = 1.0 - elapsed as f64 / SHAKE_MS as f64;
        let phase = elapsed as f64 / SHAKE_PERIOD_MS * std::f64::consts::TAU;
        shake.set(SHAKE_AMPLITUDE * decay * phase.sin());
        schedule_shake_frame(shake, gen_counter, generation, started);
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

fn run_success_pulse(clock: Signal<f64>, gen_counter: &'static std::sync::atomic::AtomicU32) {
    let generation = gen_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
    clock.set(0.0);
    schedule_success_frame(clock.setter(), gen_counter, generation, std::time::Instant::now());
}

fn schedule_success_frame(
    clock: Setter<f64>,
    gen_counter: &'static std::sync::atomic::AtomicU32,
    generation: u32,
    started: std::time::Instant,
) {
    day::reactive::on_main_delayed(16, move || {
        if gen_counter.load(std::sync::atomic::Ordering::SeqCst) != generation {
            return;
        }
        let t = started.elapsed().as_millis() as f64;
        clock.set(t);
        if t < SUC_TOTAL_MS {
            schedule_success_frame(clock, gen_counter, generation, started);
        }
    });
}

fn numpad(state: AppState, shake: Signal<f64>, clock: Signal<f64>) -> impl Piece {
    column((
        numpad_row(state, shake, clock, &["1", "2", "3"]),
        numpad_row(state, shake, clock, &["4", "5", "6"]),
        numpad_row(state, shake, clock, &["7", "8", "9"]),
        numpad_row(state, shake, clock, &["", "0", "⌫"]),
    ))
    .spacing(12.0)
    .align(HAlign::Center)
}

fn numpad_row(state: AppState, shake: Signal<f64>, clock: Signal<f64>, keys: &[&str]) -> impl Piece {
    let s1 = state;
    let s2 = state;
    let s3 = state;
    let k1 = keys[0];
    let k2 = keys[1];
    let k3 = keys[2];

    row((
        numpad_key(s1, shake, clock, k1),
        numpad_key(s2, shake, clock, k2),
        numpad_key(s3, shake, clock, k3),
    ))
    .spacing(24.0)
    .align(VAlign::Center)
}

fn numpad_key(state: AppState, shake: Signal<f64>, clock: Signal<f64>, key: &str) -> impl Piece {
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
                if pin::remaining_lockout_secs().is_some() {
                    s.pin_error.set(true);
                    run_shake(shake, &UNLOCK_SHAKE_GEN);
                    return;
                }
                let mut input = s.pin_input.get();
                if input.len() >= PIN_LENGTH {
                    return;
                }
                input.push_str(&key_clone);
                s.pin_input.set(input.clone());
                s.pin_error.set(false);

                if input.len() == PIN_LENGTH {
                    if pin::verify(&input) {
                        crate::shared::haptics::pop();
                        run_success_pulse(clock, &UNLOCK_SUCCESS_GEN);
                        let unlock = s.pin_lock_active.setter();
                        let auth = s.is_authenticated.setter();
                        let clear = s.pin_input.setter();
                        day::reactive::on_main_delayed(800, move || {
                            unlock.set(false);
                            auth.set(true);
                            clear.set(String::new());
                        });
                    } else {
                        s.pin_error.set(true);
                        run_shake(shake, &UNLOCK_SHAKE_GEN);
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
    shake: Signal<f64>,
    clock: Signal<f64>,
) -> impl Piece {
    column((
        setup_numpad_row(state, step, input, first_pin, error_msg, shake, clock, &["1", "2", "3"]),
        setup_numpad_row(state, step, input, first_pin, error_msg, shake, clock, &["4", "5", "6"]),
        setup_numpad_row(state, step, input, first_pin, error_msg, shake, clock, &["7", "8", "9"]),
        setup_numpad_row(state, step, input, first_pin, error_msg, shake, clock, &["", "0", "⌫"]),
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
    shake: Signal<f64>,
    clock: Signal<f64>,
    keys: &[&str],
) -> impl Piece {
    row((
        setup_numpad_key(state, step, input, first_pin, error_msg, shake, clock, keys[0]),
        setup_numpad_key(state, step, input, first_pin, error_msg, shake, clock, keys[1]),
        setup_numpad_key(state, step, input, first_pin, error_msg, shake, clock, keys[2]),
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
    shake: Signal<f64>,
    clock: Signal<f64>,
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
                            run_success_pulse(clock, &SETUP_SUCCESS_GEN);
                            pin::save_pin(&v);
                            nslog::nslog("[PIN] New PIN set successfully");
                            let setup_active = state.pin_setup_active.setter();
                            let setup_route = state.pin_setup_route.setter();
                            day::reactive::on_main_delayed(800, move || {
                                setup_active.set(false);
                                setup_route.set(None);
                            });
                        } else {
                            // Mismatch! Shake and reset to step 0
                            nslog::nslog("[PIN] Mismatch in confirmation");
                            error_msg.set("PIN-коды не совпадают".into());
                            run_shake(shake, &SETUP_SHAKE_GEN);
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
