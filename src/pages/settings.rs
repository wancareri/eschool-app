use crate::app::AppState;
use crate::features;
use crate::widgets;
use crate::res;
use crate::shared::{biometric, colors, nslog, pin};
use day::prelude::*;
use day_piece_texteditor::text_editor;

pub fn render() -> impl Piece {
    let state = AppState::ambient();
    let current_tab = Signal::new(0usize);
    let show_setup = Signal::new(false);
    let pin_enabled = Signal::new(crate::shared::pin::is_enabled());

    // When toggle turns ON -> show setup modal
    {
        let show = show_setup.clone();
        day::reactive::watch(
            move || pin_enabled.get(),
            move |on, old| {
                if old == Some(&false) && *on {
                    show.set(true);
                }
            },
        );
    }
    // When toggle turns OFF -> delete PIN
    day::reactive::watch(
        move || pin_enabled.get(),
        move |on, old| {
            if old == Some(&true) && !*on {
                crate::shared::pin::delete_pin();
                crate::shared::nslog::nslog("[PIN] Disabled");
            }
        },
    );

    
    when(
        move || show_setup.get(),
        move || pin_setup_modal(show_setup, pin_enabled)
    ).otherwise(move || zstack((
        column((
        column((
            label(move || res::str::settings_title().format())
                .font(Font::LargeTitle).align(TextAlign::Center),
            label("Настройки").font(Font::Subheadline).secondary().align(TextAlign::Center),
        ))
        .spacing(6.0)
        .padding(Insets { top: 16.0, leading: 20.0, bottom: 8.0, trailing: 20.0 }),

        picker(
            vec!["Основные", "Вид", "Защита", "Dev"],
            current_tab.clone(),
        )
        .segmented()
        .padding(Insets { top: 8.0, leading: 20.0, bottom: 16.0, trailing: 20.0 }),

        scroll(column((
            when(
                move || current_tab.get() == 0,
                move || column((
                    when(move || state.is_authenticated.get(), move || profile_section(state)),
                    when(move || state.is_authenticated.get(), move || logout_section(state)),
                )).spacing(0.0).grow()
            ),

            when(
                move || current_tab.get() == 1,
                move || column((
                    appearance_section(state),
                    system_settings(),
                )).spacing(0.0).grow()
            ),
            
            when(
                move || current_tab.get() == 2,
                move || column((
                    pin_section(state, pin_enabled),
                    biometric_section(state),
                )).spacing(0.0).grow()
            ),

            when(
                move || current_tab.get() == 3,
                move || dev_settings(state)
            ),
        ))
        .spacing(0.0)
        .grow())
        .grow()
    ))
    .grow(),

    // Sticky status indicator in top-left corner
    widgets::conn_status::render()
        .padding(Insets { top: 16.0, leading: 16.0, bottom: 0.0, trailing: 0.0 }),
    ))
    .align(Alignment::TopLeading)
    .grow()
    .any())
}

fn profile_section(state: AppState) -> impl Piece {
    form((
        section(
            (
                label(move || state.full_name.get())
                    .font(Font::Title3).align(TextAlign::Center),
                label(move || state.school_name.get())
                    .font(Font::Body).secondary().align(TextAlign::Center),
                when(
                    move || !state.class_label.get().is_empty(),
                    move || label(move || format!("Класс: {}", state.class_label.get()))
                        .font(Font::Body).secondary().align(TextAlign::Center),
                ),
            )
        ).title("Профиль"),
    ))
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}

