use super::auth_error_code;

#[cfg(feature = "mock")]
#[test]
fn auth_action_keeps_a_full_capsule_when_title_and_enabled_state_change() {
    use day::prelude::*;
    day_core::uninstall_tree();
    let (mock, probe) = day_mock::MockToolkit::new();
    let title = Signal::global("Continue");
    let enabled = Signal::global(true);
    day_core::launch_with(mock, day_spec::WindowOptions::default(), move || {
        super::auth_action_button(move || title.get(), move || enabled.get(), || {}).any()
    });
    for (text, active) in [("Continue", true), ("Signing in…", false), ("Войти", true)] {
        title.set(text);
        enabled.set(active);
        day::reactive::flush_sync();
        let button = probe.find_by_kind(day_spec::kinds::BUTTON)[0].0;
        let state = probe.state.borrow();
        assert_eq!(state.widgets[&button.0].text, text);
        assert_eq!(state.widgets[&button.0].enabled, active);
        assert!(
            state.widgets.values().any(|widget| {
                widget.corner_radius == 26.0 && (widget.frame.size.height - 52.0).abs() < 0.5
            }),
            "auth action must retain a 26pt radius around its 52pt height"
        );
    }
    day_core::uninstall_tree();
}

#[cfg(feature = "mock")]
#[test]
fn password_eye_crossfades_in_place_and_reverses_on_a_second_tap() {
    use day::prelude::*;
    use std::{cell::RefCell, collections::VecDeque, rc::Rc};
    day_core::uninstall_tree();
    let (mock, probe) = day_mock::MockToolkit::new();
    let pending: Rc<RefCell<VecDeque<day_spec::FrameCallback>>> = Rc::default();
    let queue = pending.clone();
    day_core::launch_with(mock, day_spec::WindowOptions::default(), move || {
        // Install before constructing the page: initial field focus can
        // already arm the shared display clock for its leading-icon tint.
        day::frame::install_frame_requester(move |_, callback| {
            queue.borrow_mut().push_back(callback);
            Box::new(|| {})
        });
        super::password_step_view(
            Signal::new("account".into()),
            Signal::new("fixture".into()),
            Signal::new(true),
            crate::app::AppState::create(),
            || {},
        )
        .any()
    });
    let field = probe.find_by_kind(day_spec::kinds::TEXT_FIELD)[0].0;
    let images: Vec<_> = probe
        .find_by_kind(day_spec::kinds::IMAGE)
        .iter()
        .map(|(h, _)| *h)
        .collect();
    let (visible, hidden, tap_node) = {
        let state = probe.state.borrow();
        let visible = state
            .widgets
            .iter()
            .find(|(_, w)| w.opacity == Some(1.0))
            .map(|(h, _)| *h)
            .expect("visible eye");
        let hidden = state
            .widgets
            .iter()
            .find(|(_, w)| w.opacity == Some(0.0))
            .map(|(h, _)| *h)
            .expect("hidden eye remains mounted");
        let tap = state
            .log
            .iter()
            .find_map(|line| {
                line.strip_prefix("enable_gesture #")
                    .and_then(|s| s.strip_suffix(" Tap"))
                    .and_then(|s| s.parse::<u64>().ok())
            })
            .expect("eye tap target");
        (visible, hidden, state.widgets[&tap].node)
    };
    let frame = |time| {
        let callback = pending
            .borrow_mut()
            .pop_front()
            .expect("requested animation frame");
        callback(day_spec::FrameStamp::new(time));
        day::reactive::flush_sync();
    };
    let tap = || {
        probe.emit(
            day_spec::NodeId(tap_node),
            day_spec::Event::Tap(Point::ZERO),
        );
        day::reactive::flush_sync();
    };
    let opacity = |handle| probe.state.borrow().widgets[&handle].opacity.unwrap();
    tap();
    assert!(
        !probe.widget(field).input_traits.unwrap().secure,
        "masking changes immediately while the artwork animates"
    );
    assert_eq!(opacity(visible), 1.0, "eye must not snap");
    frame(1.0);
    frame(1.1);
    let middle = opacity(visible);
    assert!(middle > 0.0 && middle < 1.0);
    assert!((middle + opacity(hidden) - 1.0).abs() < 0.001);
    tap();
    assert!(probe.widget(field).input_traits.unwrap().secure);
    assert_eq!(
        opacity(visible),
        middle,
        "rapid toggle reverses from displayed opacity"
    );
    frame(1.11); // queued request shared with the focus tint
    frame(1.12);
    frame(1.4);
    assert_eq!(opacity(visible), 1.0);
    assert_eq!(opacity(hidden), 0.0);
    assert!(probe.widget(field).focused);
    assert_eq!(probe.widget(field).text, "fixture");
    assert!(
        images
            .iter()
            .all(|h| probe.state.borrow().widgets.contains_key(&h.0)),
        "neither artwork nor field may be recreated"
    );
    assert!(pending.borrow().is_empty());
    day_core::uninstall_tree();
}

