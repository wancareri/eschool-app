use crate::app::AppState;
use crate::features;
use crate::widgets;
use crate::res;
use crate::shared::{biometric, colors, nslog, pin};
use day::prelude::*;
use day_piece_texteditor::text_editor;
use std::sync::atomic::{AtomicU64, Ordering};

// Retarget-safe switch: a new switch (picker tap or drag commit) bumps the
// generation, and only the still-current landing commits displayed_tab —
// requests are never dropped while an animation is in flight.
static SWITCH_GEN: AtomicU64 = AtomicU64::new(0);

pub fn render() -> impl Piece {
    let state = AppState::ambient();
    let current_tab = state.settings_tab;

    day::reactive::watch(
        move || current_tab.get(),
        move |&t, _| {
            day::prefs::set("app.settings_tab", &t.to_string());
        },
    );
    let displayed_tab = state.settings_displayed;
    let tab_dx = state.settings_dx;
    let pager_w = crate::pages::diary::get_screen_width();

    // A watch cb runs inside the drain, where with_animation defers past its own
    // scope and the write lands instantly (day-core anim.rs edge case — a
    // teleport). Hop to a clean main-queue turn instead; start_switch re-derives
    // the shared pager signals there via get_main (Signals are !Send, so nothing
    // crosses the hop itself).
    day::reactive::watch(
        move || current_tab.get(),
        move |&t, _| {
            day::reactive::on_main(move || start_switch(t, pager_w));
        },
    );
    day::reactive::watch(
        move || displayed_tab.get(),
        move |&shown, _| {
            let want = current_tab.get();
            if want != shown {
                day::reactive::on_main(move || start_switch(want, pager_w));
            }
        },
    );
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

    let axis: Signal<Option<bool>> = Signal::new(None);
    let drag_base = Signal::new(0.0f64);
    let drag_handler = move |drag: Drag| {
        let dx = drag.translation.x;
        let dy = drag.translation.y;
        match drag.phase {
            DragPhase::Began => {
                axis.set(None);
                // The finger takes the pager over: any in-flight switch landing
                // is invalidated and the drag continues from the current offset.
                SWITCH_GEN.fetch_add(1, Ordering::Relaxed);
                drag_base.set(tab_dx.get());
            }
            DragPhase::Changed => {
                let mut horiz = axis.get();
                if horiz.is_none() && (dx.abs() > 10.0 || dy.abs() > 10.0) {
                    horiz = Some(dx.abs() >= dy.abs() * 0.7);
                    axis.set(horiz);
                }
                if horiz == Some(false) && dx.abs() > 24.0 && dx.abs() > dy.abs() * 1.2 {
                    horiz = Some(true);
                    axis.set(horiz);
                }
                if horiz == Some(true) {
                    let shown = displayed_tab.get();
                    let mut disp = dx;
                    if shown == 0 && disp > 0.0 {
                        disp *= 0.3;
                    }
                    if shown == 3 && disp < 0.0 {
                        disp *= 0.3;
                    }
                    tab_dx.set(drag_base.get() + disp);
                }
            }
            DragPhase::Ended => {
                let mut was_horiz = axis.get() == Some(true);
                if !was_horiz
                    && axis.get().is_none()
                    && dx.abs() >= 40.0
                    && dx.abs() > dy.abs()
                {
                    was_horiz = true;
                    tab_dx.set(drag_base.get() + dx);
                }
                axis.set(None);
                let disp = tab_dx.get() - drag_base.get();
                if was_horiz && disp.abs() >= 40.0 {
                    let shown = displayed_tab.get();
                    if disp < 0.0 && shown < 3 {
                        current_tab.set(shown + 1);
                        return;
                    }
                    if disp > 0.0 && shown > 0 {
                        current_tab.set(shown - 1);
                        return;
                    }
                }
                // No commit: the Began cancel killed any in-flight landing, so
                // slide home and drop the picker back to the displayed tab —
                // otherwise current_tab and displayed_tab would diverge and a
                // re-tap of the highlighted segment would do nothing.
                with_animation(AnimSpec::ease_out(180), move || {
                    tab_dx.set(0.0);
                });
                let shown = displayed_tab.get();
                if current_tab.get() != shown {
                    current_tab.set(shown);
                }
            }
        }
    };

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

        // Full four-page carousel: every page carries its own scroll, so a
        // short page ends with its content instead of inheriting the longest
        // page's length from one shared scroll. Idle pages sit at ±w/±2w,
        // tab_dx drags them like diary week cards, and a switch keeps the
        // old page visually continuous until the landing batch.
        zstack((
            each(
                items(move || vec![0usize, 1, 2, 3], |t| *t),
                move |slot| {
                    let t = slot.with(|t: &usize| *t);
                    scroll(tab_body(state, t, pin_enabled))
                        .translation(
                            move || {
                                tab_dx.get()
                                    + (t as f64 - displayed_tab.get() as f64) * pager_w
                            },
                            0.0,
                        )
                        .width(pager_w)
                        .grow_h()
                },
            ),
        ))
        .align(Alignment::TopLeading)
        .width(pager_w)
        .grow()
    ))
    .grow()
    .on_drag(drag_handler),

    // Sticky status indicator in top-left corner
    widgets::conn_status::render()
        .padding(Insets { top: 16.0, leading: 16.0, bottom: 0.0, trailing: 0.0 }),
    ))
    .align(Alignment::TopLeading)
    .grow()
    .any())
}