fn appearance_section(state: AppState) -> impl Piece {
    let s = state;
    column((
        form((
            section(
                (
                    label("Акцентный цвет").font(Font::Headline),
                    accent_option(state, "Синий", colors::BLUE),
                    accent_option(state, "Зелёный", colors::GREEN),
                    accent_option(state, "Фиолетовый", colors::PURPLE),
                    accent_option(state, "Оранжевый", colors::ORANGE),
                    accent_option(state, "Красный", colors::RED),
                    label("Для полного применения нового цвета к системным элементам (таб-бар, навигация) перезапустите приложение или используйте кнопку ниже.")
                        .font(Font::Caption)
                        .secondary(),
                    button("Быстрая перезагрузка интерфейса")
                        .action(move || {
                            let tok = s.ui_reload_token.get();
                            s.ui_reload_token.set(tok + 1);
                        })
                        .id("quick-reload-btn"),
                )
            ).title("Цвета"),
        )),
    ))
    .spacing(0.0)
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}

fn system_settings() -> impl Piece {
    form((day_piece_settings::settings_sections(
        crate::THEME_KEY,
        crate::LOCALE_KEY,
        res::locales::ALL,
    ),))
}

fn accent_option(state: AppState, lbl: &'static str, hex: u32) -> impl Piece {
    let s = state;
    button(move || {
        if s.accent_color.get() == hex { format!("\u{2713} \u{25CF} {lbl}") } else { format!("\u{25CF} {lbl}") }
    })
    .id(format!("accent-{hex}"))
    .action(move || {
        day::prefs::set("app.accent_color", &hex.to_string());
        colors::set_accent(hex);
        state.accent_color.set(hex);
        #[cfg(target_os = "ios")]
        colors::apply_ios_tint(hex);
        let cur_tok = state.ui_reload_token.get();
        state.ui_reload_token.set(cur_tok + 1);
    })
}

fn pin_section(_state: AppState, pin_enabled: Signal<bool>) -> impl Piece {
    column((
        form((
            section(
                (
                    row((
                        label("PIN-код при входе")
                            .font(Font::Body)
                            .grow(),
                        toggle(pin_enabled),
                    ))
                    .spacing(8.0),

                    when(
                        move || crate::shared::pin::is_enabled(),
                        || label("PIN-код активен")
                            .font(Font::Caption)
                            .secondary(),
                    ),
                )
            ).title("Блокировка"),
        )),
    ))
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}

fn pin_setup_modal(show_setup: Signal<bool>, pin_enabled: Signal<bool>) -> impl Piece {
    let step = Signal::new(0u8);
    let input = Signal::new(String::new());
    let confirm = Signal::new(String::new());
    let error = Signal::new(String::new());

    let c = column(( 
        column((
            label(move || res::str::app_title().format())
                .font(Font::LargeTitle)
                .align(TextAlign::Center),
            label(move || {
                if step.get() == 0 { "Придумайте PIN-код" } else { "Повторите PIN-код" }
            })
            .font(Font::Subheadline)
            .secondary()
            .align(TextAlign::Center)
            .padding(Insets { top: 4.0, leading: 0.0, bottom: 0.0, trailing: 0.0 }),
        ))
        .spacing(4.0)
        .align(HAlign::Center),

        row((
            setup_dot(0, input),
            setup_dot(1, input),
            setup_dot(2, input),
            setup_dot(3, input),
        ))
        .spacing(20.0)
        .padding(Insets { top: 24.0, leading: 0.0, bottom: 8.0, trailing: 0.0 })
        .align(VAlign::Center),

        when(
            move || !error.get().is_empty(),
            move || label(move || error.get())
                .font(Font::Caption)
                .color(colors::ERROR)
                .align(TextAlign::Center)
                .padding(Insets { top: 4.0, leading: 0.0, bottom: 0.0, trailing: 0.0 }),
        ),

        setup_numpad(input, step, confirm, error, show_setup, pin_enabled),
        
        button("Отмена")
            .action(move || {
                show_setup.set(false);
                pin_enabled.set(false);
            })
            .id("pin-modal-cancel")
            .padding(Insets { top: 24.0, leading: 0.0, bottom: 0.0, trailing: 0.0 }),
    ))
    .spacing(8.0)
    .align(HAlign::Center)
    .padding(Insets { top: 80.0, leading: 40.0, bottom: 40.0, trailing: 40.0 });
    
    // Wrap in a row to force horizontal centering
    let c = column((c,)).grow();
    row((spacer().grow(), c, spacer().grow())).grow().any()
}

