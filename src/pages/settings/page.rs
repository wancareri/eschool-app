use crate::app::AppState;
use crate::res;
use crate::pages::pin::biometric;
use crate::app::preferences::colors;
use crate::app::preferences::haptics;
use crate::ios::nslog;
use crate::pages::pin::security as pin;
use day::prelude::*;

pub fn render() -> impl Piece {
    let state = AppState::ambient();
    let current_tab = state.settings_tab;
    let pager_state = Signal::new(day::pager::PagerState {
        order: vec![0, 1, 2, 3],
        selected: current_tab.get().min(3),
        epoch: 1,
    });
    let pager_busy = Signal::new(false);
    day::reactive::watch(
        move || (current_tab.get(), pager_busy.get()),
        move |&(selected, busy), _| {
            if !busy && pager_state.get().selected != selected {
                pager_state.update(|s| {
                    s.selected = selected.min(3);
                    s.epoch += 1;
                });
            }
        },
    );

    let pin_enabled = Signal::new(crate::pages::pin::security::is_enabled());

    // When toggle turns ON -> activate pin setup at window level (hiding bottom bar)
    {
        day::reactive::watch(
            move || pin_enabled.get(),
            move |on, old| {
                if old == Some(&false) && *on {
                    crate::pages::pin::open_setup(state);
                }
            },
        );
    }
    // When toggle turns OFF -> delete PIN and disable biometric
    day::reactive::watch(
        move || pin_enabled.get(),
        move |on, old| {
            if old == Some(&true) && !*on {
                crate::pages::pin::security::delete_pin();
                crate::pages::pin::biometric::set_enabled(false);
                crate::ios::nslog::nslog("[PIN] Disabled, Face ID also disabled");
            }
        },
    );
    // When setup closes, sync toggle state with whether PIN was actually set
    {
        let pe = pin_enabled;
        day::reactive::watch(
            move || state.pin_setup_active.get(),
            move |active, old| {
                if old == Some(&true) && !*active {
                    pe.set(crate::pages::pin::security::is_enabled());
                }
            },
        );
    }

    let piece = zstack((
        column((


            picker(current_tab, vec![(0usize, "Основные"), (1, "Вид"), (2, "Защита"), (3, "Dev")])
            .segmented()
            .enabled(move || !pager_busy.get())
            .id(day::id!(settings_tabs))
            .padding(Insets {
                top: 4.0,
                leading: 20.0,
                bottom: 14.0,
                trailing: 20.0,
            }),

            day::pager(
                pager_state,
                pager_busy,
                (0..4).map(|tab| scroll(tab_body(state, tab, pin_enabled)).any()).collect(),
                move |selected| {
                    if current_tab.get() != selected {
                        haptics::tick();
                        current_tab.set(selected);
                    }
                    pager_state.update(|s| s.selected = selected);
                },
            ).id(day::id!(settings_pager)).grow(Dimension::Both),
        ))
        .grow(Dimension::Both),
    ))
    .align(Alignment::TopLeading)
    .grow(Dimension::Both);


    piece
}

fn tab_body(state: AppState, tab: usize, pin_enabled: Signal<bool>) -> impl Piece {
    match tab {
        0 => column((
            when(move || state.is_authenticated.get(), move || profile_section(state)),
            when(move || state.is_authenticated.get(), move || logout_section(state)),
            spacer().frame(0.0, 16.0),
        ))
        .spacing(0.0)
        .grow(Dimension::Both)
        .any(),
        1 => column((
            appearance_section(state),
            haptics_section(state),
            system_settings(),
            spacer().frame(0.0, 16.0),
        ))
        .spacing(0.0)
        .grow(Dimension::Both)
        .any(),
        2 => column((
            pin_section(state, pin_enabled),
            biometric_section(state, pin_enabled),
            spacer().frame(0.0, 16.0),
        ))
        .spacing(0.0)
        .grow(Dimension::Both)
        .any(),
        _ => column((
            dev_settings(state),
            spacer().frame(0.0, 16.0),
        ))
        .spacing(0.0)
        .grow(Dimension::Both)
        .any(),
    }
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
                )
            ).title("Цвета"),
        )),
    ))
    .spacing(0.0)
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}

