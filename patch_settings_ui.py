import re

with open("src/pages/settings.rs", "r") as f:
    content = f.read()

# Replace vertical tab buttons with picker
old_tabs = """        column((
            button("Основные").action(move || current_tab.set(0)),
            button("Безопасность").action(move || current_tab.set(1)),
            button("DevTools").action(move || current_tab.set(2)),
        ))
        .spacing(12.0)
        .align(HAlign::Leading)"""

new_tabs = """        picker(
            vec!["Основные", "Безопасность", "DevTools"],
            current_tab.clone(),
        )"""

content = content.replace(old_tabs, new_tabs)

with open("src/pages/settings.rs", "w") as f:
    f.write(content)
