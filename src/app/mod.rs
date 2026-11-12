pub mod data;
pub mod format;
pub mod preferences;
pub mod i18n;
pub mod log_bridge;
pub mod state;
pub mod window;

pub use state::{AppState, ConnStatus, FinalMarks};
pub use crate::pages::Section;
pub use window::{window, root};
