import re

with open('src/pages/diary.rs', 'r') as f:
    content = f.read()

# Add the zstack and the touch catching rect
content = re.sub(
    r"    pull_to_refresh\(refreshing, scroll\(column\(\(\n\s*zstack\(\(",
    r"""    zstack((
        // Touch catcher for empty space
        rect().color(Color::rgba(255.0, 255.0, 255.0, 0.02)).grow(),
        pull_to_refresh(refreshing, scroll(column((
        zstack((""".strip(),
    content
)

# Replace the end of render
content = re.sub(
    r"        \}\n    \}\)\n\}",
    r"""        }
    })
    )) // end of zstack wrapping pull_to_refresh
    .on_drag(global_drag(strip_tx.clone(), page_width.clone(), state.clone(), show_summary.clone()))
    .grow()
}""",
    content
)

with open('src/pages/diary.rs', 'w') as f:
    f.write(content)
