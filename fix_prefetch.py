import re

with open('src/features/diary.rs', 'r') as f:
    content = f.read()

content = content.replace("""        // Quiet neighbor prefetch
        for (ni, nuuid) in prefetch {""",
"""        drop(_lock);
        // Quiet neighbor prefetch
        for (ni, nuuid) in prefetch {
            let _plock = fetch_lock().lock().unwrap();
            if AppState::ambient().current_week_index.get() != cur_i {
                break;
            }""")

with open('src/features/diary.rs', 'w') as f:
    f.write(content)