#[cfg(feature = "mock")]
#[test]
fn disposing_credential_animations_cancels_pending_frames() {
    use day::prelude::*;
    use std::{cell::RefCell, collections::VecDeque, rc::Rc};
    day_core::uninstall_tree();
    let focused = Signal::global(false);
    let visible = Signal::global(false);
    let saved = Rc::new(RefCell::new(None));
    let result = saved.clone();
    let (mock, _) = day_mock::MockToolkit::new();
    day_core::launch_with(mock, day_spec::WindowOptions::default(), move || {
        let scope = Scope::child();
        scope.enter(|| {
            let tint = super::animated_focus_tint(focused, Signal::new(0x007AFF));
            let eye = super::animated_visibility(visible);
            *result.borrow_mut() = Some((scope, tint, eye));
        });
        spacer().any()
    });
    let pending: Rc<RefCell<VecDeque<day_spec::FrameCallback>>> = Rc::default();
    let queue = pending.clone();
    day::frame::install_frame_requester(move |_, callback| {
        queue.borrow_mut().push_back(callback);
        Box::new(|| {})
    });
    focused.set(true);
    visible.set(true);
    day::reactive::flush_sync();
    let (scope, tint, eye) = saved.borrow_mut().take().unwrap();
    assert!(tint.is_animating() && eye.is_animating());
    assert!(!pending.borrow().is_empty());

    scope.dispose();
    assert!(
        !tint.is_animating() && !eye.is_animating(),
        "teardown must cancel both animations before freeing their signals"
    );
    let stale_frames = std::mem::take(&mut *pending.borrow_mut());
    for callback in stale_frames {
        callback(day_spec::FrameStamp::new(1.0));
    }
    day::reactive::flush_sync();
    assert!(
        pending.borrow().is_empty(),
        "disposed pages must not request more frames"
    );
    day_core::uninstall_tree();
}

