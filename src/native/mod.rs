pub mod common;

#[cfg(any(feature = "appkit", feature = "uikit"))]
pub mod ios;

#[cfg(feature = "mdc")]
pub mod android;

// Re-export the active platform module
#[cfg(any(feature = "appkit", feature = "uikit"))]
pub use ios as platform;

#[cfg(feature = "mdc")]
pub use android as platform;

// Fallback for other platforms (mock, gtk, etc.)
#[cfg(not(any(feature = "appkit", feature = "uikit", feature = "mdc")))]
pub use common as platform;
