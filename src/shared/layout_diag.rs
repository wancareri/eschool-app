//! Layout positioning diagnostics and swizzling.

use day::prelude::*;

pub fn init() {
    #[cfg(target_os = "ios")]
    ios::install();
}

pub trait PosLogExt: Decorate + Sized {
    #[cfg(target_os = "ios")]
    fn log_pos(self, tag: &'static str) -> Decorated<Self> {
        use day_uikit::UiKitExt;
        self.uikit(move |view, class, _mtm| {
            let f = view.frame();
            let b = view.bounds();
            crate::shared::nslog::nslog(&format!(
                "[MountPos:{tag}] class={class} frame=({:.1}, {:.1}, {:.1}x{:.1}) bounds=({:.1}x{:.1})",
                f.origin.x, f.origin.y, f.size.width, f.size.height,
                b.size.width, b.size.height
            ));
        })
    }

    #[cfg(not(target_os = "ios"))]
    fn log_pos(self, _tag: &'static str) -> Self {
        self
    }
}

impl<P: Decorate> PosLogExt for P {}

pub fn debug_log_button(open: Signal<bool>) -> impl Piece {
    row((
        spacer().grow(),
        button("📋 Логи")
            .action(move || {
                crate::shared::haptics::tick();
                open.set(true);
            })
            .padding(Insets { top: 4.0, leading: 8.0, bottom: 4.0, trailing: 8.0 })
            .background(Color::rgba(0.5, 0.5, 0.5, 0.2))
            .corner_radius(12.0),
    ))
    .padding(Insets { top: 8.0, leading: 0.0, bottom: 0.0, trailing: 16.0 })
}

pub fn debug_log_overlay(open: Signal<bool>) -> impl Piece {
    let copied = Signal::new(false);

    column((
        row((
            label("Логи позиционирования").font(Font::Headline).grow(),
            button(move || if copied.get() { "Скопировано!" } else { "Скопировать" })
                .action(move || {
                    let txt = crate::shared::nslog::get_logs();
                    day_part_clipboard::set_text(&txt);
                    crate::shared::haptics::pop();
                    copied.set(true);
                }),
            spacer().width(8.0),
            button("✕ Закрыть")
                .action(move || {
                    crate::shared::haptics::tick();
                    open.set(false);
                }),
        ))
        .align(VAlign::Center)
        .padding(Insets { top: 12.0, leading: 16.0, bottom: 8.0, trailing: 16.0 }),

        scroll(
            label(move || {
                let all = crate::shared::nslog::get_logs();
                let lines: Vec<&str> = all.lines().rev().take(80).collect();
                let mut reversed = lines;
                reversed.reverse();
                reversed.join("\n")
            })
            .font(Font::Caption2)
            .padding(Insets { top: 8.0, leading: 12.0, bottom: 16.0, trailing: 12.0 })
            .grow_w()
        )
        .grow(),
    ))
    .background(move || {
        if day::dark_mode() {
            Color::rgba(0.12, 0.12, 0.14, 0.98)
        } else {
            Color::rgba(0.96, 0.96, 0.98, 0.98)
        }
    })
    .corner_radius(16.0)
    .padding(Insets { top: 44.0, leading: 8.0, bottom: 20.0, trailing: 8.0 })
    .grow()
}

#[cfg(target_os = "ios")]
mod ios {
    use objc2::runtime::{AnyClass, AnyObject, Imp, Sel};
    use objc2_core_foundation::{CGPoint, CGRect, CGSize};
    use objc2_foundation::NSString;
    use std::sync::atomic::{AtomicBool, Ordering};

    static INSTALLED: AtomicBool = AtomicBool::new(false);
    static mut ORIG_VIEW_SET_FRAME: Option<unsafe extern "C-unwind" fn(*mut AnyObject, Sel, CGRect)> = None;
    static mut ORIG_SCROLL_SET_OFFSET: Option<unsafe extern "C-unwind" fn(*mut AnyObject, Sel, CGPoint)> = None;

    unsafe extern "C-unwind" fn swizzled_view_set_frame(
        this: *mut AnyObject,
        cmd: Sel,
        frame: CGRect,
    ) {
        unsafe {
            if let Some(orig) = ORIG_VIEW_SET_FRAME {
                orig(this, cmd, frame);
            }
        }

        if this.is_null() {
            return;
        }

        let cls_name = unsafe { (*this).class().name().to_str().unwrap_or("Unknown") };

        let id_str: Option<String> = unsafe {
            let id: *mut NSString = objc2::msg_send![this, accessibilityIdentifier];
            if !id.is_null() {
                Some((*id).to_string())
            } else {
                None
            }
        };

        let is_tf = cls_name.contains("TextField");
        let is_scroll = cls_name.contains("ScrollView");
        let has_id = id_str.is_some();
        let is_small = (frame.size.width > 0.0 && frame.size.width < 50.0)
            || (frame.size.height > 0.0 && frame.size.height < 50.0);

        if is_tf || is_scroll || has_id || is_small {
            let id_info = id_str.map(|s| format!(" id='{s}'")).unwrap_or_default();
            crate::shared::nslog::nslog(&format!(
                "[Layout:Pos] <{cls_name}: {:p}>{id_info} frame=({:.1}, {:.1}, {:.1}x{:.1})",
                this, frame.origin.x, frame.origin.y, frame.size.width, frame.size.height
            ));
        }
    }

    unsafe extern "C-unwind" fn swizzled_scroll_set_offset(
        this: *mut AnyObject,
        cmd: Sel,
        offset: CGPoint,
    ) {
        unsafe {
            if let Some(orig) = ORIG_SCROLL_SET_OFFSET {
                orig(this, cmd, offset);
            }
        }
        if this.is_null() {
            return;
        }
        let cs: CGSize = unsafe { objc2::msg_send![this, contentSize] };
        crate::shared::nslog::nslog(&format!(
            "[Layout:ScrollOffset] <{:p}> offset=({:.1}, {:.1}) contentSize={:.1}x{:.1}",
            this, offset.x, offset.y, cs.width, cs.height
        ));
    }

    pub fn install() {
        if INSTALLED.swap(true, Ordering::SeqCst) {
            return;
        }

        // Swizzle UIView setFrame:
        if let Some(cls) = AnyClass::get(c"UIView") {
            let sel = objc2::sel!(setFrame:);
            if let Some(method) = cls.instance_method(sel) {
                let orig: Imp = method.implementation();
                unsafe {
                    ORIG_VIEW_SET_FRAME = Some(std::mem::transmute(orig));
                    let imp: Imp = std::mem::transmute(swizzled_view_set_frame as *const ());
                    method.set_implementation(imp);
                }
                crate::shared::nslog::nslog("[LayoutDiag] Swizzled UIView setFrame:");
            }
        }

        // Swizzle UIScrollView setContentOffset:
        if let Some(cls) = AnyClass::get(c"UIScrollView") {
            let sel = objc2::sel!(setContentOffset:);
            if let Some(method) = cls.instance_method(sel) {
                let orig: Imp = method.implementation();
                unsafe {
                    ORIG_SCROLL_SET_OFFSET = Some(std::mem::transmute(orig));
                    let imp: Imp = std::mem::transmute(swizzled_scroll_set_offset as *const ());
                    method.set_implementation(imp);
                }
                crate::shared::nslog::nslog("[LayoutDiag] Swizzled UIScrollView setContentOffset:");
            }
        }
    }
}