/// Push an accent through every live consumer — the signal plus the UIKit tint
/// cascade. Nothing remounts: nav glyphs are template icons that follow the
/// window tint, and every other accent consumer is a reactive closure.
fn accent_apply(state: AppState, hex: u32) {
    colors::set_accent(hex);
    state.accent_color.set(hex);
    #[cfg(feature = "uikit")]
    colors::apply_ios_tint(hex);
}

/// Land an accent: persist it and push it through every live consumer. The UI
/// recolors in place — no nav reload, no page remount, the settings pager
/// stays exactly where the user put it.
fn accent_commit(state: AppState, hex: u32) {
    day::prefs::set("app.accent_color", &hex.to_string());
    accent_apply(state, hex);
    haptics::tick();
}

/// The tactile layer's master switch — default ON, persisted as `app.haptics`.
fn haptics_section(_state: AppState) -> impl Piece {
    let haptics_on = Signal::new(haptics::is_enabled());
    day::reactive::watch(
        move || haptics_on.get(),
        move |val, old| {
            if old.is_some() {
                haptics::set_enabled(*val);
            }
        },
    );

    form((
        section(
            (
                row((
                    label("Вибрация при действиях")
                        .font(Font::Body)
                        .grow(Dimension::Both),
                    toggle(haptics_on),
                ))
                .spacing(8.0),
                label("Тактильный отклик при перелистывании недель и смене разделов")
                    .font(Font::Caption)
                    .secondary(),
            )
        ).title("Ощущения"),
    ))
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}

/// Keep the existing preference keys so upgrading preserves the user's choices.
pub fn apply_startup() {
    if day::env("DAY_THEME").is_none() {
        let appearance = match day::prefs::get(crate::THEME_KEY).as_deref() {
            Some("dark") => Some(true),
            Some("light") => Some(false),
            _ => None,
        };
        day::set_appearance(appearance);
    }
    if day::env("DAY_LOCALE").is_none() {
        if let Some(locale) = day::prefs::get(crate::LOCALE_KEY).filter(|v| !v.is_empty()) {
            set_locale(&locale);
        }
    }
}

fn system_settings() -> impl Piece {
    let theme = Signal::new(match day::prefs::get(crate::THEME_KEY).as_deref() {
        Some("light") => 1usize,
        Some("dark") => 2,
        _ => 0,
    });
    let language = Signal::new(day::prefs::get(crate::LOCALE_KEY).unwrap_or_default());
    day::reactive::watch(move || theme.get(), move |value, old| {
        if old.is_none() { return; }
        match value {
            1 => day::prefs::set(crate::THEME_KEY, "light"),
            2 => day::prefs::set(crate::THEME_KEY, "dark"),
            _ => day::prefs::remove(crate::THEME_KEY),
        };
        day::set_appearance(match value { 1 => Some(false), 2 => Some(true), _ => None });
    });
    day::reactive::watch(move || language.get(), move |value, old| {
        if old.is_none() { return; }
        day::prefs::set(crate::LOCALE_KEY, value);
        let locale = if value.is_empty() { crate::app::preferences::locale::system_locale() } else { value.clone() };
        set_locale(&locale);
    });
    let languages = move || {
        let mut options = vec![(String::new(), res::str::auth_theme_system().format())];
        options.extend(res::locales::ALL.iter().map(|(tag, name)| (tag.to_string(), name.to_string())));
        options
    };
    form((
        section((picker(theme, move || vec![
            (0usize, res::str::auth_theme_system().format()),
            (1, res::str::auth_theme_light().format()),
            (2, res::str::auth_theme_dark().format()),
        ]).segmented(),)).title(res::str::auth_appearance()),
        section((picker(language, languages),)).title(res::str::auth_language()),
    ))
}

fn accent_option(state: AppState, lbl: &'static str, hex: u32) -> impl Piece {
    let s = state;
    button(move || {
        if s.accent_color.get() == hex { format!("\u{2713} \u{25CF} {lbl}") } else { format!("\u{25CF} {lbl}") }
    })
    .id(day::id!(accent, hex))
    .action(move || accent_commit(state, hex))
}

