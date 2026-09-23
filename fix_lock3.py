import re

with open('src/features/diary.rs', 'r') as f:
    content = f.read()

content = content.replace("""lazy_static::lazy_static! {
    static ref GLOBAL_FETCH_LOCK: Mutex<()> = Mutex::new(());
}""",
"""static GLOBAL_FETCH_LOCK: std::sync::OnceLock<Mutex<()>> = std::sync::OnceLock::new();
fn fetch_lock() -> &'static Mutex<()> {
    GLOBAL_FETCH_LOCK.get_or_init(|| Mutex::new(()))
}""")

content = content.replace("""        let _lock = GLOBAL_FETCH_LOCK.lock().unwrap();""",
"""        let _lock = fetch_lock().lock().unwrap();""")

with open('src/features/diary.rs', 'w') as f:
    f.write(content)
