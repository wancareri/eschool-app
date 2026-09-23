import re

with open('src/pages/diary.rs', 'r') as f:
    content = f.read()

# Fix pager_drag logic
content = re.sub(
r"""                let tx = strip_tx\.get\(\);\s*if tx < -threshold \{\s*actual_dx = strip_tx\.get\(\) \+ w;\s*dir = 1;\s*\} else if tx > -w \+ threshold \{\s*actual_dx = strip_tx\.get\(\);\s*dir = -1;\s*\} else \{\s*actual_dx = strip_tx\.get\(\) \+ w;\s*dir = 0;\s*\}""",
"""                let tx = strip_tx.get();
                if tx < -w - threshold {
                    actual_dx = -2.0 * w;
                    dir = 1;
                } else if tx > -w + threshold {
                    actual_dx = 0.0;
                    dir = -1;
                } else {
                    actual_dx = -w;
                    dir = 0;
                }""", content)

# Fix summary_drag logic
content = re.sub(
r"""                let tx = strip_tx\.get\(\);\s*if tx < -threshold \{\s*actual_dx = strip_tx\.get\(\) \+ w;\s*dir = 1;\s*\} else if tx > -w \+ threshold \{\s*actual_dx = strip_tx\.get\(\);\s*dir = -1;\s*\} else \{\s*actual_dx = strip_tx\.get\(\) \+ w;\s*dir = 0;\s*\}""",
"""                let tx = strip_tx.get();
                if tx < -w - threshold {
                    actual_dx = -2.0 * w;
                    dir = 1;
                } else if tx > -w + threshold {
                    actual_dx = 0.0;
                    dir = -1;
                } else {
                    actual_dx = -w;
                    dir = 0;
                }""", content)

# Remove .grow() from week_view and summary_view and their children
# week_view row
content = content.replace(""".on_drag(pager_drag(strip_tx.clone(), page_width.clone(), state.clone()))
                .grow()
            },
        ),
    ))
    .spacing(0.0)
    .background(Color::CLEAR)
    .grow()""",
""".on_drag(pager_drag(strip_tx.clone(), page_width.clone(), state.clone()))
            },
        ),
    ))
    .spacing(0.0)
    .background(Color::CLEAR)""")

# week_page
content = content.replace("""    ))
    .spacing(0.0)
    .width(w)
    .grow()""",
"""    ))
    .spacing(0.0)
    .width(w)""")

# summary_view row
content = content.replace(""".on_drag(summary_drag(strip_tx.clone(), page_width.clone(), state.clone()))
                .grow()
            },
        ),
    ))
    .spacing(0.0)
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 8.0, trailing: 0.0 })
    .grow()""",
""".on_drag(summary_drag(strip_tx.clone(), page_width.clone(), state.clone()))
            },
        ),
    ))
    .spacing(0.0)
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 8.0, trailing: 0.0 })""")

# summary_page
content = content.replace("""    ))
    .spacing(0.0)
    .width(w)
    .grow()""",
"""    ))
    .spacing(0.0)
    .width(w)""")

with open('src/pages/diary.rs', 'w') as f:
    f.write(content)
