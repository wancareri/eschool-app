mod common;

#[cfg(any(feature = "appkit", feature = "uikit"))]
mod ios;

#[cfg(feature = "mdc")]
mod android;

#[cfg(feature = "appkit")]
mod macos;

#[cfg(feature = "xaml")]
mod windows;

// Re-export active platform's glass functions directly
#[cfg(feature = "uikit")]
pub use ios::glass::*;

#[cfg(feature = "appkit")]
pub use macos::glass::*;

#[cfg(feature = "mdc")]
pub use android::glass::*;

#[cfg(feature = "xaml")]
pub use windows::glass::*;

// Fallback
#[cfg(not(any(feature = "appkit", feature = "uikit", feature = "mdc", feature = "xaml")))]
pub use common::glass::*;