#[cfg(feature = "mock")]
#[test]
fn focus_tint_interpolates_and_reverses_without_a_color_jump() {
    use day::prelude::*;
    use std::{cell::RefCell, collections::VecDeque, rc::Rc};
    day_core::uninstall_tree();
    let focus = Signal::global(false);
    let accent = Signal::global(0x007AFF);
    let saved = Rc::new(RefCell::new(None));
    let result = saved.clone();
    let (mock, _) = day_mock::MockToolkit::new();
    day_core::launch_with(mock, day_spec::WindowOptions::default(), move || {
        let tint = super::animated_focus_tint(focus, accent);
        *result.borrow_mut() = Some(tint.clone());
        vector(crate::res::vectors::tab_login)
            .tint(move || tint.get())
            .any()
    });
    let pending: Rc<RefCell<VecDeque<day_spec::FrameCallback>>> = Rc::default();
    let queue = pending.clone();
    day::frame::install_frame_requester(move |_, callback| {
        queue.borrow_mut().push_back(callback);
        Box::new(|| {})
    });
    let frame = |time| {
        let callback = pending
            .borrow_mut()
            .pop_front()
            .expect("animation requested a frame");
        callback(day_spec::FrameStamp::new(time));
        day::reactive::flush_sync();
    };
    let tint = saved.borrow().clone().unwrap();
    let gray = Color::hex(0x8E8E93);
    let blue = Color::hex(0x007AFF);
    assert_eq!(tint.get(), gray);
    focus.set(true);
    day::reactive::flush_sync();
    assert_eq!(tint.get(), gray, "focus must not snap straight to accent");
    frame(1.0);
    frame(1.1);
    let middle = tint.get();
    assert!(
        middle != gray && middle != blue,
        "expected an intermediate color: {middle:?}"
    );
    focus.set(false);
    day::reactive::flush_sync();
    assert_eq!(
        tint.get(),
        middle,
        "blur must reverse from the displayed color"
    );
    // Deliver the stale cancelled request, then the new flight's start/end.
    frame(1.11);
    frame(1.12);
    frame(1.4);
    assert_eq!(tint.get(), gray);
    focus.set(true);
    day::reactive::flush_sync();
    frame(2.0);
    frame(2.3);
    assert_eq!(tint.get(), blue);
    accent.set(0xFF9500);
    day::reactive::flush_sync();
    frame(3.0);
    frame(3.3);
    assert_eq!(tint.get(), Color::hex(0xFF9500));
    assert!(
        pending.borrow().is_empty(),
        "idle icons must stop requesting frames"
    );
    day_core::uninstall_tree();
}

#[cfg(feature = "mock")]
#[test]
fn welcome_action_uses_the_same_bottom_inset_as_credential_steps() {
    use day::prelude::*;
    for height in [600.0, 720.0, 844.0] {
        day_core::uninstall_tree();
        let (mock, probe) = day_mock::MockToolkit::new();
        day_core::launch_with(
            mock,
            day_spec::WindowOptions {
                size: Size::new(390.0, height),
                ..Default::default()
            },
            || super::intro_view(|| {}).any(),
        );
        let state = probe.state.borrow();
        let mut handle = probe.find_by_kind(day_spec::kinds::BUTTON)[0].0.0;
        let button = state.widgets[&handle].frame;
        let mut bottom = button.origin.y + button.size.height;
        while let Some((parent, widget)) = state
            .widgets
            .iter()
            .find(|(_, widget)| widget.children.contains(&handle))
        {
            bottom += widget.frame.origin.y;
            handle = *parent;
        }
        assert!(
            (bottom - (height - 8.0)).abs() < 0.5,
            "welcome action bottom {bottom}, expected {}",
            height - 8.0
        );
        drop(state);
        day_core::uninstall_tree();
    }
}

#[cfg(feature = "mock")]
#[test]
fn auth_steps_keep_inputs_below_compact_sticker_and_action_at_bottom() {
    use day::prelude::*;
    for password_step in [false, true] {
        day_core::uninstall_tree();
        let (mock, probe) = day_mock::MockToolkit::new();
        day_core::launch_with(
            mock,
            day_spec::WindowOptions {
                size: Size::new(390.0, 600.0),
                ..Default::default()
            },
            move || {
                let state = crate::app::AppState::create();
                if password_step {
                    super::password_step_view(
                        Signal::new("account".into()),
                        Signal::new(String::new()),
                        Signal::new(false),
                        state,
                        || {},
                    )
                    .any()
                } else {
                    super::username_step_view(
                        Signal::new(String::new()),
                        Signal::new(String::new()),
                        Signal::new(true),
                        Signal::new(false),
                        state,
                        || {},
                    )
                    .any()
                }
            },
        );
        let state = probe.state.borrow();
        let bounds = |handle: u64| {
            let mut frame = state.widgets[&handle].frame;
            let mut child = handle;
            while let Some((parent, widget)) = state
                .widgets
                .iter()
                .find(|(_, widget)| widget.children.contains(&child))
            {
                frame.origin.x += widget.frame.origin.x;
                frame.origin.y += widget.frame.origin.y;
                child = *parent;
            }
            frame
        };
        let sticker = probe.find_by_kind("day.piece.lottie")[0].0;
        let field = probe.find_by_kind(day_spec::kinds::TEXT_FIELD)[0].0;
        let button = probe.find_by_kind(day_spec::kinds::BUTTON)[0].0;
        let sticker = bounds(sticker.0);
        let field = bounds(field.0);
        let button = bounds(button.0);
        assert!(
            (110.0..=120.0).contains(&sticker.size.height),
            "sticker: {sticker:?}"
        );
        assert!(
            (16.0..=24.0).contains(&sticker.origin.y),
            "top spacing: {sticker:?}"
        );
        assert!(field.origin.y > sticker.origin.y + sticker.size.height);
        assert!(
            field.origin.y < 240.0,
            "input must follow sticker: {field:?}"
        );
        assert!(
            (button.origin.y + button.size.height - 592.0).abs() < 0.5,
            "action must be docked 8pt above safe-area bottom: {button:?}"
        );
        drop(state);
        day_core::uninstall_tree();
    }
}

