use super::*;

pub(super) fn username_step_view(
    username: Signal<String>,
    password: Signal<String>,
    remember: Signal<bool>,
    username_focused: Signal<bool>,
    state: AppState,
    on_submit: impl Fn() + Clone + 'static,
) -> impl Piece {
    let sticker = row((
        spacer().grow(Dimension::Both),
        crate::pages::login::lottie_view::duck_login_animation(116.0),
        spacer().grow(Dimension::Both),
    ));

    let on_sub = on_submit.clone();
    let input_field = text_field(username)
        .input_purpose(InputPurpose::Username)
        .submit_label(SubmitLabel::Next)
        .placeholder(res::str::auth_username_placeholder())
        .focused(username_focused)
        .log_pos("Auth:UsernameField")
        .on_submit(on_sub);

    #[cfg(feature = "uikit")]
    let input_field = native_auth_input(input_field, move |node| {
        let _ = day_uikit::with_native(node, move |view, _, _| {
            crate::ios::autofill::ios::username(view, username);
        });
    });
    #[cfg(not(feature = "uikit"))]
    let input_field = input_field.grow(Dimension::Width);

    let input_card = row((
        vector(res::vectors::tab_login)
            .tint({
                let tint = animated_focus_tint(username_focused, state.accent_color);
                move || tint.get()
            })
            .frame(20.0, 20.0),
        spacer().width(12.0),
        input_field,
    ))
    .align(VAlign::Center)
    .padding(Insets {
        top: 12.0,
        leading: 16.0,
        bottom: 12.0,
        trailing: 16.0,
    })
    .background(move || {
        if day::dark_mode() {
            if username_focused.get() {
                Color::rgba(1.0, 1.0, 1.0, 0.12)
            } else {
                Color::rgba(1.0, 1.0, 1.0, 0.07)
            }
        } else {
            if username_focused.get() {
                Color::rgba(0.0, 0.0, 0.0, 0.06)
            } else {
                Color::rgba(0.0, 0.0, 0.0, 0.035)
            }
        }
    })
    .glass_radius(16.0);

    // The user explicitly requested keeping two steps with a capture input.
    // Mount only while this step is active so it never competes with the real
    // password field on the destination page. It cannot receive touches.
    #[cfg(feature = "uikit")]
    let credential_capture = {
        when(
            move || get_auth_state().path.get().last() == Some(&AuthRoute::Login),
            move || {
                native_auth_input(
                    text_field(password)
                        .secure(true)
                        .input_purpose(InputPurpose::Password)
                        .submit_label(SubmitLabel::Go),
                    move |node| {
                        let _ = day_uikit::with_native(node, move |view, _, _| {
                            crate::ios::autofill::ios::capture_password(view, password);
                        });
                    },
                )
                .height(1.0)
                // A zero-radius surface supplies a native clipping wrapper around
                // the 1pt layout frame, as well as the field's own native clipping.
                .corner_radius(0.0)
                .any()
            },
        )
    };
    #[cfg(not(feature = "uikit"))]
    let credential_capture = {
        let _ = password;
        spacer().height(0.0)
    };

    let content = column((
        input_card
            .padding(Insets {
                top: 0.0,
                leading: 20.0,
                bottom: 12.0,
                trailing: 20.0,
            })
            .grow(Dimension::Width),
        credential_capture,
        // Minimalist native session remember toggle
        row((
            label(res::str::auth_remember())
                .font(Font::Subheadline)
                .secondary()
                .grow(Dimension::Both),
            toggle(remember),
        ))
        .align(VAlign::Center)
        .padding(Insets {
            top: 0.0,
            leading: 24.0,
            bottom: 8.0,
            trailing: 24.0,
        })
        .grow(Dimension::Width),
    ));

    let action = auth_keyboard_dock(auth_action_button(
        res::str::auth_continue(),
        move || !username.get().trim().is_empty() && !state.loading.get(),
        on_submit,
    ));

    let piece = column((
        spacer().height(20.0),
        sticker,
        label(res::str::auth_step_username())
            .font(Font::Headline)
            .secondary()
            .padding(Insets {
                top: 16.0,
                leading: 20.0,
                bottom: 16.0,
                trailing: 20.0,
            }),
        content,
        spacer().grow(Dimension::Both),
        action,
    ))
    .spacing(0.0)
    .grow(Dimension::Both)
    .background(move || {
        if day::dark_mode() {
            Color::hex(0x000000)
        } else {
            Color::hex(0xFFFFFF)
        }
    });

    #[cfg(feature = "uikit")]
    let piece = {
        use day_uikit::UiKitExt;
        piece.uikit(move |view, _class, _mtm| {
            crate::ios::ios_nav::ios_impl::bind_auth_page_title(view, || {
                res::str::auth_username_title().format()
            });
            view.setBackgroundColor(Some(&objc2_ui_kit::UIColor::systemBackgroundColor()));
            if let Some(w) = view.window() {
                w.setBackgroundColor(Some(&objc2_ui_kit::UIColor::systemBackgroundColor()));
            }
        })
    };

    piece
}