fn setup_dot(index: usize, input: Signal<String>) -> impl Piece {
    button(move || {
        if input.get().len() > index { "●" } else { "○" }
    })
    .action(|| {})
    .id(format!("setup-dot-{index}"))
    .frame(20.0, 20.0)
}

fn setup_numpad(
    input: Signal<String>,
    step: Signal<u8>,
    confirm: Signal<String>,
    error: Signal<String>,
    setup_mode: Signal<bool>,
    _pin_enabled: Signal<bool>,
) -> impl Piece {
    fn setup_key(state: Signal<String>, key: String, confirm: Signal<String>, step: Signal<u8>, error: Signal<String>, setup_mode: Signal<bool>) -> impl Piece {
        let k = key.clone();
        let k2 = key.clone();
        if k.is_empty() {
            spacer().frame(72.0, 52.0).any()
        } else if k == "⌫" {
            button("⌫")
                .action(move || {
                    let mut v = state.get();
                    if !v.is_empty() {
                        v.pop();
                        state.set(v);
                        error.set("".into());
                    }
                })
                .frame(72.0, 52.0)
                .id(format!("sk-{k2}"))
                .any()
        } else {
            button(k.clone())
                .action(move || {
                    let mut v = state.get();
                    if v.len() >= 4 { return; }
                    v.push_str(&k);
                    state.set(v.clone());
                    error.set("".into());
                    
                    if v.len() == 4 {
                        if step.get() == 0 {
                            confirm.set(v);
                            state.set(String::new());
                            step.set(1);
                        } else if v == confirm.get() {
                            pin::save_pin(&v);
                            setup_mode.set(false);
                            state.set(String::new());
                            confirm.set(String::new());
                            nslog::nslog("[PIN] Setup complete");
                        } else {
                            error.set("PIN-коды не совпадают".into());
                            state.set(String::new());
                        }
                    }
                })
                .frame(72.0, 52.0)
                .id(format!("sk-{k2}"))
                .any()
        }
    }

    column((
        row((
            setup_key(input.clone(), "1".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
            setup_key(input.clone(), "2".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
            setup_key(input.clone(), "3".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
        )).spacing(16.0),
        row((
            setup_key(input.clone(), "4".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
            setup_key(input.clone(), "5".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
            setup_key(input.clone(), "6".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
        )).spacing(16.0),
        row((
            setup_key(input.clone(), "7".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
            setup_key(input.clone(), "8".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
            setup_key(input.clone(), "9".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
        )).spacing(16.0),
        row((
            setup_key(input.clone(), "".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
            setup_key(input.clone(), "0".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
            setup_key(input.clone(), "⌫".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
        )).spacing(16.0),
    ))
    .spacing(12.0)
    .align(HAlign::Center)
}

fn biometric_section(_state: AppState) -> impl Piece {
    let available = biometric::is_available();
    let biometric_on = Signal::new(biometric::is_enabled());
    let show_alert = Signal::new(false);

    // Persist toggle changes only when PIN is enabled
    day::reactive::watch(
        move || biometric_on.get(),
        move |val, old| {
            if old.is_some() {
                if pin::is_enabled() {
                    biometric::set_enabled(*val);
                } else {
                    // PIN not enabled — revert and show alert
                    biometric_on.set(false);
                    show_alert.set(true);
                }
            }
        },
    );

    form((
        section(
            (
                when(
                    move || available,
                    move || {
                        row((
                            label("Вход по Face ID / Touch ID")
                                .font(Font::Body)
                                .grow(),
                            toggle(biometric_on),
                        ))
                        .spacing(8.0)
                    },
                ),
                when(
                    move || !available && pin::is_enabled(),
                    || label("Биометрия не поддерживается")
                        .font(Font::Body)
                        .secondary(),
                ),
                when(
                    move || !pin::is_enabled() && available,
                    || label("Сначала включите PIN-код")
                        .font(Font::Caption)
                        .secondary()
                        .color(colors::WARNING),
                ),

                // Alert when trying to enable biometric without PIN
                when(
                    move || show_alert.get(),
                    move || {
                        column((
                            label("Для использования биометрии сначала включите PIN-код")
                                .font(Font::Caption)
                                .color(colors::ERROR),
                            button("Понятно")
                                .action(move || show_alert.set(false))
                                .id("bio-alert-ok"),
                        ))
                        .spacing(8.0)
                    },
                ),
            )
        ).title("Безопасность"),
    ))
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}

fn logout_section(state: AppState) -> impl Piece {
    form((
        section(
            (button("Выйти из аккаунта")
                .action(move || features::auth::logout(state)),)
        ).title("Аккаунт"),
    ))
    .padding(Insets { top: 16.0, leading: 0.0, bottom: 20.0, trailing: 0.0 })
}

pub fn settings_body() -> impl Piece {
    form((day_piece_settings::settings_sections(
        crate::THEME_KEY,
        crate::LOCALE_KEY,
        res::locales::ALL,
    ),))
}

fn dev_settings(state: AppState) -> impl Piece {
    let log_doc = Signal::new(StyledText::plain(nslog::get_logs()));

    form((
        section(
            (
                label("Инструменты разработчика").font(Font::Headline).color(colors::WARNING),

                label("Версия: v0.1.0"),
                button("Показать токен")
                    .action(move || {
                        let msg = if let Some(t) = features::auth::get_token() {
                            let preview = if t.len() > 40 { &t[..40] } else { &t };
                            format!("Token: {preview}...")
                        } else {
                            "Нет токена".into()
                        };
                        nslog::nslog(&format!("[Dev] {msg}"));
                        log_doc.set(StyledText::plain(nslog::get_logs()));
                    }),

                button("Обновить токен")
                    .action(move || {
                        nslog::nslog("[Dev] Manual token refresh...");
                        let log_setter = log_doc.setter();
                        std::thread::spawn(move || {
                            let msg: String = match features::auth::try_refresh_token() {
                                Some(_) => "Refresh OK".into(),
                                None => "Refresh FAILED".into(),
                            };
                            nslog::nslog(&format!("[Dev] {msg}"));
                            log_setter.set(StyledText::plain(nslog::get_logs()));
                        });
                    }),

                button("Очистить кэш")
                    .action(move || {
                        day::prefs::set("diary.cache", "");
                        nslog::nslog("[Dev] Cache cleared");
                        log_doc.set(StyledText::plain(nslog::get_logs()));
                    }),

                button("Перезагрузить данные")
                    .action(move || {
                        log_doc.set(StyledText::plain("Загрузка...".to_string()));
                        features::diary::load_all(state);
                    }),

                button("Показать ID")
                    .action(move || {
                        let (sid, cid, pid) = features::auth::get_stored_ids();
                        nslog::nslog(&format!("[Dev] school={sid} class={cid} profile={pid}"));
                        log_doc.set(StyledText::plain(nslog::get_logs()));
                    }),
            )
        ).title("Dev Tools"),

        section(
            (
                label("Логи приложения").font(Font::Headline).color(colors::INFO),

                text_editor(log_doc.clone())
                    .editable(false)
                    .min_lines(12),

                row((
                    button("Обновить")
                        .action(move || {
                            log_doc.set(StyledText::plain(nslog::get_logs()));
                        }),
                    button("Очистить логи")
                        .action(move || {
                            nslog::clear_logs();
                            log_doc.set(StyledText::plain(String::new()));
                        }),
                )).spacing(8.0),
            )
        ).title("Логи"),
    ))
    .padding(Insets { top: 16.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}

