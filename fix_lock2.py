import re

with open('src/features/diary.rs', 'r') as f:
    content = f.read()

# Undo previous replace
content = content.replace("""    std::thread::spawn(move || {
        let _lock = crate::shared::secure::GLOBAL_FETCH_LOCK.lock().unwrap();
        let client = blocking::build_client(&token);

        let state = AppState::ambient();
        if state.current_week_index.get() != cur_i {
            return; // Abort if user navigated away while waiting for lock
        }

        if need_current {""",
"""    std::thread::spawn(move || {
        let _lock = GLOBAL_FETCH_LOCK.lock().unwrap();
        let client = blocking::build_client(&token);

        let state = AppState::ambient();
        if state.current_week_index.get() != cur_i {
            return; // Abort if user navigated away while waiting for lock
        }

        if need_current {""")

# Add lazy_static lock
if "GLOBAL_FETCH_LOCK" not in content[:1000]:
    content = content.replace("""use crate::shared::{cache, nslog};""",
"""use crate::shared::{cache, nslog};
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref GLOBAL_FETCH_LOCK: Mutex<()> = Mutex::new(());
}""")

with open('src/features/diary.rs', 'w') as f:
    f.write(content)
