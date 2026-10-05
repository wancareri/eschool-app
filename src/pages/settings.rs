use crate::app::AppState;
use crate::features;
use crate::widgets;
use crate::res;
use crate::shared::{biometric, colors, haptics, nslog, pin};
use day::prelude::*;
use day_piece_texteditor::text_editor;

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
    // The strip's pill rides this every scroll event — page offset / page width,
    // so it moves exactly as far as the finger, not one segment at a time.
    let strip_pos: Signal<f64> = Signal::new(current_tab.get() as f64);
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
    ).otherwise(move || {
        // Per-build clone: the outer closure is Fn (it may rebuild), while the
        // inner `on_scroll` takes its flag by move — a Signal would be Copy.
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

        tab_strip(state, current_tab, strip_pos),

        // Four-page native pager: UIKit owns the finger physics (paging
        // snap with its velocity throw), every page keeps its own vertical
        // scroll, and scroll events keep settings_tab in step with the strip.
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
            strip_pos.set((p.x / pager_w).clamp(0.0, 3.0));
            let idx = ((p.x / pager_w).round() as usize).min(3);
            let settled = (p.x - idx as f64 * pager_w).abs() < 0.5;
            if let Some((want, tries)) = restore.get() {
                // Restoring: never commit from here. Landing on the built tab ends the
                // restore; settling on the wrong edge re-arms the native jump (bounded,
                // so a pager that simply refuses can't lock the user out).
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
            last_scroll_idx.set(Some(idx));
            // Only a settled offset (on a page edge) commits — mid-flight
            // offsets must not step the picker highlight.
            if settled && idx != current_tab.get() {
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
    .any()
    })
}

/// Sliding tab strip: the pill tracks the pager offset on every scroll event,
/// so it moves exactly as far as the finger — a native UISegmentedControl can
/// only hop between segments. Labels flip over as the pill passes their slot,
/// and a tap commits the tab; the pill then rides the pager's own animation in
/// on the didScroll events it produces.
fn tab_strip(
    state: AppState,
    current_tab: Signal<usize>,
    strip_pos: Signal<f64>,
) -> impl Piece {
    let pager_w = crate::pages::diary::get_screen_width();
    let track_w = pager_w - 40.0;
    let slot_w = track_w / 4.0;

    let track = capsule()
        .fill(move || {
            if day::dark_mode() {
                Color::rgba(0.22, 0.22, 0.24, 1.0)
            } else {
                Color::rgba(0.90, 0.90, 0.92, 1.0)
            }
        })
        .frame(track_w, 32.0);

    // zstack centers children: the pill starts at (track_w - pill_w)/2, so the
    // translation back from that to slot `pos` is slot_w * (pos - 1.5). The pill
    // is a capsule too — its only way to sit inside the strongly rounded track
    // ends at the edge slots without poking past the curve.
    let pill = capsule()
        .fill(move || Color::hex(state.accent_color.get()))
        .frame(slot_w - 2.0, 28.0)
        .translation(
            move || slot_w * (strip_pos.get().clamp(0.0, 3.0) - 1.5),
            0.0,
        );

    let tab = |i: usize, title: &'static str| {
        let tab = current_tab;
        let pos = strip_pos;
        // The title is centred in the slot and spans roughly `g` of it
        // (Subheadline ≈ 7.5 pt per glyph) — that maps the pill's slot-local
        // edges onto individual glyphs. zstack outside, width on the zstack:
        // a stretched label sits flush-left on UIKit, the overlay centers it.
        let n = title.chars().count();
        let g = ((n as f64 * 7.5) / slot_w).clamp(0.1, 1.0);
        let cw = g / n as f64;
        let half = (slot_w - 2.0) / (2.0 * slot_w);
        let letters: Vec<AnyPiece> = title
            .chars()
            .enumerate()
            .map(|(k, ch)| {
                let x0 = 0.5 - g / 2.0 + k as f64 * cw;
                label(ch.to_string())
                    .font(Font::Subheadline)
                    .color(move || {
                        // Coverage of this glyph by the sliding pill: white where
                        // the pill has already crossed, the secondary tone ahead
                        // of it, blended inside the boundary glyph.
                        let d = pos.get() - i as f64;
                        let (pl, pr) = (0.5 + d - half, 0.5 + d + half);
                        let cov = (pr.min(x0 + cw) - pl.max(x0)).max(0.0) / cw;
                        let t = cov.clamp(0.0, 1.0);
                        let s = colors::SECONDARY;
                        Color::rgba(
                            s.r + (1.0 - s.r) * t,
                            s.g + (1.0 - s.g) * t,
                            s.b + (1.0 - s.b) * t,
                            1.0,
                        )
                    })
                    .any()
            })
            .collect();
        zstack((row(PieceVec(letters)),))
            .width(slot_w)
            .on_tap(move || tab.set(i))
            .a11y(move |b| b.role(Role::Button).label(title))
            .any()
    };

    zstack((
        track,
        pill,
        row((
            tab(0, "Основные"),
            tab(1, "Вид"),
            tab(2, "Защита"),
            tab(3, "Dev"),
        ))
        .frame(track_w, 32.0),
    ))
    .frame(track_w, 32.0)
    .padding(Insets {
        top: 8.0,
        leading: 20.0,
        bottom: 16.0,
        trailing: 20.0,
    })
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
            haptics_section(state),
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
                haptics::tick();
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
                haptics::tick();
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

