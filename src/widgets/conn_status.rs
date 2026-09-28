/// Dynamic Island connection status indicator — frosted glass capsule with Lucide icons.
/// Status changes crossfade spinner↔icon, stretch/shrink the capsule and fade the text.
use std::sync::atomic::{AtomicU64, Ordering};
use crate::app::{AppState, ConnStatus};
use crate::res;
use day::prelude::*;

static COLLAPSE_GEN: AtomicU64 = AtomicU64::new(0);

fn collapse_island(setter_exp: Setter<bool>, setter_op: Setter<f64>) {
    with_animation(Animation::ease_out(130), move || {
        setter_op.set(0.0);
    });
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(140));
        day::reactive::on_main(move || {
            with_animation(Animation::ease_out(220), move || {
                setter_exp.set(false);
            });
        });
    });
}

fn schedule_collapse(state: AppState) {
    // Setters (Send) cross the thread boundary — AppState/Signal cannot.
    let setter_exp = state.island_expanded.setter();
    let setter_op = state.island_text_opacity.setter();
    let gen_id = COLLAPSE_GEN.fetch_add(1, Ordering::SeqCst) + 1;
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(3000));
        if COLLAPSE_GEN.load(Ordering::SeqCst) == gen_id {
            day::reactive::on_main(move || {
                collapse_island(setter_exp, setter_op);
            });
        }
    });
}

#[cfg(target_os = "ios")]
fn apply_glass_blur(piece: impl Decorate) -> impl Piece {
    use day_uikit::UiKitExt;
    use objc2::MainThreadOnly;
    use objc2::rc::Retained;
    use objc2_quartz_core::CALayer;
    use objc2_ui_kit::{UIBlurEffect, UIBlurEffectStyle, UIVisualEffectView, UIViewAutoresizing};

    piece.uikit(|view, _class, mtm| {
        let tag: objc2::ffi::NSInteger = 9991;
        if view.viewWithTag(tag).is_some() {
            return;
        }

        let blur = UIBlurEffect::effectWithStyle(
            UIBlurEffectStyle::SystemUltraThinMaterial,
            mtm,
        );
        let effect_view = UIVisualEffectView::initWithEffect(
            UIVisualEffectView::alloc(mtm),
            Some(&blur),
        );
        effect_view.setTag(tag);
        effect_view.setFrame(view.bounds());
        effect_view.setAutoresizingMask(UIViewAutoresizing(
            UIViewAutoresizing::FlexibleWidth.0 | UIViewAutoresizing::FlexibleHeight.0,
        ));
        effect_view.setUserInteractionEnabled(false);

        let view_layer: Retained<CALayer> = view.layer();
        view_layer.setCornerRadius(12.5);
        view_layer.setMasksToBounds(true);

        view.insertSubview_atIndex(&effect_view, 0);
        view.setClipsToBounds(true);
    })
}

#[cfg(not(target_os = "ios"))]
fn apply_glass_blur(piece: impl Decorate) -> impl Piece {
    piece
}

fn accent_dark_bg(hex: u32, alpha: f64) -> Color {
    let r = (((hex >> 16) & 0xFF) as f64 / 255.0) * 0.70;
    let g = (((hex >> 8) & 0xFF) as f64 / 255.0) * 0.70;
    let b = ((hex & 0xFF) as f64 / 255.0) * 0.70;
    Color::rgba(r, g, b, alpha)
}

