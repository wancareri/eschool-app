use crate::app::AppState;
use crate::features;
use crate::widgets;
use crate::res;
use crate::shared::{biometric, colors, haptics, nslog, pin};
use day::prelude::*;

pub fn render() -> impl Piece {
    let state = AppState::ambient();
    let current_tab = state.settings_tab;
    let pager_w = crate::pages::diary::get_screen_width();

    // The pager's own report: a tab write that came FROM a scroll event is
    // skipped by the scroll-back watch below (no scroll → scroll ping-pong).
    let last_scroll_idx: Signal<Option<usize>> = Signal::new(None);
    // Picker segment: the strip animates there natively.
    let tap_target: Signal<Option<ScrollTarget>> = Signal::new(None);
    // First mount lands on the persisted tab without sliding through the rest.
    let initial_target: Signal<Option<ScrollTarget>> = Signal::new(Some(ScrollTarget::Offset(
        Point::new(pager_w * current_tab.get() as f64, 0.0),
    )));

    // The tab this pager was BUILT for. It mounts at offset 0 regardless, so until
    // the restore has actually landed, scroll reports must not be allowed to commit
    // a different tab — that write was the bounce to «Основные». Tab 0 IS the mount
    // offset: nothing to restore there, so start unguarded.
    let built_tab = current_tab.get();
    let restore: Signal<Option<(usize, u32)>> =
        Signal::new(if built_tab == 0 { None } else { Some((built_tab, 0)) });

    // The mount jump can be swallowed: the scroll applies before the pager's
    // contentSize exists, scrollRectToVisible no-ops, no event fires — the content
    // stays on page 0 while the strip shows the built tab. Re-send the target via
    // the INSTANT jump signal (an animated target would visibly slide the page in);
    // an already-landed pager ignores the redundant jumps, and the flag stops the
    // timers once `restore` completes. Setters are Send (a plain Signal is not).
    let restore_done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    if built_tab != 0 {
        for ms in [50u32, 160, 360, 700] {
            let jump = initial_target.setter();
            let done = restore_done.clone();
            day::reactive::on_main_delayed(ms, move || {
                if !done.load(std::sync::atomic::Ordering::Relaxed) {
                    jump.set(Some(ScrollTarget::Offset(Point::new(
                        pager_w * built_tab as f64,
                        0.0,
                    ))));
                }
            });
        }
    }

    // A tab change from OUTSIDE the pager (picker segment) scrolls
    // the strip there. The first callback is the mount one — the strip is
    // already positioned by scroll_jump — so only later changes act.
    {
        let last = last_scroll_idx;
        let tap = tap_target;
        day::reactive::watch(
            move || current_tab.get(),
            move |&t, old| {
                if old.is_none() || last.get() == Some(t) {
                    return;
                }
                tap.set(Some(ScrollTarget::Offset(Point::new(
                    pager_w * t as f64,
                    0.0,
                ))));
            },
        );
    }

    let pin_enabled = Signal::new(crate::shared::pin::is_enabled());

    // When toggle turns ON -> activate pin setup at window level (hiding bottom bar)
    {
        day::reactive::watch(
            move || pin_enabled.get(),
            move |on, old| {
                if old == Some(&false) && *on {
                    crate::pages::pin_lock::open_setup(state);
                }
            },
        );
    }
    // When toggle turns OFF -> delete PIN and disable biometric
    day::reactive::watch(
        move || pin_enabled.get(),
        move |on, old| {
            if old == Some(&true) && !*on {
                crate::shared::pin::delete_pin();
                crate::shared::biometric::set_enabled(false);
                crate::shared::nslog::nslog("[PIN] Disabled, Face ID also disabled");
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
                    pe.set(crate::shared::pin::is_enabled());
                }
            },
        );
    }

    let rd = restore_done.clone();
    zstack((
        scroll(column((
            column((
                label(move || res::str::settings_title().format())
                    .font(Font::LargeTitle).align(TextAlign::Center),
                label("Настройки").font(Font::Subheadline).secondary().align(TextAlign::Center),
            ))
            .spacing(6.0)
            .padding(Insets { top: 16.0, leading: 20.0, bottom: 8.0, trailing: 20.0 }),

            picker(
                vec![
                    "Основные",
                    "Вид",
                    "Защита",
                    "Dev",
                ],
                current_tab,
            )
            .segmented()
            .padding(Insets {
                top: 4.0,
                leading: 20.0,
                bottom: 14.0,
                trailing: 20.0,
            }),

            // Four-page native pager
            scroll(row((
                tab_body(state, 0, pin_enabled).width(pager_w),
                tab_body(state, 1, pin_enabled).width(pager_w),
                tab_body(state, 2, pin_enabled).width(pager_w),
                tab_body(state, 3, pin_enabled).width(pager_w),
            )))
            .horizontal()
            .paging(true)
            .scroll_target(tap_target)
            .scroll_jump(initial_target)
            .on_scroll(move |p| {
                let idx = ((p.x + pager_w * 0.5) / pager_w).floor() as usize;
                let idx = idx.min(3);
                let settled = (p.x - idx as f64 * pager_w).abs() < 0.5;
                if let Some((want, tries)) = restore.get() {
                    if settled {
                        if idx == want {
                            restore.set(None);
                            rd.store(true, std::sync::atomic::Ordering::Relaxed);
                            last_scroll_idx.set(Some(idx));
                        } else if tries < 5 {
                            initial_target.set(Some(ScrollTarget::Offset(Point::new(
                                pager_w * want as f64,
                                0.0,
                            ))));
                            restore.set(Some((want, tries + 1)));
                        } else {
                            restore.set(None);
                            rd.store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                    return;
                }
                if settled && current_tab.get() != idx {
                    crate::shared::haptics::tick();
                    last_scroll_idx.set(Some(idx));
                    current_tab.set(idx);
                }
            })
            .width(pager_w)
            .grow()
        )))
        .grow(),

        // Sticky status indicator in top-left corner
        widgets::conn_status::page_overlay(),
    ))
    .align(Alignment::TopLeading)
    .grow()
}

fn tab_body(state: AppState, tab: usize, pin_enabled: Signal<bool>) -> impl Piece {
    match tab {
        0 => column((
            when(move || state.is_authenticated.get(), move || profile_section(state)),
            when(move || state.is_authenticated.get(), move || logout_section(state)),
            spacer().frame(0.0, 90.0),
        ))
        .spacing(0.0)
        .grow()
        .any(),
        1 => column((
            appearance_section(state),
            haptics_section(state),
            system_settings(),
            spacer().frame(0.0, 90.0),
        ))
        .spacing(0.0)
        .grow()
        .any(),
        2 => column((
            pin_section(state, pin_enabled),
            biometric_section(state, pin_enabled),
            spacer().frame(0.0, 90.0),
        ))
        .spacing(0.0)
        .grow()
        .any(),
        _ => column((
            dev_settings(state),
            spacer().frame(0.0, 90.0),
        ))
        .spacing(0.0)
        .grow()
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
    #[cfg(target_os = "ios")]
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
                        .grow(),
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
                            .grow(),
                        toggle(pin_enabled),
                    ))
                    .spacing(8.0),

                    when(
                        move || crate::shared::pin::is_enabled(),
                        move || row((
                            label("PIN-код активен")
                                .font(Font::Caption)
                                .secondary()
                                .grow(),
                            button("Сменить")
                                .action(move || {
                                    crate::shared::haptics::tick();
                                    crate::pages::pin_lock::open_setup(state);
                                }),
                        )),
                    ),
                )
            ).title("Блокировка"),
        )),
    ))
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}

