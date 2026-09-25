use day::prelude::*;
use crate::app::AppState;
use crate::shared::{locale, nslog};
use crate::pages;
use crate::res;

pub fn window() -> day::WindowOptions {
    day::WindowOptions {
        locales: Some((res::locales::DEFAULT, res::locales::CATALOG)),
        title_fn: Some(|| res::str::app_title().format()),
        size: day::prelude::Size::new(400.0, 720.0),
        min_size: Some(day::prelude::Size::new(320.0, 480.0)),
        ..Default::default()
    }
}

pub fn root() -> impl Piece {
    nslog::nslog("[init] Eschool App starting");
    let has_locale = day::prefs::get("app.locale")
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    if !has_locale {
        let sys = locale::system_locale();
        day::prefs::set("app.locale", sys);
        nslog::nslog(&format!("[init] First launch — system locale: {sys}"));
    }
    day_piece_settings::apply_startup("app.theme", "app.locale");
    crate::shared::colors::init_accent();
    day::register_preferences(pages::settings::settings_body);
    day::register_new_window(|| window_shell(false));

    // Start background token refresh if authenticated
    let has_token = crate::shared::secure::load("auth.token").is_some();
    if has_token {
        crate::features::auth::start_background_refresh();
        crate::features::grade_checker::start_grade_checker();
        #[cfg(target_os = "ios")]
        crate::shared::notifications::ios::request_permission();
    }

    window_shell(true)
}

fn window_shell(primary: bool) -> impl Piece {
    AppState::scoped(move |state| {
        state.register_main();
        day::window_title(move || res::str::app_title().format());

        // Auto-lock: save background timestamp periodically + re-lock after 5 min
        #[cfg(target_os = "ios")]
        {
            use std::sync::atomic::{AtomicBool, Ordering};
            static BG_WATCHER_STARTED: AtomicBool = AtomicBool::new(false);
            if !BG_WATCHER_STARTED.swap(true, Ordering::Relaxed) {
                nslog::nslog("[App] Starting background timestamp saver (30s)");
                let lock_sig = state.pin_lock_active.setter();
                let auth_sig = state.is_authenticated.setter();
                std::thread::spawn(move || {
                    loop {
                        std::thread::sleep(std::time::Duration::from_secs(30));
                        crate::shared::pin::save_last_background();
                        if crate::shared::pin::is_enabled()
                            && crate::shared::pin::should_auto_lock()
                        {
                            nslog::nslog("[PIN] Auto-lock triggered");
                            auth_sig.set(false);
                            lock_sig.set(true);
                        }
                    }
                });
            }
        }

        // Reactive: trigger load_all when is_authenticated transitions to true
        day::reactive::watch(
            move || state.is_authenticated.get(),
            move |&auth, old| {
                if auth && old != Some(&true) {
                    nslog::nslog("[App] is_authenticated changed true, triggering load_all");
                    crate::features::diary::load_all(state);
                }
            },
        );

        // Watch biometric signal — set by Setter from Face ID reply block
        day::reactive::watch(
            move || state.biometric_ok.get(),
            move |&ok, old| {
                if ok && old != Some(&true) {
                    nslog::nslog("[App] biometric_ok became true, setting is_authenticated");
                    state.pin_lock_active.set(false);
                    state.is_authenticated.set(true);
                }
            },
        );

        // Biometric lock: if token exists + biometric enabled, prompt Face ID on startup
        // Works for both pure biometric lock and PIN+biometric combo
        if !state.is_authenticated.get()
            && crate::shared::secure::load("auth.token").is_some()
            && crate::shared::biometric::is_available()
            && crate::shared::biometric::is_enabled()
        {
            nslog::nslog("[App] Biometric lock: prompting Face ID on startup");
            crate::shared::biometric::authenticate_async(state);
        }

        build_nav(primary)
    })
}

