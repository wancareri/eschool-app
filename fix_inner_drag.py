import re

with open('src/pages/diary.rs', 'r') as f:
    content = f.read()

# For week_view
content = content.replace("""                    .translation(strip_tx, 0.0),
                ))
            },
        ),
    ))
    .spacing(0.0)""",
"""                    .translation(strip_tx, 0.0),
                ))
                .on_drag(pager_drag(strip_tx.clone(), page_width.clone(), state.clone()))
            },
        ),
    ))
    .spacing(0.0)""")

# For summary_view
content = content.replace("""                    .translation(strip_tx, 0.0),
                ))
            },
        ),
    ))
    .spacing(0.0)
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 8.0, trailing: 0.0 })""",
"""                    .translation(strip_tx, 0.0),
                ))
                .on_drag(summary_drag(strip_tx.clone(), page_width.clone(), state.clone()))
            },
        ),
    ))
    .spacing(0.0)
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 8.0, trailing: 0.0 })""")

with open('src/pages/diary.rs', 'w') as f:
    f.write(content)
