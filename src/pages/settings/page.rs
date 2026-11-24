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
    let pin_enabled = Signal::new(crate::pages::pin::security::is_enabled());

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

    scroll(
        column((
            when(move || state.is_authenticated.get(), move || profile_section(state)),
            system_settings().padding(Insets {
                top: 0.0,
                leading: 0.0,
                bottom: 16.0,
                trailing: 0.0,
            }),
            appearance_section(state),
            haptics_section(state),
            pin_section(state, pin_enabled),
            biometric_section(state, pin_enabled),
            dev_settings(state),
            when(move || state.is_authenticated.get(), move || logout_section(state)),
            spacer().frame(0.0, 16.0),
        ))
        .spacing(0.0)
        .grow(Dimension::Both)
    )
    .grow(Dimension::Both)
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
                    move || label(move || format!("{}: {}", res::str::settings_class().format(), state.class_label.get()))
                        .font(Font::Body).secondary().align(TextAlign::Center),
                ),
                crate::components::child_context::child_switcher(state),
            )
        ).title(res::str::settings_profile()),
    ))
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}

fn appearance_section(state: AppState) -> impl Piece {
    let accent = Signal::new(state.accent_color.get());
    day::reactive::watch(
        move || accent.get(),
        move |value, old| {
            if old.is_some() && old != Some(value) {
                accent_commit(state, *value);
            }
        },
    );
    form((section((labeled(
        res::str::settings_accent_color(),
        picker(
            accent,
            vec![
                (colors::BLUE, res::str::settings_color_blue().format()),
                (colors::GREEN, res::str::settings_color_green().format()),
                (colors::PURPLE, res::str::settings_color_purple().format()),
                (colors::ORANGE, res::str::settings_color_orange().format()),
                (colors::RED, res::str::settings_color_red().format()),
            ],
        ),
    ),))
    .title(res::str::settings_colors()),))
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}

fn accent_apply(state: AppState, hex: u32) {
    colors::set_accent(hex);
    state.accent_color.set(hex);
    colors::apply_platform_tint(hex);
}

fn accent_commit(state: AppState, hex: u32) {
    day::prefs::set("app.accent_color", &hex.to_string());
    accent_apply(state, hex);
    haptics::tick();
}

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
                    label(res::str::settings_haptics())
                        .font(Font::Body)
                        .grow(Dimension::Both),
                    toggle(haptics_on),
                ))
                .spacing(8.0),
                label(res::str::settings_haptics_hint())
                    .font(Font::Caption)
                    .secondary(),
            )
        ).title(res::str::settings_feedback()),
    ))
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}

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

fn pin_section(state: AppState, pin_enabled: Signal<bool>) -> impl Piece {
    column((
        form((
            section(
                (
                    row((
                        label(res::str::settings_pin_login())
                            .font(Font::Body)
                            .grow(Dimension::Both),
                        toggle(pin_enabled),
                    ))
                    .spacing(8.0),

                    when(
                        move || crate::pages::pin::security::is_enabled(),
                        move || row((
                            label(res::str::settings_pin_active())
                                .font(Font::Caption)
                                .secondary()
                                .grow(Dimension::Both),
                            button(res::str::settings_change())
                                .action(move || {
                                    crate::app::preferences::haptics::tick();
                                    crate::pages::pin::open_setup(state);
                                }),
                        )),
                    ),
                )
            ).title(res::str::settings_lock()),
        )),
    ))
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}

fn biometric_section(state: AppState, pin_enabled: Signal<bool>) -> impl Piece {
    let available = biometric::is_available();
    let biometric_on = Signal::new(biometric::is_enabled());
    let notices = crate::components::notice::presenter(state);

    day::reactive::watch(
        move || pin_enabled.get(),
        move |on, old| {
            if old == Some(&true) && !*on {
                biometric_on.set(false);
                biometric::set_enabled(false);
            }
        },
    );

    day::reactive::watch(
        move || biometric_on.get(),
        move |val, old| {
            if old.is_some() {
                if pin::is_enabled() {
                    biometric::set_enabled(*val);
                } else if *val {
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
                            label(res::str::settings_biometric_login())
                                .font(Font::Body)
                                .grow(Dimension::Both),
                            toggle(biometric_on),
                        ))
                        .spacing(8.0)
                    },
                ),
                when(
                    move || !available && pin::is_enabled(),
                    || label(res::str::settings_biometric_unavailable())
                        .font(Font::Body)
                        .secondary(),
                ),
                when(
                    move || !pin::is_enabled() && available,
                    || label(res::str::settings_enable_pin_first())
                        .font(Font::Caption)
                        .secondary()
                        .color(colors::WARNING),
                ),


            )
        ).title(res::str::settings_security()),
    ))
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}

fn logout_section(state: AppState) -> impl Piece {
    form((
        section(
            (button(res::str::settings_logout())
                .action(move || crate::pages::login::auth::logout(state)),)
        ).title(res::str::settings_account()),
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
                label(res::str::settings_dev_tools()).font(Font::Headline).color(colors::WARNING),

                label(move || format!("{}: v0.1.0 (debug)", res::str::settings_version().format())),
                button(res::str::settings_refresh_token())
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

                button(res::str::settings_clear_cache())
                    .action(move || {
                        crate::app::data::cache::clear();
                        nslog::nslog("[Dev] Cache cleared");
                        log_count.set(nslog::get_logs().lines().count());
                    }),

                button(res::str::settings_reload_data())
                    .action(move || {
                        nslog::nslog("[Dev] Reloading data...");
                        crate::pages::diary::data::load_all(state);
                        log_count.set(nslog::get_logs().lines().count());
                    }),
            )
        ).title("Dev Tools"),

        section(
            (
                label(res::str::settings_app_logs()).font(Font::Headline).color(colors::INFO),

                row((
                    button(res::str::settings_copy_logs())
                        .action(move || {
                            let txt = nslog::get_logs();
                            let lines = txt.lines().count();
                            day_part_clipboard::set_text(&txt);
                            crate::app::preferences::haptics::tick();
                            copy_msg.set(format!("✓ {} ({lines})", res::str::settings_logs_copied().format()));
                        }),
                    button(res::str::settings_clear())
                        .action(move || {
                            nslog::clear_logs();
                            copy_msg.set(res::str::settings_logs_cleared().format());
                            log_count.set(0);
                        }),
                )).spacing(8.0),

                when(
                    move || !copy_msg.get().is_empty(),
                    move || label(move || copy_msg.get()).font(Font::Subheadline).color(colors::SUCCESS)
                ),

                label(move || format!("{}: {}", res::str::settings_log_entries().format(), log_count.get()))
                    .font(Font::Caption)
                    .color(colors::SECONDARY),
            )
        ).title(res::str::settings_logs()),
        input_diagnostics(),
    ))
    .padding(Insets { top: 16.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}

#[cfg(not(debug_assertions))]
fn dev_settings(_state: AppState) -> impl Piece {
    form((
        section(
            (
                label(res::str::settings_about()).font(Font::Headline),
                label(move || format!("{}: v0.1.0", res::str::settings_version().format())),
            )
        ).title(res::str::settings_information()),
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