pub(super) fn password_step_view(
    username: Signal<String>,
    password: Signal<String>,
    password_focused: Signal<bool>,
    state: AppState,
    on_submit: impl Fn() + Clone + 'static,
) -> impl Piece {
    let sticker = row((
        spacer().grow(Dimension::Both),
        crate::pages::login::lottie_view::duck_password_animation(116.0),
        spacer().grow(Dimension::Both),
    ));

    let show_password = Signal::new(false);

    let eye_progress = animated_visibility(show_password);
    let eye_visible = eye_progress.clone();
    let toggle_eye = zstack((
        vector(res::vectors::eye)
            .tint(Color::hex(0x8E8E93))
            .frame(20.0, 20.0)
            .opacity(move || 1.0 - eye_visible.get()),
        vector(res::vectors::eye_off)
            .tint(Color::hex(0x8E8E93))
            .frame(20.0, 20.0)
            .opacity(move || eye_progress.get()),
    ))
    .frame(36.0, 36.0)
    .on_tap({
        let show_password = show_password;
        move || {
            crate::app::preferences::haptics::tick();
            show_password.set(!show_password.get());
        }
    });

    let pw_field = text_field(password)
        .input_purpose(InputPurpose::Password)
        .submit_label(SubmitLabel::Go)
        .secure(move || !show_password.get())
        .placeholder(res::str::auth_password_placeholder())
        .focused(password_focused)
        .id(day::id!(password_field))
        .log_pos("Auth:PasswordField")
        .on_submit(on_submit.clone());

    #[cfg(feature = "uikit")]
    let pw_field = native_auth_input(pw_field, move |node| {
        let _ = day_uikit::with_native(node, move |view, _, _| {
            crate::ios::autofill::ios::password(view, password);
        });
    });

    #[cfg(not(feature = "uikit"))]
    let pw_field = pw_field.grow(Dimension::Width);

    let input_card = row((
        vector(res::vectors::lock)
            .tint({
                let tint = animated_focus_tint(password_focused, state.accent_color);
                move || tint.get()
            })
            .frame(20.0, 20.0),
        spacer().width(12.0),
        pw_field,
        toggle_eye,
    ))
    .align(VAlign::Center)
    .padding(Insets {
        top: 6.0,
        leading: 16.0,
        bottom: 6.0,
        trailing: 10.0,
    })
    .background(move || {
        if day::dark_mode() {
            if password_focused.get() || !password.get().is_empty() {
                Color::rgba(1.0, 1.0, 1.0, 0.12)
            } else {
                Color::rgba(1.0, 1.0, 1.0, 0.07)
            }
        } else {
            if password_focused.get() || !password.get().is_empty() {
                Color::rgba(0.0, 0.0, 0.0, 0.06)
            } else {
                Color::rgba(0.0, 0.0, 0.0, 0.035)
            }
        }
    })
    .glass_radius(16.0);

    let content = column((input_card
        .padding(Insets {
            top: 0.0,
            leading: 20.0,
            bottom: 8.0,
            trailing: 20.0,
        })
        .grow(Dimension::Width),));

    let btn_title = move || {
        if state.loading.get() {
            res::str::auth_signing_in().format()
        } else {
            res::str::auth_sign_in().format()
        }
    };

    let action = auth_keyboard_dock(auth_action_button(
        btn_title,
        move || !password.get().trim().is_empty() && !state.loading.get(),
        on_submit,
    ));

    let piece = column((
        spacer().height(20.0),
        sticker,
        label(res::str::auth_step_password())
            .font(Font::Headline)
            .secondary()
            .padding(Insets {
                top: 16.0,
                leading: 20.0,
                bottom: 4.0,
                trailing: 20.0,
            }),
        label(move || username.get())
            .font(Font::Subheadline)
            .bold()
            .padding(Insets {
                top: 0.0,
                leading: 20.0,
                bottom: 12.0,
                trailing: 20.0,
            }),
        content,
        spacer().grow(Dimension::Both),
        action,
    ))
    .spacing(0.0)
    .grow(Dimension::Both)
    .background(move || {
        if day::dark_mode() {
            Color::hex(0x000000)
        } else {
            Color::hex(0xFFFFFF)
        }
    });

    #[cfg(feature = "uikit")]
    let piece = {
        use day_uikit::UiKitExt;
        piece.uikit(move |view, _class, _mtm| {
            crate::ios::ios_nav::ios_impl::bind_auth_page_title(view, || {
                res::str::auth_password_title().format()
            });
            view.setBackgroundColor(Some(&objc2_ui_kit::UIColor::systemBackgroundColor()));
            if let Some(w) = view.window() {
                w.setBackgroundColor(Some(&objc2_ui_kit::UIColor::systemBackgroundColor()));
            }
        })
    };

    piece
}
