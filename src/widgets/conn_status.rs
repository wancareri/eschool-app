/// Dynamic Island connection status indicator — frosted glass capsule with Lucide icons and smooth animations.
use std::sync::atomic::{AtomicU64, Ordering};
use crate::app::{AppState, ConnStatus};
use crate::res;
use day::prelude::*;

static COLLAPSE_GEN: AtomicU64 = AtomicU64::new(0);

fn expand_island(setter_exp: Setter<bool>, setter_op: Setter<f64>) {
    with_animation(Animation::ease_out(220), move || {
        setter_exp.set(true);
    });
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(40));
        day::reactive::on_main(move || {
            with_animation(Animation::ease_out(180), move || {
                setter_op.set(1.0);
            });
        });
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

fn schedule_collapse(setter_exp: Setter<bool>, setter_op: Setter<f64>) {
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
    let expanded = Signal::new(true);
    let text_opacity = Signal::new(1.0);
    let setter_exp = expanded.setter();
    let setter_op = text_opacity.setter();

    // Initial collapse after 3 seconds
    schedule_collapse(setter_exp, setter_op);

    // Watch for connection status updates to re-expand island
    watch(
        move || state.conn_status.get(),
        move |new_st, old_st| {
            if old_st != Some(new_st) {
                expand_island(setter_exp, setter_op);
                schedule_collapse(setter_exp, setter_op);
            }
        },
    );

    let icon_piece = when(
        move || state.conn_status.get() == ConnStatus::Connecting,
        move || super::spinner::render(state, 13.0).any(),
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
    .background(move || match state.conn_status.get() {
        ConnStatus::Connecting | ConnStatus::Connected | ConnStatus::Idle => accent_dark_bg(state.accent_color.get(), 0.50),
        ConnStatus::Offline => Color::rgba(0.28, 0.28, 0.28, 0.32),
        ConnStatus::Error => Color::rgba(0.70, 0.15, 0.15, 0.50),
    })
    .corner_radius(14.0)
    .animation(Animation::ease_out(220))
    .on_tap(move || {
        let cur_st = state.conn_status.get();
        if cur_st == ConnStatus::Error || cur_st == ConnStatus::Offline {
            state.show_network_modal.set(true);
        } else {
            let cur = expanded.get();
            if !cur {
                expand_island(setter_exp, setter_op);
                schedule_collapse(setter_exp, setter_op);
            } else {
                collapse_island(setter_exp, setter_op);
            }
        }
    });

    apply_glass_blur(content)
}