/// iOS: Telegram-style edge scrims — a soft ~20% black gradient at the very top
/// (80pt) and bottom (64pt), so full-bleed content stays readable where it runs
/// into the notch / home-indicator safe areas. Must sit on a VIEW-backed
/// container: `when` builds a layout-only node with no UIView, where
/// `with_native` returns None and the tweak would silently do nothing.
#[cfg(target_os = "ios")]
fn edge_scrims(piece: impl Decorate) -> impl Piece {
    use day_uikit::UiKitExt;
    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2_core_foundation::{CGPoint, CGRect, CGSize};
    use objc2_core_graphics::CGColor;
    use objc2_foundation::{NSArray, NSString};
    use objc2_quartz_core::{CAAutoresizingMask, CAGradientLayer};
    use objc2_ui_kit::UIScreen;

    piece.uikit(|view, _class, mtm| {
        const NAME: &str = "eschool.edge_scrim";

        let root = view.layer();
        // A patched piece may re-run the tweak on the same mounted view —
        // don't stack a second pair of scrims on top of the first.
        // SAFETY: tweaks run once at mount on the main thread; nothing mutates
        // the layer tree concurrently.
        if let Some(subs) = unsafe { root.sublayers() } {
            let n = subs.count();
            for i in 0..n {
                if subs
                    .objectAtIndex(i)
                    .name()
                    .map(|found| found.to_string() == NAME)
                    .unwrap_or(false)
                {
                    return;
                }
            }
        }

        // Bounds are zero before the first layout pass; fall back to the screen
        // so the gradients land somewhere sane. Autoresizing is armed only when
        // a real size was known — arming it on a zero-size frame pins the
        // bottom scrim to y=0, and CA never recomputes the margins afterwards.
        let bounds = view.bounds();
        let laid_out = bounds.size.width >= 1.0 && bounds.size.height >= 1.0;
        let (w, h) = if laid_out {
            (bounds.size.width, bounds.size.height)
        } else {
            #[allow(deprecated)]
            let screen = UIScreen::mainScreen(mtm);
            let sb = screen.bounds();
            (sb.size.width, sb.size.height)
        };
        if w < 1.0 || h < 1.0 {
            return;
        }

        // SAFETY: CGColor is a CF type — objc2 guarantees CF objects lay out
        // identically to NSObjects, so the cast keeps the same object and only
        // widens the static type for storage in one NSArray<AnyObject>.
        let dark: Retained<AnyObject> = unsafe {
            Retained::cast_unchecked(Retained::<CGColor>::from(CGColor::new_generic_rgb(
                0.0, 0.0, 0.0, 0.2,
            )))
        };
        let clear: Retained<AnyObject> = unsafe {
            Retained::cast_unchecked(Retained::<CGColor>::from(CGColor::new_generic_rgb(
                0.0, 0.0, 0.0, 0.0,
            )))
        };

        let tag = NSString::from_str(NAME);
        let gradient = |frame: CGRect, dark_first: bool| {
            let g = CAGradientLayer::new();
            g.setFrame(frame);
            let pair = if dark_first {
                [dark.clone(), clear.clone()]
            } else {
                [clear.clone(), dark.clone()]
            };
            let colors = NSArray::from_retained_slice(&pair);
            // SAFETY: setColors takes CGColorRefs; the pair holds exactly that.
            unsafe { g.setColors(Some(&colors)) };
            g.setStartPoint(CGPoint::new(0.5, 0.0));
            g.setEndPoint(CGPoint::new(0.5, 1.0));
            g.setZPosition(1000.0);
            g.setName(Some(&tag));
            g
        };

        let top = gradient(
            CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(w, 80.0)),
            true,
        );
        let bottom = gradient(
            CGRect::new(CGPoint::new(0.0, h - 64.0), CGSize::new(w, 64.0)),
            false,
        );
        if laid_out {
            top.setAutoresizingMask(CAAutoresizingMask::LayerWidthSizable);
            bottom.setAutoresizingMask(CAAutoresizingMask(
                CAAutoresizingMask::LayerWidthSizable.0 | CAAutoresizingMask::LayerMinYMargin.0,
            ));
        }
        root.addSublayer(&top);
        root.addSublayer(&bottom);
    })
}

#[cfg(not(target_os = "ios"))]
fn edge_scrims(piece: impl Decorate) -> impl Piece {
    piece
}

fn build_nav(primary: bool) -> impl Piece {
    let state = AppState::ambient();
    let show_modal = state.show_network_modal;
    let s_modal = state;

    // Necessary: multi-child root zstack makes iOS content_frame pad by the home
    // inset and lifts the tab bar; single-child body keeps full-bleed layout.
    when(
        move || !show_modal.get(),
        move || nav_body(state, primary).any(),
    )
    .otherwise(move || {
        zstack((
            nav_body(s_modal, primary),
            crate::widgets::bottom_sheet::network_error_sheet(s_modal),
        ))
        .grow()
        .any()
    })
    .grow()
}

fn nav_body(state: AppState, primary: bool) -> impl Piece {
    let s_auth = state;
    let s_even = state;
    let s_odd = state;

    when(
        move || s_auth.is_authenticated.get(),
        move || {
            let s_tok1 = s_even;
            let s_tok2 = s_odd;
            edge_scrims(column((
                when(
                    move || s_tok1.ui_reload_token.get() % 2 == 0,
                    move || render_nav_content(s_even, primary),
                ),
                when(
                    move || s_tok2.ui_reload_token.get() % 2 == 1,
                    move || render_nav_content(s_odd, primary),
                ),
            )))
            .grow()
            .any()
        },
    )
    .otherwise(move || {
        if state.pin_lock_active.get() {
            edge_scrims(pages::pin_lock::render(state)).any()
        } else {
            edge_scrims(pages::login::render()).any()
        }
    })
    .grow()
}

fn render_nav_content(state: AppState, primary: bool) -> impl Piece {
    let section = state.current_section;
    let accent = Color::hex(state.accent_color.get());
    #[cfg(target_os = "ios")]
    crate::shared::colors::apply_ios_tint(state.accent_color.get());

    day::reactive::watch(
        move || section.get(),
        move |&s, _| {
            let name = match s {
                crate::Section::Diary => "diary",
                crate::Section::Schedule => "schedule",
                crate::Section::Teachers => "teachers",
                crate::Section::Settings => "settings",
                _ => "diary",
            };
            day::prefs::set("app.section", name);
        },
    );

    let sel = nav(section)
        .style(day::prelude::NavStyle::Tabs)
        .title(move || {
            if state.loading.get() {
                format!("{}  ⏳", res::str::app_title().format())
            } else {
                res::str::app_title().format()
            }
        })
        .item_icon(
            crate::Section::Diary,
            res::str::nav_diary(),
            res::vectors::tab_diary,
            pages::diary::render,
        )
        .icon_tint(accent)
        .item_icon(
            crate::Section::Schedule,
            res::str::nav_schedule(),
            res::vectors::tab_schedule,
            pages::schedule::render,
        )
        .icon_tint(accent)
        .item_icon(
            crate::Section::Teachers,
            res::str::nav_teachers(),
            res::vectors::tab_teachers,
            pages::teachers::render,
        )
        .icon_tint(accent)

        .item_icon(
            crate::Section::Settings,
            res::str::nav_settings(),
            res::vectors::tab_settings,
            pages::settings::render,
        )
        .icon_tint(accent);

    if primary {
        sel.id("nav").restore("app.section").any()
    } else {
        sel.id("nav").local().any()
    }
}
