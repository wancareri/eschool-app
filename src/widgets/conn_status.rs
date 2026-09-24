/// Dynamic Island connection status indicator — frosted glass pill with auto-collapse.
use std::sync::atomic::{AtomicU64, Ordering};
use crate::app::{AppState, ConnStatus};
use crate::res;
use crate::shared::colors;
use day::prelude::*;

static COLLAPSE_GEN: AtomicU64 = AtomicU64::new(0);

fn schedule_collapse(setter: Setter<bool>) {
    let gen_id = COLLAPSE_GEN.fetch_add(1, Ordering::SeqCst) + 1;
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(3000));
        if COLLAPSE_GEN.load(Ordering::SeqCst) == gen_id {
            day::reactive::on_main(move || {
                with_animation(Animation::ease_out(300), move || {
                    setter.set(false);
                });
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
        effect_view.setClipsToBounds(true);

        let effect_layer: Retained<CALayer> = effect_view.layer();
        effect_layer.setCornerRadius(11.0);
        effect_layer.setMasksToBounds(true);

        let view_layer: Retained<CALayer> = view.layer();
        view_layer.setCornerRadius(11.0);
        view_layer.setMasksToBounds(true);

        view.insertSubview_atIndex(&effect_view, 0);
        view.setClipsToBounds(true);
    })
}

#[cfg(not(target_os = "ios"))]
fn apply_glass_blur(piece: impl Decorate) -> impl Piece {
    piece
}

fn render_pill(status: ConnStatus, expanded: Signal<bool>) -> impl Piece {
    let setter = expanded.setter();
    let (icon_kind, text, text_color, bg_tint) = match status {
        ConnStatus::Connecting => (
            0,
            "Обновление…",
            Color::rgba(0.45, 0.52, 0.62, 1.0),
            Color::rgba(0.2, 0.25, 0.35, 0.08),
        ),
        ConnStatus::Connected => (
            1,
            "Подключено",
            colors::SUCCESS,
            Color::rgba(0.06, 0.72, 0.50, 0.08),
        ),
        ConnStatus::Idle => (
            2,
            "В сети",
            colors::SUCCESS,
            Color::rgba(0.06, 0.72, 0.50, 0.06),
        ),
        ConnStatus::Offline => (
            3,
            "Оффлайн",
            colors::WARNING,
            Color::rgba(0.96, 0.62, 0.04, 0.08),
        ),
        ConnStatus::Error => (
            4,
            "Ошибка сети",
            colors::ERROR,
            Color::rgba(0.94, 0.27, 0.27, 0.08),
        ),
    };

    let icon_piece = match icon_kind {
        0 => spinner().frame(12.0, 12.0).any(),
        1 => vector(res::vectors::status_ok).frame(12.0, 12.0).tint(colors::SUCCESS).any(),
        2 => zstack((
            circle().fill(colors::SUCCESS).frame(7.0, 7.0),
        )).align(Alignment::Center).frame(12.0, 12.0).any(),
        3 => vector(res::vectors::status_offline).frame(12.0, 12.0).tint(colors::WARNING).any(),
        _ => vector(res::vectors::status_error).frame(12.0, 12.0).tint(colors::ERROR).any(),
    };

    let content = row((
        icon_piece,
        when(
            move || expanded.get(),
            move || {
                label(text)
                    .font(Font::Caption2)
                    .color(text_color)
                    .padding(Insets { top: 0.0, leading: 4.0, bottom: 0.0, trailing: 2.0 })
            },
        ),
    ))
    .align(VAlign::Center)
    .padding(Insets { top: 4.5, leading: 6.0, bottom: 4.5, trailing: 6.0 })
    .background(bg_tint)
    .corner_radius(11.0)
    .animation(Animation::ease_out(300))
    .on_tap(move || {
        let cur = expanded.get();
        if !cur {
            with_animation(Animation::ease_out(250), move || {
                setter.set(true);
            });
            schedule_collapse(setter);
        } else {
            with_animation(Animation::ease_out(250), move || {
                setter.set(false);
            });
        }
    });

    apply_glass_blur(content)
}

pub fn render() -> impl Piece {
    let state = AppState::ambient();
    let expanded = Signal::new(true);
    let setter = expanded.setter();

    // Initial collapse after 3 seconds
    schedule_collapse(setter);

    // Watch for connection status updates
    watch(
        move || state.conn_status.get(),
        move |new_st, old_st| {
            if old_st != Some(new_st) {
                with_animation(Animation::ease_out(250), move || {
                    expanded.set(true);
                });
                schedule_collapse(setter);
            }
        },
    );

    when(
        move || state.conn_status.get() == ConnStatus::Connecting,
        move || render_pill(ConnStatus::Connecting, expanded),
    )
    .otherwise(move || {
        when(
            move || state.conn_status.get() == ConnStatus::Connected,
            move || render_pill(ConnStatus::Connected, expanded),
        )
        .otherwise(move || {
            when(
                move || state.conn_status.get() == ConnStatus::Offline,
                move || render_pill(ConnStatus::Offline, expanded),
            )
            .otherwise(move || {
                when(
                    move || state.conn_status.get() == ConnStatus::Error,
                    move || render_pill(ConnStatus::Error, expanded),
                )
                .otherwise(move || {
                    render_pill(ConnStatus::Idle, expanded)
                })
            })
        })
    })
}
