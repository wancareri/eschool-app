import re

with open("src/pages/settings.rs", "r") as f:
    content = f.read()

# Make settings tabs vertical
tabs_horizontal = """        row((
            button("Основные").action(move || current_tab.set(0)),
            button("Безопасность").action(move || current_tab.set(1)),
            button("DevTools").action(move || current_tab.set(2)),
        ))
        .spacing(12.0)
        .padding(Insets { top: 8.0, leading: 20.0, bottom: 16.0, trailing: 20.0 }),"""

tabs_vertical = """        column((
            button("Основные").action(move || current_tab.set(0)),
            button("Безопасность").action(move || current_tab.set(1)),
            button("DevTools").action(move || current_tab.set(2)),
        ))
        .spacing(12.0)
        .align(HAlign::Leading)
        .padding(Insets { top: 8.0, leading: 20.0, bottom: 16.0, trailing: 20.0 }),"""

content = content.replace(tabs_horizontal, tabs_vertical)

# Center pin_setup_modal
pin_modal_str = "    column((\n        column((\n            label(move || res::str::app_title().format())"

# Wait, instead of row in the return, let's just do it in settings::render.
when_str = "        move || pin_setup_modal(show_setup, pin_enabled)\n    ).otherwise"
when_replacement = "        move || row((spacer().grow(), pin_setup_modal(show_setup, pin_enabled), spacer().grow())).grow().any()\n    ).otherwise"

content = content.replace(when_str, when_replacement)

with open("src/pages/settings.rs", "w") as f:
    f.write(content)
