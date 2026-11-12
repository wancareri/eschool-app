//! Day drag adapter with iOS horizontal gesture arbitration.

use day::prelude::*;

/// Attach a Day drag and apply native horizontal-axis arbitration on UIKit.
pub(crate) fn horizontal_pager(
    content: impl Piece,
    handler: impl Fn(Drag) + 'static,
) -> impl Piece {
    let piece = content.on_drag(handler);
    #[cfg(feature = "uikit")]
    let piece = piece.tweak(|node| {
        day_uikit::with_native(node, |view, _, mtm| {
            crate::ios::gestures::horizontal_pager(view, mtm);
        });
    });
    piece
}
