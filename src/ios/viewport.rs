
pub(crate) fn get_screen_width() -> f64 {
    #[cfg(feature = "uikit")]
    {
        use objc2::MainThreadMarker;
        use objc2_ui_kit::UIScreen;
        if let Some(mtm) = MainThreadMarker::new() {
            let screen = UIScreen::mainScreen(mtm);
            let bounds = screen.bounds();
            if bounds.size.width > 50.0 {
                let width = bounds.size.width as f64 - window_side_insets(mtm);
                if width > 50.0 {
                    return width;
                }
            }
        }
    }
    390.0
}

#[cfg(feature = "uikit")]
#[allow(deprecated)] // keyWindow — single-scene app; day's window is the key one
fn window_side_insets(mtm: objc2::MainThreadMarker) -> f64 {
    use objc2_ui_kit::UIApplication;
    let app = UIApplication::sharedApplication(mtm);
    match app.keyWindow() {
        Some(window) => {
            let insets = window.safeAreaInsets();
            (insets.left + insets.right) as f64
        }
        None => 0.0,
    }
}

#[allow(deprecated)] // same UIScreen::mainScreen pattern as get_screen_width above
pub(crate) fn get_screen_height() -> f64 {
    #[cfg(feature = "uikit")]
    {
        use objc2::MainThreadMarker;
        use objc2_ui_kit::UIScreen;
        if let Some(mtm) = MainThreadMarker::new() {
            let screen = UIScreen::mainScreen(mtm);
            let bounds = screen.bounds();
            if bounds.size.height > 50.0 {
                return bounds.size.height as f64;
            }
        }
    }
    844.0
}
