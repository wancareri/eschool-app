use crate::app::AppState;
use day::prelude::*;

pub fn render(state: AppState, size: f64) -> impl Piece {
    #[cfg(target_os = "ios")]
    {
        use day_uikit::UiKitExt;
        let hex = state.accent_color.get();
        let r = ((hex >> 16) & 0xFF) as f64 / 255.0;
        let g = ((hex >> 8) & 0xFF) as f64 / 255.0;
        let b = (hex & 0xFF) as f64 / 255.0;
        spinner()
            .frame(size, size)
            .uikit(move |view, _class, _mtm| {
                if let Some(ai) = view.downcast_ref::<objc2_ui_kit::UIActivityIndicatorView>() {
                    let color = objc2_ui_kit::UIColor::colorWithRed_green_blue_alpha(r, g, b, 1.0);
                    unsafe { ai.setColor(Some(&color)) };
                }
            })
            .any()
    }
    #[cfg(not(target_os = "ios"))]
    {
        let _ = state;
        spinner()
            .frame(size, size)
            .any()
    }
}