#[cfg(feature = "mock")]
#[test]
fn auth_step_buttons_follow_trimmed_input_and_loading() {
    use day::prelude::*;
    for password_step in [false, true] {
        day_core::uninstall_tree();
        let value = Signal::global(String::new());
        let loading = Signal::global(false);
        let (mock, probe) = day_mock::MockToolkit::new();
        day_core::launch_with(mock, day_spec::WindowOptions::default(), move || {
            let mut state = crate::app::AppState::create();
            state.loading = loading;
            if password_step {
                super::password_step_view(
                    Signal::new("account".into()),
                    value,
                    Signal::new(false),
                    state,
                    || {},
                )
                .any()
            } else {
                super::username_step_view(
                    value,
                    Signal::new(String::new()),
                    Signal::new(true),
                    Signal::new(false),
                    state,
                    || {},
                )
                .any()
            }
        });
        let buttons = probe.find_by_kind(day_spec::kinds::BUTTON);
        assert_eq!(buttons.len(), 1);
        let handle = buttons[0].0;
        for (text, busy, enabled) in [
            ("", false, false),
            (" \t\n\u{2003}", false, false),
            (" account ", false, true),
            ("account", true, false),
            ("account", false, true),
            ("", false, false),
        ] {
            value.set(text.into());
            loading.set(busy);
            day::reactive::flush_sync();
            let button = probe.widget(handle);
            assert_eq!(
                button.enabled, enabled,
                "password_step={password_step}, busy={busy}, expected enabled={enabled}"
            );
            assert_eq!(value.get(), text, "validation must not rewrite credentials");
            assert!(
                !button.text.is_empty(),
                "disabled native buttons keep their label"
            );
        }
        day_core::uninstall_tree();
    }
}

#[test]
fn reset_discards_credentials_focus_and_navigation() {
    use super::{AuthRoute, get_auth_state, reset_auth_flow};
    let auth = get_auth_state();
    auth.username.set("test-account".into());
    auth.password.set("test-secret".into());
    auth.path.set(vec![AuthRoute::Login, AuthRoute::Password]);
    auth.username_focused.set(true);
    auth.password_focused.set(true);
    auth.error_local.set("previous error".into());
    auth.remember.set(false);
    reset_auth_flow();
    assert!(auth.username.get().is_empty());
    assert!(auth.password.get().is_empty());
    assert!(auth.path.get().is_empty());
    assert!(!auth.username_focused.get());
    assert!(!auth.password_focused.get());
    assert!(auth.error_local.get().is_empty());
    assert!(auth.remember.get());
    reset_auth_flow();
    assert!(auth.path.get().is_empty());
}

