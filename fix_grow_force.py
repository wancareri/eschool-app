import re

with open('src/pages/diary.rs', 'r') as f:
    content = f.read()

content = content.replace("""                    .translation(strip_tx, 0.0),
                ))
                .on_drag(pager_drag(strip_tx.clone(), page_width.clone(), state.clone()))
            },
        ),
    ))
    .spacing(0.0)
}""",
"""                    .translation(strip_tx, 0.0),
                ))
                .on_drag(pager_drag(strip_tx.clone(), page_width.clone(), state.clone()))
                .grow()
            },
        ),
    ))
    .spacing(0.0)
    .grow()
}""")

with open('src/pages/diary.rs', 'w') as f:
    f.write(content)
