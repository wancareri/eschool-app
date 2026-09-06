mod common;

#[cfg(any(feature = "appkit", feature = "uikit"))]
mod ios;

#[cfg(feature = "mdc")]
mod android;

#[cfg(feature = "appkit")]
mod macos;

#[cfg(feature = "xaml")]
mod windows;

// Re-export active platform's components directly
#[cfg(feature = "uikit")]
pub use ios::*;

#[cfg(feature = "appkit")]
pub use macos::*;

#[cfg(feature = "mdc")]
pub use android::*;

#[cfg(feature = "xaml")]
pub use windows::*;

// Fallback
#[cfg(not(any(feature = "appkit", feature = "uikit", feature = "mdc", feature = "xaml")))]
pub use common::*;
