//! Widget data provider — stub for now.

use crate::app::AppState;

/// Write today's schedule to shared UserDefaults (App Group) for the widget.
pub fn update_widget_data(_state: AppState) {
    // Widget data writing requires complex objc2 setup.
    // Will be implemented when widget extension is added in Xcode.
}