fn pin_section(state: AppState, pin_enabled: Signal<bool>) -> impl Piece {
    column((
        form((
            section(
                (
                    row((
                        label("PIN-код при входе")
                            .font(Font::Body)
                            .grow(Dimension::Both),
                        toggle(pin_enabled),
                    ))
                    .spacing(8.0),

                    when(
                        move || crate::pages::pin::security::is_enabled(),
                        move || row((
                            label("PIN-код активен")
                                .font(Font::Caption)
                                .secondary()
                                .grow(Dimension::Both),
                            button("Сменить")
                                .action(move || {
                                    crate::app::preferences::haptics::tick();
                                    crate::pages::pin::open_setup(state);
                                }),
                        )),
                    ),
                )
            ).title("Блокировка"),
        )),
    ))
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}

fn biometric_section(state: AppState, pin_enabled: Signal<bool>) -> impl Piece {
    let available = biometric::is_available();
    let biometric_on = Signal::new(biometric::is_enabled());
    let notices = crate::components::notice::presenter(state);

    // When PIN is disabled, automatically turn biometric off
    day::reactive::watch(
        move || pin_enabled.get(),
        move |on, old| {
            if old == Some(&true) && !*on {
                biometric_on.set(false);
                biometric::set_enabled(false);
            }
        },
    );

    // Persist toggle changes only when PIN is enabled
    day::reactive::watch(
        move || biometric_on.get(),
        move |val, old| {
            if old.is_some() {
                if pin::is_enabled() {
                    biometric::set_enabled(*val);
                } else if *val {
                    // PIN not enabled — revert and show alert
                    biometric_on.set(false);
                    notices.show(crate::components::notice::Notice {
                        title: res::str::notice_pin_title().format(),
                        message: res::str::notice_pin_message().format(),
                        kind: crate::components::notice::NoticeKind::Info,
                    });
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
                                .grow(Dimension::Both),
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


            )
        ).title("Безопасность"),
    ))
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}

fn logout_section(state: AppState) -> impl Piece {
    form((
        section(
            (button("Выйти из аккаунта")
                .action(move || crate::pages::login::auth::logout(state)),)
        ).title("Аккаунт"),
    ))
    .padding(Insets { top: 16.0, leading: 0.0, bottom: 20.0, trailing: 0.0 })
}

pub fn settings_body() -> impl Piece {
    system_settings()
}

