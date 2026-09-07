use crate::core::network::{Auth, ApiClient};
use crate::native;
use crate::res;
use day::prelude::*;

/// Login page with native OAuth2 flow
pub(crate) fn login_page() -> impl Piece {
    let auth = Auth::new();
    let login_url = auth.login_url();
    
    column((
        native::header::render(
            res::str::app_title().format(),
            "Вход в аккаунт",
        ),
        spacer(),
        // TODO: WebView for OAuth2 flow
        // For now, show a button that opens the login URL
        native::button::render(
            "Войти",
            "person.circle",
        ),
        spacer(),
    ))
    .spacing(16.0)
    .padding(24.0)
    .grow()
}
