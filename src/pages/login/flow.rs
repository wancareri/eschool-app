use super::*;

#[derive(Clone, Copy)]
pub(super) struct AuthState {
    pub(super) path: Signal<Vec<AuthRoute>>,
    pub(super) username: Signal<String>,
    pub(super) password: Signal<String>,
    pub(super) username_focused: Signal<bool>,
    pub(super) password_focused: Signal<bool>,
    pub(super) remember: Signal<bool>,
    pub(super) error_local: Signal<String>,
    pub(super) settings_revision: Signal<u64>,
}

pub(super) fn get_auth_state() -> AuthState {
    thread_local! {
        static STATE: std::cell::RefCell<Option<AuthState>> = const { std::cell::RefCell::new(None) };
    }
    STATE.with(|cell| {
        let mut b = cell.borrow_mut();
        if let Some(s) = b.as_ref() {
            *s
        } else {
            let s = AuthState {
                path: Signal::global(Vec::<AuthRoute>::new()),
                username: Signal::global(String::new()),
                password: Signal::global(String::new()),
                username_focused: Signal::global(false),
                password_focused: Signal::global(false),
                remember: Signal::global(true),
                error_local: Signal::global(String::new()),
                settings_revision: Signal::global(0),
            };
            *b = Some(s);
            s
        }
    })
}

/// Clear every auth draft before showing the welcome page again.
pub fn reset_auth_flow() {
    #[cfg(feature = "uikit")]
    crate::ios::autofill::ios::reset();
    let auth = get_auth_state();
    day::reactive::batch(|| {
        auth.username_focused.set(false);
        auth.password_focused.set(false);
        auth.username.set(String::new());
        auth.password.set(String::new());
        auth.error_local.set(String::new());
        auth.remember.set(true);
        auth.path.set(Vec::new());
    });
}

pub(super) fn focus_auth_route(route: AuthRoute) {
    let ticket = crate::app::data::session::ticket();
    day::reactive::on_main(move || {
        let allowed = move || {
            if !ticket.is_current() {
                return false;
            }
            let Some(state) = AppState::get_main() else {
                return false;
            };
            !state.is_authenticated.get_untracked()
                && !state.setup_new_password_active.get_untracked()
                && get_auth_state().path.get_untracked().last() == Some(&route)
        };
        #[cfg(feature = "uikit")]
        crate::ios::autofill::ios::focus_after_navigation(
            match route {
                AuthRoute::Login => crate::ios::autofill::ios::Role::Username,
                AuthRoute::Password => crate::ios::autofill::ios::Role::Password,
            },
            allowed,
        );
        #[cfg(not(feature = "uikit"))]
        if allowed() {
            let auth = get_auth_state();
            match route {
                AuthRoute::Login => auth.username_focused.set(true),
                AuthRoute::Password => auth.password_focused.set(true),
            }
        }
    });
}

pub(super) fn auth_form(state: AppState, _show_logs: Signal<bool>) -> impl Piece {
    let auth = get_auth_state();
    let path = auth.path;
    let username = auth.username;
    let password = auth.password;
    let username_focused = auth.username_focused;
    let password_focused = auth.password_focused;
    let remember = auth.remember;
    let error_local = auth.error_local;
    crate::components::notice::bind_error(state, error_local, error_notice);

    let root = intro_view({
        let path = path;
        let error_local = error_local;
        let state = state;
        move || {
            crate::app::preferences::haptics::tick();
            error_local.set(String::new());
            state.error_msg.set(String::new());
            path.set(vec![AuthRoute::Login]);
            focus_auth_route(AuthRoute::Login);
        }
    });

    let proceed_username = {
        let username = username;
        let error_local = error_local;
        let state = state;
        let path = path;
        move || {
            if state.loading.get() {
                return;
            }
            #[cfg(feature = "uikit")]
            crate::ios::autofill::ios::sync_values();
            let u = username.get_untracked();
            if u.trim().is_empty() {
                error_local.set("Введите логин".into());
            } else {
                error_local.set(String::new());
                state.error_msg.set(String::new());
                crate::app::preferences::haptics::tick();
                path.set(vec![AuthRoute::Login, AuthRoute::Password]);
                focus_auth_route(AuthRoute::Password);
            }
        }
    };

    let do_login = {
        let username = username;
        let password = password;
        let remember = remember;
        let error_local = error_local;
        let state = state;
        move || {
            if state.loading.get() {
                return;
            }
            #[cfg(feature = "uikit")]
            crate::ios::autofill::ios::sync_values();
            let u = username.get_untracked();
            let p = password.get_untracked();
            let r = remember.get_untracked();
            if p.trim().is_empty() {
                error_local.set("Введите пароль".into());
                return;
            }
            error_local.set(String::new());
            state.error_msg.set(String::new());
            crate::app::preferences::haptics::pop();
            crate::pages::login::auth::login(state, &u, &p, r);
        }
    };

    let proceed_username_sub = proceed_username.clone();
    let do_login_sub = do_login.clone();

    let stack = crate::components::header::page("ESchool", path, root)
        .destination(move |route: &AuthRoute| match route {
            AuthRoute::Login => username_step_view(
                username,
                password,
                remember,
                username_focused,
                state,
                proceed_username_sub.clone(),
            )
            .any(),
            AuthRoute::Password => password_step_view(
                username,
                password,
                password_focused,
                state,
                do_login_sub.clone(),
            )
            .any(),
        })
        .id(day::id!(auth_stack))
        .grow(Dimension::Both)
        .background(move || {
            if day::dark_mode() {
                Color::hex(0x000000)
            } else {
                Color::hex(0xFFFFFF)
            }
        });

    day::reactive::watch(
        move || path.get(),
        move |p, old| {
            if p.is_empty() {
                username_focused.set(false);
                password_focused.set(false);
                crate::ios::keyboard::dismiss();
            } else if old.is_some_and(|old| old.len() > p.len()) {
                // Day reports only completed native pops. A cancelled swipe
                // never changes the path and must not dismiss/refocus fields.
                if p.last() == Some(&AuthRoute::Login) {
                    password_focused.set(false);
                    focus_auth_route(AuthRoute::Login);
                }
            }
        },
    );

    stack
}