pub fn render() -> impl Piece {
    let state = AppState::ambient();

    // Shared signals: all four mounted instances show one state — a change
    // made on one tab is already applied when another tab comes back.
    let expanded = state.island_expanded;
    let text_opacity = state.island_text_opacity;
    // Spinner opacity for the crossfade: 1 = loader, 0 = status icon.
    let spinner_op: Signal<f64> = Signal::new(
        if state.conn_status.get() == ConnStatus::Connecting { 1.0 } else { 0.0 },
    );

    // Watch for connection status updates: a refresh fades the text out and
    // shrinks back to the circle with the spinner; arrival stretches the capsule
    // open, fades the text in and crossfades the spinner into the icon. Nothing
    // on mount: the island starts compact, so the initial callback
    // (old_st == None) must not count as a change.
    watch(
        move || state.conn_status.get(),
        move |new_st, old_st| {
            if let Some(old) = old_st {
                if old != new_st {
                    // Kill any pending hide/collapse before arming new ones.
                    let _ = COLLAPSE_GEN.fetch_add(1, Ordering::SeqCst);
                    if *new_st == ConnStatus::Connecting {
                        // Refresh started: the text fades out, the icon
                        // crossfades into the spinner, and once the text is gone
                        // the capsule shrinks back to the circle.
                        with_animation(AnimSpec::ease_out(130), move || {
                            state.island_text_opacity.set(0.0);
                        });
                        with_animation(AnimSpec::ease_out(220), move || {
                            spinner_op.set(1.0);
                        });
                        let setter_exp = state.island_expanded.setter();
                        let gen_id = COLLAPSE_GEN.fetch_add(1, Ordering::SeqCst) + 1;
                        day::reactive::on_main_delayed(140, move || {
                            if COLLAPSE_GEN.load(Ordering::SeqCst) == gen_id {
                                with_animation(AnimSpec::ease_out(220), move || {
                                    setter_exp.set(false);
                                });
                            }
                        });
                    } else {
                        // Data arrived. Park hidden text at 0 first (safe: the
                        // label is not mounted), then — one ease-out — open the
                        // capsule (which inserts the label at 0) and fade the
                        // text in with the stretch while the spinner crossfades
                        // into the icon.
                        if !expanded.get() && state.island_text_opacity.get() != 0.0 {
                            state.island_text_opacity.set(0.0);
                        }
                        with_animation(AnimSpec::ease_out(220), move || {
                            spinner_op.set(0.0);
                            state.island_expanded.set(true);
                            state.island_text_opacity.set(1.0);
                        });
                        schedule_collapse(state);
                    }
                }
            }
        },
    );

    let status_icon = when(
        move || state.conn_status.get() == ConnStatus::Offline,
        || vector(res::vectors::status_offline).frame(13.0, 13.0).tint(Color::hex(0x8E8E93)).any(),
    )
    .otherwise(move || {
        when(
            move || state.conn_status.get() == ConnStatus::Error,
            || vector(res::vectors::status_error).frame(13.0, 13.0).tint(Color::hex(0xEF4444)).any(),
        )
        .otherwise(move || {
            vector(res::vectors::status_ok)
                .frame(13.0, 13.0)
                .tint(move || match state.conn_status.get() {
                    ConnStatus::Connecting | ConnStatus::Connected | ConnStatus::Idle => {
                        Color::hex(state.accent_color.get())
                    }
                    ConnStatus::Offline => Color::hex(0x8E8E93),
                    ConnStatus::Error => Color::hex(0xEF4444),
                })
                .any()
        })
    });

    // Both glyphs share the 13pt slot and crossfade on every status change:
    // spinner_op 1 = loader, 0 = status icon (inverse opacity).
    let icon_piece = zstack((
        super::spinner::render_gated(state, 13.0, move || spinner_op.get() > 0.0)
            .opacity(move || spinner_op.get())
            .any(),
        status_icon
            .opacity(move || 1.0 - spinner_op.get())
            .any(),
    ));

    let content = row((
        icon_piece,
        when(
            move || expanded.get(),
            move || {
                label(move || match state.conn_status.get() {
                    ConnStatus::Connecting => "Обновление…",
                    ConnStatus::Connected | ConnStatus::Idle => "Обновлено",
                    ConnStatus::Offline => "Оффлайн",
                    ConnStatus::Error => "Ошибка сети",
                })
                .font(Font::Caption2)
                .color(move || match state.conn_status.get() {
                    ConnStatus::Connecting | ConnStatus::Connected | ConnStatus::Idle => Color::hex(state.accent_color.get()),
                    ConnStatus::Offline => Color::hex(0x8E8E93),
                    ConnStatus::Error => Color::hex(0xEF4444),
                })
                .opacity(move || text_opacity.get())
                .padding(Insets { top: 0.0, leading: 6.0, bottom: 0.0, trailing: 3.0 })
            },
        ),
    ))
    .align(VAlign::Center)
    .padding(Insets { top: 6.0, leading: 6.0, bottom: 6.0, trailing: 6.0 })
    .background(move || {
        let dark = day::dark_mode();
        match state.conn_status.get() {
            ConnStatus::Connecting | ConnStatus::Connected | ConnStatus::Idle => {
                let accent = state.accent_color.get();
                if dark {
                    accent_dark_bg(accent, 0.50)
                } else {
                    Color::hex(accent).with_alpha(0.16)
                }
            }
            ConnStatus::Offline => {
                if dark {
                    Color::rgba(0.28, 0.28, 0.28, 0.32)
                } else {
                    Color::rgba(0.55, 0.55, 0.55, 0.18)
                }
            }
            ConnStatus::Error => {
                if dark {
                    Color::rgba(0.70, 0.15, 0.15, 0.50)
                } else {
                    Color::rgba(0.94, 0.27, 0.27, 0.16)
                }
            }
        }
    })
    // Half of the 25 pt box: CALayer does not clamp a larger radius, and 14 on
    // 25 overshoots into a self-intersecting lens (the "eye").
    .corner_radius(12.5)
    .on_tap(move || {
        state
            .show_network_modal
            .set(Some(String::from("net")));
    });

    apply_glass_blur(content)
}
