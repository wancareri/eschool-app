/// Dynamic Island connection status indicator — frosted glass capsule with Lucide icons and smooth animations.
use std::sync::atomic::{AtomicU64, Ordering};
use crate::app::{AppState, ConnStatus};
use crate::res;
use day::prelude::*;

static COLLAPSE_GEN: AtomicU64 = AtomicU64::new(0);

fn expand_island(state: AppState) {
    let exp = state.island_expanded.setter();
    let op = state.island_text_opacity.setter();
    let sc = state.island_scale.setter();
    let fx = state.island_fx.setter();
    // Masked pop-in: the capsule starts invisible (fx 0) and scaled 0.85, so
    // the width can reach its final value without the corner-growth artifact;
    // the fade+scale that follows reveals it from its own center.
    with_animation(AnimSpec::ease_out(0), move || {
        exp.set(true);
        sc.set(0.85);
        fx.set(0.0);
        op.set(1.0);
    });
    with_animation(AnimSpec::ease_out(220), move || {
        sc.set(1.0);
        fx.set(1.0);
    });
}

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
        view_layer.setCornerRadius(14.0);
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

    // Watch for connection status updates to reveal the island (full → collapse
    // after 3 s). Nothing on mount: the island starts compact, so the initial
    // callback (old_st == None) must not count as a change.
    watch(
        move || state.conn_status.get(),
        move |new_st, old_st| {
            if let Some(old) = old_st {
                if old != new_st {
                    expand_island(state);
                    schedule_collapse(state);
                }
            }
        },
    );

    let icon_piece = when(
        move || state.conn_status.get() == ConnStatus::Connecting,
        move || super::spinner::render(state, 10.0).any(),
    )
    .otherwise(move || {
        when(
            move || state.conn_status.get() == ConnStatus::Offline,
            || vector(res::vectors::status_offline).frame(13.0, 13.0).tint(Color::hex(0x8E8E93)).any(),
        )
        .otherwise(move || {
            when(
                move || state.conn_status.get() == ConnStatus::Error,
                || vector(res::vectors::status_error).frame(13.0, 13.0).tint(Color::hex(0xEF4444)).any(),
            )
            .otherwise(move || {
                let st = state.conn_status.get();
                let tint_color = match st {
                    ConnStatus::Connecting | ConnStatus::Connected | ConnStatus::Idle => Color::hex(state.accent_color.get()),
                    ConnStatus::Offline => Color::hex(0x8E8E93),
                    ConnStatus::Error => Color::hex(0xEF4444),
                };
                vector(res::vectors::status_ok).frame(13.0, 13.0).tint(tint_color).any()
            })
        })
    });

    let content = row((
        icon_piece,
        when(
            move || expanded.get(),
            move || {
                label(move || match state.conn_status.get() {
                    ConnStatus::Connecting => "Обновление…",
                    ConnStatus::Connected => "Подключено",
                    ConnStatus::Idle => "В сети",
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
    .padding(Insets { top: 6.0, leading: 9.0, bottom: 6.0, trailing: 9.0 })
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
    .corner_radius(14.0)
    .animation(Animation::ease_out(220))
    .scale(move || state.island_scale.get())
    .opacity(move || state.island_fx.get())
    .on_tap(move || {
        state
            .show_network_modal
            .set(Some(String::from("net")));
    });

    apply_glass_blur(content)
}
