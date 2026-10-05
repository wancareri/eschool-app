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

/// The sticky top-left capsule as a page sibling: tagged `ANNOTATION_TAG` so
/// ios-uikit's bleed walk skips it — the page keeps its single-scroll chain and
/// full-bleed frame under the bars while this stays pinned outside the scroll.
pub fn page_overlay() -> impl Piece {
    let piece = render();
    #[cfg(target_os = "ios")]
    let piece = {
        use day_uikit::UiKitExt;
        piece.uikit(|view, _class, _mtm| {
            view.setTag(day_uikit::ANNOTATION_TAG);
        })
    };
    #[cfg(not(target_os = "ios"))]
    let piece = {
        piece
    };
    piece.padding(Insets {
        top: 16.0,
        leading: 16.0,
        bottom: 0.0,
        trailing: 0.0,
    })
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
        if state.island_state.get() == ConnStatus::Connecting { 1.0 } else { 0.0 },
    );

    // Watch for connection status updates. The raw write itself carries no
    // animation intent, so every visual step runs on the main queue under
    // `with_animation` (an in-progress drain swallows the intent otherwise):
    // the capsule tint eases with the spinner crossfade, the old wording
    // fades out untouched, the label copy swaps only once the text is
    // invisible, and the arrival mounts the label at parked opacity 0
    // before fading it in — nothing pops.
    watch(
        move || state.conn_status.get(),
        move |new_st, old_st| {
            if let Some(old) = old_st {
                if old != new_st {
                    // Kill any pending hide/collapse before arming new ones.
                    let _ = COLLAPSE_GEN.fetch_add(1, Ordering::SeqCst);
                    let connecting = *new_st == ConnStatus::Connecting;
                    let new = *new_st;
                    // Park hidden text at 0 synchronously — the label is not
                    // mounted, so this instant write is invisible, and the fade
                    // below must start from 0 to fade in.
                    if !connecting && !expanded.get() && state.island_text_opacity.get() != 0.0 {
                        state.island_text_opacity.set(0.0);
                    }
                    let sp = spinner_op.setter();
                    let exp = state.island_expanded.setter();
                    let txt = state.island_text_opacity.setter();
                    let st = state.island_state.setter();
                    let lb = state.island_label.setter();
                    if connecting {
                        // Refresh started: only the spinner crossfade and the
                        // capsule tint respond right away. The wording and the
                        // shape wait 300 ms — a reload that finishes inside
                        // that window never fades the text or collapses the
                        // island, so overlapping loads can't flap it.
                        day::reactive::on_main(move || {
                            with_animation(AnimSpec::ease_out(220), move || {
                                sp.set(1.0);
                                st.set(new);
                            });
                        });
                        let gen_id = COLLAPSE_GEN.fetch_add(1, Ordering::SeqCst) + 1;
                        day::reactive::on_main_delayed(300, move || {
                            if COLLAPSE_GEN.load(Ordering::SeqCst) == gen_id {
                                with_animation(AnimSpec::ease_out(130), move || {
                                    txt.set(0.0);
                                });
                                // Swap the wording only after the fade reached
                                // 0 (invisible), then shrink to the circle.
                                day::reactive::on_main_delayed(140, move || {
                                    if COLLAPSE_GEN.load(Ordering::SeqCst) == gen_id {
                                        lb.set(new);
                                        with_animation(AnimSpec::ease_out(220), move || {
                                            exp.set(false);
                                        });
                                    }
                                });
                            }
                        });
                    } else {
                        // Data arrived: mount first — the label enters at
                        // the parked opacity 0, mirrors update in the same
                        // animated batch (tint eases, glyph crossfades) —
                        // then fade the text in as a second, real change.
                        day::reactive::on_main(move || {
                            with_animation(AnimSpec::ease_out(220), move || {
                                sp.set(0.0);
                                st.set(new);
                                lb.set(new);
                                exp.set(true);
                            });
                            with_animation(AnimSpec::ease_out(220), move || {
                                txt.set(1.0);
                            });
                        });
                        schedule_collapse(state);
                    }
                }
            }
        },
    );

    let status_icon = when(
        move || state.island_state.get() == ConnStatus::Offline,
        || vector(res::vectors::status_offline).frame(13.0, 13.0).tint(Color::hex(0x8E8E93)).any(),
    )
    .otherwise(move || {
        when(
            move || state.island_state.get() == ConnStatus::Error,
            || vector(res::vectors::status_error).frame(13.0, 13.0).tint(Color::hex(0xEF4444)).any(),
        )
        .otherwise(move || {
            vector(res::vectors::status_ok)
                .frame(13.0, 13.0)
                .tint(move || match state.island_state.get() {
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
                label(move || match state.island_label.get() {
                    ConnStatus::Connecting => "Обновление…",
                    ConnStatus::Connected | ConnStatus::Idle => "Обновлено",
                    ConnStatus::Offline => "Оффлайн",
                    ConnStatus::Error => "Ошибка сети",
                })
                .font(Font::Caption2)
                .color(move || match state.island_label.get() {
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
        match state.island_state.get() {
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
    // Node-scoped fallback so the capsule's RESIZE animates even when the collapse's
    // with_animation ambient doesn't reach the turn-end layout (the snap-to-circle).
    // Mounts stay instant: day places a never-placed node without animation (day
    // 1f440116), so this no longer grows the capsule out of its corner the way
    // 0761b13 removed it for.
    .animation(Animation::ease_out(220))
    .on_tap(move || {
        crate::shared::nslog::nslog("[cover] status capsule tap: open");
        state
            .show_network_modal
            .set(Some(String::from("net")));
    });

    apply_glass_blur(content)
}