#[cfg(feature = "mock")]
#[test]
fn credential_configuration_receives_the_native_field_before_layout_wrappers() {
    use day::prelude::*;
    use std::{cell::RefCell, rc::Rc};
    let handles = Rc::new(RefCell::new(Vec::new()));
    let observed = handles.clone();
    let shown = Signal::global(false);
    let (mock, probe) = day_mock::MockToolkit::new();
    day_core::uninstall_tree();
    day_core::launch_with(mock, day_spec::WindowOptions::default(), move || {
        let input = |purpose: InputPurpose, submit: SubmitLabel, capture: bool| {
            let observed = observed.clone();
            let value = Signal::new(String::from("fixture"));
            let focus = Signal::new(purpose == InputPurpose::Password && !capture);
            let field = text_field(value)
                .focused(focus)
                .input_purpose(purpose)
                .submit_label(submit)
                .secure(move || purpose != InputPurpose::Username && (capture || !shown.get()));
            super::native_auth_input(field, move |node| {
                day_core::with_tree(|tree| {
                    let handle = tree
                        .node_handle_any(node)
                        .expect("credential configuration must receive a native handle");
                    assert_eq!(tree.node_kind(node), Some(day_spec::kinds::TEXT_FIELD));
                    observed
                        .borrow_mut()
                        .push(*handle.downcast::<day_mock::MockHandle>().unwrap());
                });
            })
            .height(if capture { 1.0 } else { 44.0 })
            .any()
        };
        column((
            input(InputPurpose::Username, SubmitLabel::Next, false),
            input(InputPurpose::Password, SubmitLabel::Go, false),
            input(InputPurpose::Password, SubmitLabel::Go, true),
            input(InputPurpose::NewPassword, SubmitLabel::Done, false),
        ))
        .any()
    });
    let handles = handles.borrow().clone();
    assert_eq!(
        handles.len(),
        4,
        "all credential roles and capture must configure"
    );
    for (handle, (purpose, submit_label, secure)) in handles.iter().zip([
        (InputPurpose::Username, SubmitLabel::Next, false),
        (InputPurpose::Password, SubmitLabel::Go, true),
        (InputPurpose::Password, SubmitLabel::Go, true),
        (InputPurpose::NewPassword, SubmitLabel::Done, true),
    ]) {
        let traits = probe.widget(*handle).input_traits.unwrap();
        assert_eq!(traits.purpose, purpose);
        assert_eq!(traits.submit_label, submit_label);
        assert_eq!(traits.secure, secure);
    }
    let password = probe.widget(handles[1]);
    assert!(password.focused);
    shown.set(true);
    day::reactive::flush_sync();
    let visible = probe.widget(handles[1]);
    assert!(visible.focused, "masking must preserve focus");
    assert_eq!(
        visible.node, password.node,
        "masking must preserve field identity"
    );
    assert_eq!(
        visible.text, password.text,
        "masking must preserve the captured value"
    );
    let traits = visible.input_traits.unwrap();
    assert!(!traits.secure);
    assert_eq!(traits.purpose, InputPurpose::Password);
    assert_eq!(traits.submit_label, SubmitLabel::Go);
    assert!(
        probe.widget(handles[2]).input_traits.unwrap().secure,
        "the separate capture field keeps its secure role"
    );
    shown.set(false);
    day::reactive::flush_sync();
    assert!(probe.widget(handles[1]).input_traits.unwrap().secure);
    day_core::uninstall_tree();
}

#[test]
fn transport_errors_are_not_bad_passwords() {
    assert_eq!(
        auth_error_code("password login: error sending request: DNS error"),
        "network"
    );
    assert_eq!(
        auth_error_code("invalid password: request timed out"),
        "timeout"
    );
    assert_eq!(auth_error_code("password endpoint: HTTP 503"), "server");
    assert_eq!(
        auth_error_code("HTTP 403: invalid credentials"),
        "forbidden"
    );
    assert_eq!(auth_error_code("HTTP 401 Unauthorized"), "credentials");
    assert_eq!(auth_error_code("Введите пароль"), "empty_password");
}
