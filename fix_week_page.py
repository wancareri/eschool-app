import re

with open('src/pages/diary.rs', 'r') as f:
    content = f.read()

content = content.replace('''        when(
            move || get_lessons().is_empty() && offset != 0,
            || label("Нет данных за этот период")
                .font(Font::Body)
                .secondary()
                .align(TextAlign::Center)
                .padding(Insets { top: 24.0, leading: PAD, bottom: 24.0, trailing: PAD }),
        ),''',
'''        when(
            move || get_lessons().is_empty() && offset == 0 && state.lessons_loading.get(),
            || column((
                spacer(),
                widgets::spinner::render(),
                spacer(),
            )).height(300.0),
        ),
        when(
            move || get_lessons().is_empty() && (!state.lessons_loading.get() || offset != 0),
            || label("Нет данных за этот период")
                .font(Font::Body)
                .secondary()
                .align(TextAlign::Center)
                .padding(Insets { top: 24.0, leading: PAD, bottom: 24.0, trailing: PAD }),
        ),''')

with open('src/pages/diary.rs', 'w') as f:
    f.write(content)