fn start_switch(target: usize, w: f64) {
    // Reached from an on_main hop: the shared signals are read here, on the
    // main thread, so the with_animation writes below drain synchronously with
    // the intent still ambient.
    let Some(state) = AppState::get_main() else {
        return;
    };
    let tab_dx = state.settings_dx;
    let shown = state.settings_displayed.get();
    if target == shown {
        // Retarget onto the page already displayed (e.g. the picker tapped
        // back mid-flight): cancel and slide home.
        SWITCH_GEN.fetch_add(1, Ordering::Relaxed);
        with_animation(AnimSpec::ease_out(180), move || {
            tab_dx.set(0.0);
        });
        return;
    }
    let dir = if target > shown { 1.0 } else { -1.0 };
    let dist = (target as i32 - shown as i32).abs() as f64;
    let my_gen = SWITCH_GEN.fetch_add(1, Ordering::Relaxed) + 1;
    with_animation(AnimSpec::ease_out(280), move || {
        // The target page sits `dist` slots away, so it lands at 0 only after
        // tab_dx covers the whole gap (a picker jump of 0 → 2 is two pages).
        tab_dx.set(-dir * dist * w);
    });
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(280));
        day::reactive::on_main(move || {
            if SWITCH_GEN.load(Ordering::Relaxed) != my_gen {
                return;
            }
            let Some(state) = AppState::get_main() else {
                return;
            };
            day::reactive::batch(|| {
                state.settings_dx.set(0.0);
                state.settings_displayed.set(target);
            });
        });
    });
}

fn tab_body(state: AppState, tab: usize, pin_enabled: Signal<bool>) -> impl Piece {
    match tab {
        0 => column((
            when(move || state.is_authenticated.get(), move || profile_section(state)),
            when(move || state.is_authenticated.get(), move || logout_section(state)),
        ))
        .spacing(0.0)
        .grow()
        .any(),
        1 => column((
            appearance_section(state),
            system_settings(),
        ))
        .spacing(0.0)
        .grow()
        .any(),
        2 => column((
            pin_section(state, pin_enabled),
            biometric_section(state),
        ))
        .spacing(0.0)
        .grow()
        .any(),
        _ => dev_settings(state).any(),
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
        state.current_section.set(crate::Section::Settings);
        state.settings_tab.set(1);
        day::prefs::set("app.section", "settings");
        day::prefs::set("app.settings_tab", "1");
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

    column((
        spacer().grow(),

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
        .padding(Insets { top: 20.0, leading: 0.0, bottom: 8.0, trailing: 0.0 })
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

        spacer().grow(),
    ))
    .spacing(8.0)
    .align(HAlign::Center)
    .padding(Insets {
        top: 24.0,
        leading: 24.0,
        bottom: 24.0,
        trailing: 24.0,
    })
    .grow()
    .any()
}

fn setup_dot(index: usize, input: Signal<String>) -> impl Piece {
    let accent = AppState::ambient();
    when(
        move || input.get().len() > index,
        move || circle().fill(move || Color::hex(accent.accent_color.get())).frame(16.0, 16.0).any(),
    )
    .otherwise(move || circle().stroke(colors::SECONDARY, 1.5).frame(16.0, 16.0).any())
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
            spacer().frame(70.0, 44.0).any()
        } else if k == "⌫" {
            zstack((
                label("⌫")
                    .font(Font::LargeTitle)
                    .secondary(),
            ))
            .frame(70.0, 44.0)
            .on_tap(move || {
                let mut v = state.get();
                if !v.is_empty() {
                    v.pop();
                    state.set(v);
                    error.set("".into());
                }
            })
            .a11y(|b| b.role(Role::Button))
            .id(format!("sk-{k2}"))
            .any()
        } else {
            let accent = AppState::ambient();
            zstack((
                label(k.clone())
                    .font(Font::LargeTitle)
                    .color(move || Color::hex(accent.accent_color.get())),
            ))
            .frame(70.0, 44.0)
            .on_tap(move || {
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
            .a11y(|b| b.role(Role::Button))
            .id(format!("sk-{k2}"))
            .any()
        }
    }

    column((
        row((
            setup_key(input.clone(), "1".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
            setup_key(input.clone(), "2".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
            setup_key(input.clone(), "3".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
        )).spacing(24.0).align(VAlign::Center),
        row((
            setup_key(input.clone(), "4".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
            setup_key(input.clone(), "5".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
            setup_key(input.clone(), "6".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
        )).spacing(24.0).align(VAlign::Center),
        row((
            setup_key(input.clone(), "7".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
            setup_key(input.clone(), "8".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
            setup_key(input.clone(), "9".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
        )).spacing(24.0).align(VAlign::Center),
        row((
            setup_key(input.clone(), "".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
            setup_key(input.clone(), "0".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
            setup_key(input.clone(), "⌫".into(), confirm.clone(), step.clone(), error.clone(), setup_mode.clone()),
        )).spacing(24.0).align(VAlign::Center),
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