fn biometric_section(_state: AppState, pin_enabled: Signal<bool>) -> impl Piece {
    let available = biometric::is_available();
    let biometric_on = Signal::new(biometric::is_enabled());
    let show_alert = Signal::new(false);

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
    let copy_msg = Signal::new(String::new());
    let log_count = Signal::new(nslog::get_logs().lines().count());

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
                        log_count.set(nslog::get_logs().lines().count());
                    }),

                button("Обновить токен")
                    .action(move || {
                        nslog::nslog("[Dev] Manual token refresh...");
                        let count_setter = log_count.setter();
                        std::thread::spawn(move || {
                            let msg: String = match features::auth::try_refresh_token() {
                                Some(_) => "Refresh OK".into(),
                                None => "Refresh FAILED".into(),
                            };
                            nslog::nslog(&format!("[Dev] {msg}"));
                            count_setter.set(nslog::get_logs().lines().count());
                        });
                    }),

                button("Очистить кэш")
                    .action(move || {
                        day::prefs::set("diary.cache", "");
                        nslog::nslog("[Dev] Cache cleared");
                        log_count.set(nslog::get_logs().lines().count());
                    }),

                button("Перезагрузить данные")
                    .action(move || {
                        nslog::nslog("[Dev] Reloading data...");
                        features::diary::load_all(state);
                        log_count.set(nslog::get_logs().lines().count());
                    }),

                button("Показать ID")
                    .action(move || {
                        let (sid, cid, pid) = features::auth::get_stored_ids();
                        nslog::nslog(&format!("[Dev] school={sid} class={cid} profile={pid}"));
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
                            crate::shared::haptics::tick();
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
    ))
    .padding(Insets { top: 16.0, leading: 0.0, bottom: 16.0, trailing: 0.0 })
}

