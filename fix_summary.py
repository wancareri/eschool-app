import re

with open('src/pages/diary.rs', 'r') as f:
    content = f.read()

content = content.replace(""".on_drag(summary_drag(strip_tx.clone(), page_width.clone(), state.clone()))
                .grow()""", """.on_drag(summary_drag(strip_tx.clone(), page_width.clone(), state.clone()))""")
                
with open('src/pages/diary.rs', 'w') as f:
    f.write(content)
