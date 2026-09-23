import re

with open('src/features/diary.rs', 'r') as f:
    content = f.read()

content = content.replace("""    std::thread::spawn(move || {
        let client = blocking::build_client(&token);

        if need_current {""",
"""    std::thread::spawn(move || {
        let _lock = crate::shared::secure::GLOBAL_FETCH_LOCK.lock().unwrap();
        let client = blocking::build_client(&token);

        let state = AppState::ambient();
        if state.current_week_index.get() != cur_i {
            return; // Abort if user navigated away while waiting for lock
        }

        if need_current {""")

with open('src/features/diary.rs', 'w') as f:
    f.write(content)
