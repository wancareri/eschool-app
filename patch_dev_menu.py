import re

with open('src/pages/settings.rs', 'r') as f:
    content = f.read()

# Add version label above the button("Показать токен")
content = content.replace('button("Показать токен")', 'label("Версия: v0.1.0"),\n                button("Показать токен")')

with open('src/pages/settings.rs', 'w') as f:
    f.write(content)
