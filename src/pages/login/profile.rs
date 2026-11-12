use super::*;

#[allow(dead_code)]
fn profile_view(state: AppState) -> impl Piece {
    let remember = crate::app::data::credentials::load("auth.remember_me")
        .map(|v| v == "true")
        .unwrap_or(false);
    let show_change_pwd = Signal::new(false);

    zstack((
        column((
            column((
                label(move || res::str::settings_title().format()).font(Font::LargeTitle),
                label("Профиль").font(Font::Subheadline).secondary(),
            ))
            .spacing(6.0)
            .padding(Insets {
                top: 16.0,
                leading: 20.0,
                bottom: 12.0,
                trailing: 20.0,
            })
            .grow(Dimension::Width),
            spacer(),
            form((
                section((
                    label(move || state.full_name.get()).font(Font::Title2),
                    label(move || state.school_name.get())
                        .font(Font::Body)
                        .secondary(),
                    when(
                        move || !state.class_label.get().is_empty(),
                        move || {
                            label(move || format!("Класс: {}", state.class_label.get()))
                                .font(Font::Body)
                                .secondary()
                        },
                    ),
                    label(move || {
                        if remember {
                            "Сессия сохранена"
                        } else {
                            "Сессия не сохранена"
                        }
                    })
                    .font(Font::Caption)
                    .secondary(),
                ))
                .title(move || {
                    if state.is_parent.get() {
                        "Родитель"
                    } else {
                        "Ученик"
                    }
                }),
                when(
                    move || state.is_parent.get() && !state.parent_children.get().is_empty(),
                    move || {
                        let children = state.parent_children.get();
                        let rows: Vec<AnyPiece> = children
                            .into_iter()
                            .enumerate()
                            .map(|(this_idx, ch)| {
                                let name = ch.full_name.clone();
                                row((
                                    column((
                                        label(name.clone()).font(Font::Headline),
                                        when(
                                            move || state.active_child_index.get() == this_idx,
                                            move || {
                                                label("Выбран для дневника")
                                                    .font(Font::Caption)
                                                    .color(move || {
                                                        Color::hex(state.accent_color.get())
                                                    })
                                                    .any()
                                            },
                                        ),
                                    ))
                                    .grow(Dimension::Both),
                                    when(
                                        move || state.active_child_index.get() != this_idx,
                                        move || {
                                            button("Выбрать")
                                                .action(move || {
                                                    crate::app::preferences::haptics::tick();
                                                    crate::pages::diary::data::switch_child(state, this_idx);
                                                })
                                                .any()
                                        },
                                    ),
                                ))
                                .align(VAlign::Center)
                                .padding(Insets {
                                    top: 6.0,
                                    leading: 0.0,
                                    bottom: 6.0,
                                    trailing: 0.0,
                                })
                                .any()
                            })
                            .collect();

                        section(rows).title("Дети в дневнике").any()
                    },
                ),
                section((
                    when(
                        move || state.loading.get(),
                        || label("Загрузка данных…").font(Font::Caption).secondary(),
                    ),
                    when(
                        move || !state.error_msg.get().is_empty(),
                        move || {
                            label(move || state.error_msg.get())
                                .font(Font::Caption)
                                .color(colors::ERROR)
                        },
                    ),
                ))
                .title("Статус"),
            ))
            .padding(Insets {
                top: 0.0,
                leading: 0.0,
                bottom: 16.0,
                trailing: 0.0,
            }),
            form((section((
                button("Обновить данные").action(move || {
                    crate::app::preferences::haptics::tick();
                    crate::pages::diary::data::load_all(state);
                }),
                when(
                    move || state.is_parent.get(),
                    move || {
                        button("Сменить пароль")
                            .action(move || {
                                crate::app::preferences::haptics::tick();
                                show_change_pwd.set(true);
                            })
                            .any()
                    },
                ),
                divider(),
                button("Выйти").action(move || {
                    crate::app::preferences::haptics::pop();
                    crate::pages::login::auth::logout(state);
                }),
            ))
            .title("Действия"),))
            .padding(Insets {
                top: 0.0,
                leading: 0.0,
                bottom: 20.0,
                trailing: 0.0,
            }),
        ))
        .spacing(0.0)
        .grow(Dimension::Both),
        when(
            move || show_change_pwd.get(),
            move || {
                change_password_form(state, show_change_pwd)
                    .animation(AnimSpec::ease_out(300))
                    .any()
            },
        ),
    ))
    .grow(Dimension::Both)
}
