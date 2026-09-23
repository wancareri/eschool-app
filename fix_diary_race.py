import re

with open('src/features/diary.rs', 'r') as f:
    content = f.read()

# Fix the main request
content = content.replace('''day::reactive::on_main(move || {
                                let state = AppState::ambient();
                                state.lessons.setter().set(ls.clone());
                                let mut wc = state.week_cache.get();
                                wc.insert(ci, ls);
                                state.week_cache.set(wc);
                            });''',
'''day::reactive::on_main(move || {
                                let state = AppState::ambient();
                                if state.current_week_index.get() == ci {
                                    state.lessons.setter().set(ls.clone());
                                }
                                let mut wc = state.week_cache.get();
                                wc.insert(ci, ls);
                                state.week_cache.set(wc);
                            });''')

# Fix the prefetch request
content = content.replace('''day::reactive::on_main(move || {
                            let state = AppState::ambient();
                            let mut wc = state.week_cache.get();
                            wc.insert(ni, ls);
                            state.week_cache.set(wc);
                        });''',
'''day::reactive::on_main(move || {
                            let state = AppState::ambient();
                            if state.current_week_index.get() == ni {
                                state.lessons.setter().set(ls.clone());
                            }
                            let mut wc = state.week_cache.get();
                            wc.insert(ni, ls);
                            state.week_cache.set(wc);
                        });''')

with open('src/features/diary.rs', 'w') as f:
    f.write(content)
