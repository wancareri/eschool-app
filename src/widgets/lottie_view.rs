//! Authentication stickers rendered by Day’s native Lottie piece on iOS/Android.
use day::prelude::*;
use day_piece_lottie::lottie;

pub fn duck_intro_animation(size: f64) -> impl Piece {
    lottie("animations/duck_intro").frame(size, size)
}

pub fn duck_login_animation(size: f64) -> impl Piece {
    lottie("animations/duck_login").frame(size, size)
}

pub fn duck_password_animation(size: f64) -> impl Piece {
    lottie("animations/duck_password").frame(size, size)
}
