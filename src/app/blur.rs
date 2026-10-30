// SPDX-License-Identifier: MPL-2.0
//! Helper for glass blur effect — natively powered by Day's `.glass()`.

use day::prelude::*;

/// Extension trait to allow method-like usage of `apply_glass_blur`.
pub trait ApplyGlassBlur: Decorate + Sized {
    fn apply_glass_blur(self) -> Decorated<Self> {
        self.glass()
    }
}

impl<P: Decorate + Sized> ApplyGlassBlur for P {}
