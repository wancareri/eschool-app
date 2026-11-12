//! Reject replies belonging to a previous account or selected child.
use std::sync::atomic::{AtomicU64, Ordering};

static GENERATION: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy)]
pub struct Ticket(u64);

pub fn ticket() -> Ticket {
    Ticket(GENERATION.load(Ordering::Acquire))
}

/// Call on the UI thread before changing account/child or clearing its data.
pub fn invalidate() {
    GENERATION.fetch_add(1, Ordering::AcqRel);
}

impl Ticket {
    pub fn is_current(self) -> bool {
        self.0 == GENERATION.load(Ordering::Acquire)
    }

    /// Check on delivery, not when the worker queues its reply.
    pub fn publish(self, update: impl FnOnce(crate::app::AppState) + Send + 'static) {
        day::reactive::on_main(move || {
            if self.is_current() {
                if let Some(state) = crate::app::AppState::get_main() {
                    update(state);
                }
            }
        });
    }
}
