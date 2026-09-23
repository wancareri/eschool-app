import re

with open('src/pages/settings.rs', 'r') as f:
    content = f.read()

picker_code = """
        picker(
            vec!["Основные", "Вид", "Защита", "Dev"],
            current_tab.clone(),
        )
        .segmented()
        .padding(Insets { top: 8.0, leading: 20.0, bottom: 16.0, trailing: 20.0 }),
"""

# replace the settings_tabs(...) call with the picker code
content = re.sub(r'settings_tabs\(current_tab\.clone\(\)\),', picker_code.strip() + ',', content)

# remove the fn settings_tabs implementation
content = re.sub(r'fn settings_tabs.*?\n}\n', '', content, flags=re.DOTALL)

with open('src/pages/settings.rs', 'w') as f:
    f.write(content)
