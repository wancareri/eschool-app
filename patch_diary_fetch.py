import re

with open("src/features/diary.rs", "r") as f:
    content = f.read()

# Fix 1: set_lessons and week_cache
bad_block = """                        Ok(lessons) => {
                            set_lessons.set(lessons.clone());
                            let ci = cur_i;
                            let ls = lessons;
                            day::reactive::on_main(move || {
                                let state = AppState::ambient();
                                let mut wc = state.week_cache.get();
                                wc.insert(ci, ls);
                                state.week_cache.set(wc);
                            });
                        }"""

good_block = """                        Ok(lessons) => {
                            let ci = cur_i;
                            let ls = lessons;
                            day::reactive::on_main(move || {
                                let state = AppState::ambient();
                                state.lessons.setter().set(ls.clone());
                                let mut wc = state.week_cache.get();
                                wc.insert(ci, ls);
                                state.week_cache.set(wc);
                            });
                        }"""
content = content.replace(bad_block, good_block)

# Fix 2: set_lessons_loading
bad_loading = "            set_lessons_loading.set(false);"
good_loading = "            day::reactive::on_main(|| { AppState::ambient().lessons_loading.setter().set(false); });"
content = content.replace(bad_loading, good_loading)

# There might be another place in load_all? Let's check.
with open("src/features/diary.rs", "w") as f:
    f.write(content)
