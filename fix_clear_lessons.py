import re

with open('src/features/diary.rs', 'r') as f:
    content = f.read()

content = content.replace('''} else {
        set_lessons_loading.set(true);
        set_conn.set(crate::app::ConnStatus::Connecting);
        nslog::nslog(&format!("[Diary] load_week idx={idx} uuid={week_uuid}"));
    }''',
'''} else {
        set_lessons.set(Vec::new());
        set_lessons_loading.set(true);
        set_conn.set(crate::app::ConnStatus::Connecting);
        nslog::nslog(&format!("[Diary] load_week idx={idx} uuid={week_uuid}"));
    }''')

with open('src/features/diary.rs', 'w') as f:
    f.write(content)