#[cfg(debug_assertions)]
fn dev_settings(state: AppState) -> impl Piece {
    let copy_msg = Signal::new(String::new());
    let log_count = Signal::new(nslog::get_logs().lines().count());

    form((
        section(
            (
                label("Инструменты разработчика").font(Font::Headline).color(colors::WARNING),

                label("Версия: v0.1.0 (debug)"),
                button("Обновить токен")
                    .action(move || {
                        nslog::nslog("[Dev] Manual token refresh...");
                        let count_setter = log_count.setter();
                        std::thread::spawn(move || {
                            let msg: String = match crate::pages::login::auth::try_refresh_token() {
                                Some(_) => "Refresh OK".into(),
                                None => "Refresh FAILED".into(),
                            };
                            nslog::nslog(&format!("[Dev] {msg}"));
                            count_setter.set(nslog::get_logs().lines().count());
                        });
                    }),

                button("Очистить кэш")
                    .action(move || {
                        crate::app::data::cache::clear();
                        nslog::nslog("[Dev] Cache cleared");
                        log_count.set(nslog::get_logs().lines().count());
                    }),

                button("Перезагрузить данные")
                    .action(move || {
                        nslog::nslog("[Dev] Reloading data...");
                        crate::pages::diary::data::load_all(state);
                        log_count.set(nslog::get_logs().lines().count());
                    }),
            )
        ).title("Dev Tools"),

        section(
            (
                label("Логи приложения").font(Font::Headline).color(colors::INFO),

                row((
                    button("Скопировать логи")
                        .action(move || {
                            let txt = nslog::get_logs();
                            let lines = txt.lines().count();
                            day_part_clipboard::set_text(&txt);
                            crate::app::preferences::haptics::tick();
                            copy_msg.set(format!("✓ Скопировано ({lines} строк)"));
                        }),
                    button("Очистить")
                        .action(move || {
                            nslog::clear_logs();
                            copy_msg.set("Логи очищены".into());
                            log_count.set(0);
                        }),
                )).spacing(8.0),

                when(
                    move || !copy_msg.get().is_empty(),
                    move || label(move || copy_msg.get()).font(Font::Subheadline).color(colors::SUCCESS)
                ),

                label(move || format!("Записей в буфере: {}", log_count.get()))
                    .font(Font::Caption)
                    .color(colors::SECONDARY),
            )
        ).title("Логи"),
        input_diagnostics(),
    ))
    .padding(Insets { top: 16.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}

#[cfg(not(debug_assertions))]
fn dev_settings(_state: AppState) -> impl Piece {
    form((
        section(
            (
                label("О приложении").font(Font::Headline),
                label("Версия: v0.1.0"),
            )
        ).title("Информация"),
        input_diagnostics(),
    ))
    .padding(Insets { top: 16.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}

fn input_diagnostics() -> impl Piece {
    let result = Signal::new(String::new());
    section((
        label(res::str::autofill_diagnostics_hint()).font(Font::Subheadline).secondary(),
        button(res::str::autofill_diagnostics_copy()).action(move || {
            let diagnostics = nslog::autofill_diagnostics();
            if diagnostics.is_empty() {
                result.set(res::str::autofill_diagnostics_empty().format());
            } else {
                day_part_clipboard::set_text(&diagnostics);
                result.set(res::str::autofill_diagnostics_copied().format());
                crate::app::preferences::haptics::tick();
            }
        }),
        when(move || !result.get().is_empty(), move || {
            label(move || result.get()).font(Font::Caption).secondary().any()
        }),
    )).title(res::str::autofill_diagnostics_title())
}

#[cfg(all(test, feature = "mock"))]
mod pager_tests {
    use super::*;
    use day_spec::Event;

    fn check_swipe_settle(landed: usize) {
        day_core::uninstall_tree();
        let (mock, probe) = day_mock::MockToolkit::new();
        day_core::launch_with(mock, day_spec::WindowOptions::default(), || {
            AppState::scoped(|state| {
                state.settings_tab.set(0);
                render()
            }).any()
        });
        let find = |id: &str| {
            probe.state.borrow().widgets.iter()
                .find(|(_, widget)| day_core::id_of(day_spec::NodeId(widget.node)).as_deref() == Some(id))
                .map(|(handle, widget)| (day_mock::MockHandle(*handle), day_spec::NodeId(widget.node)))
                .unwrap_or_else(|| panic!("missing {id}"))
        };
        let (picker, picker_node) = find("settings_tabs");
        let (_, pager_node) = find("settings_pager");
        let state = AppState::get_main().unwrap();
        assert!(probe.widget(picker).enabled);
        probe.emit(pager_node, Event::Custom { tag: "", num: 0.0, text: "begin 1".into() });
        assert!(!probe.widget(picker).enabled, "native picker must reject segment taps throughout a swipe");
        assert_eq!(state.settings_tab.get(), 0);
        probe.emit(pager_node, Event::Custom { tag: "", num: 0.0, text: format!("settle 1 {landed}") });
        assert_eq!(state.settings_tab.get(), landed, "completed and cancelled swipes must report their landed slot");
        assert!(probe.widget(picker).enabled, "picker must become usable again after settling");
        probe.emit(picker_node, Event::SelectionChanged(3));
        assert_eq!(state.settings_tab.get(), 3, "the next segment selection must be accepted");
        // The picker change advances the presentation epoch; an obsolete native completion
        // must not take the selection back to the previously settled page.
        probe.emit(pager_node, Event::Custom { tag: "", num: 0.0, text: format!("settle 1 {landed}") });
        assert_eq!(state.settings_tab.get(), 3);
        day_core::uninstall_tree();
    }

    #[test]
    fn settings_picker_reenabled_after_completed_swipe() { check_swipe_settle(1); }

    #[test]
    fn settings_picker_reenabled_after_cancelled_swipe() { check_swipe_settle(0); }
}
