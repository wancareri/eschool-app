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

# Center pin_setup_modal properly by forcing it to fill width using a row with spacers
pin_modal_str = "    column((\n        column((\n            label(move || res::str::app_title().format())"

# Replace the inner return of pin_setup_modal
old_modal_end = """    ))
    .spacing(8.0)
    .align(HAlign::Center)
    .padding(Insets { top: 80.0, leading: 40.0, bottom: 40.0, trailing: 40.0 })
    .grow()
    .any()"""

new_modal_end = """    ))
    .spacing(8.0)
    .align(HAlign::Center)
    .padding(Insets { top: 80.0, leading: 40.0, bottom: 40.0, trailing: 40.0 })
    
    // Wrap in a row to force horizontal centering
    let c = column((c,)).grow();
    row((spacer().grow(), c, spacer().grow())).grow().any()"""

# We need to assign the column to `c` first
old_modal_start = """    let error = Signal::new(String::new());

    column(("""

new_modal_start = """    let error = Signal::new(String::new());

    let c = column(( """

content = content.replace(old_modal_start, new_modal_start)
content = content.replace(old_modal_end, new_modal_end)

with open("src/pages/settings.rs", "w") as f:
    f.write(content)
