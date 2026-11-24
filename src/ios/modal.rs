use block2::{Block, RcBlock};
use objc2::rc::{Retained, Weak};
use objc2::runtime::{AnyClass, AnyObject, Bool, Imp, ProtocolObject, Sel};
use objc2::{ClassType, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_core_foundation::CGAffineTransform;
use objc2_foundation::{NSObject, NSObjectProtocol};
use objc2_quartz_core::{CACornerMask, kCACornerCurveContinuous};
use objc2_ui_kit::{
    UIButton, UIColor, UIGestureRecognizer, UIGestureRecognizerDelegate, UIModalPresentationStyle,
    UIModalTransitionStyle, UIPanGestureRecognizer, UITouch,
    UITransitionContextFromViewControllerKey, UITransitionContextToViewControllerKey, UIView,
    UIViewAnimationOptions, UIViewController, UIViewControllerAnimatedTransitioning,
    UIViewControllerContextTransitioning, UIViewControllerTransitioningDelegate,
};

pub(crate) const CARD_TAG: isize = 0x45534301;
pub(crate) const FILLER_TAG: isize = 0x45534302;
const DURATION: f64 = 0.38;
const COVER_TAG: isize = 0x45534303;

type PresentViewController = unsafe extern "C-unwind" fn(
    *mut AnyObject,
    Sel,
    &UIViewController,
    Bool,
    Option<&Block<dyn Fn()>>,
);

static ORIGINAL_PRESENT: std::sync::OnceLock<PresentViewController> = std::sync::OnceLock::new();

unsafe extern "C-unwind" fn present_cover(
    this: *mut AnyObject,
    cmd: Sel,
    vc: &UIViewController,
    animated: Bool,
    completion: Option<&Block<dyn Fn()>>,
) {
    configure_presentation(vc);
    if let Some(original) = ORIGINAL_PRESENT.get() {
        unsafe { original(this, cmd, vc, animated, completion) };
    }
}

fn configure_presentation(vc: &UIViewController) {
    if vc.class().name().to_bytes() == b"DayCoverVC" {
        vc.setModalPresentationStyle(UIModalPresentationStyle::OverFullScreen);
        vc.setModalTransitionStyle(UIModalTransitionStyle::CoverVertical);
        DELEGATE.with(|delegate| {
            vc.setTransitioningDelegate(Some(ProtocolObject::from_ref(&**delegate)));
        });
        if let Some(root) = vc.viewIfLoaded() {
            root.setOpaque(false);
            root.setBackgroundColor(Some(&UIColor::clearColor()));
        }
    }
}

pub(crate) fn install_presentation_hook() {
    ORIGINAL_PRESENT.get_or_init(|| {
        let class = UIViewController::class();
        let selector = objc2::sel!(presentViewController:animated:completion:);
        let method = class
            .instance_method(selector)
            .expect("UIViewController presentation method");
        let previous = method.implementation();
        let replacement: Imp = unsafe { std::mem::transmute(present_cover as *const ()) };
        unsafe {
            let encoding = objc2::ffi::method_getTypeEncoding(method);
            objc2::ffi::class_replaceMethod(
                class as *const AnyClass as *mut AnyClass,
                selector,
                replacement,
                encoding,
            );
        }
        unsafe { std::mem::transmute::<Imp, PresentViewController>(previous) }
    });
}

thread_local! {
    static DELEGATE: Retained<ModalTransitionDelegate> = {
        let mtm = MainThreadMarker::new().expect("UIKit main thread");
        unsafe { msg_send![ModalTransitionDelegate::alloc(mtm), init] }
    };
    static SHEET_PAN_DELEGATE: Retained<SheetPanDelegate> = {
        let mtm = MainThreadMarker::new().expect("UIKit main thread");
        unsafe { msg_send![SheetPanDelegate::alloc(mtm), init] }
    };
}

// A sheet still pans from its body, but a pull which begins on a native button
// belongs to that button. Without this delegate UIKit recognizes the parent
// pan and makes the button and whole modal jump together.
define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    struct SheetPanDelegate;

    unsafe impl NSObjectProtocol for SheetPanDelegate {}
    unsafe impl UIGestureRecognizerDelegate for SheetPanDelegate {
        #[unsafe(method(gestureRecognizer:shouldReceiveTouch:))]
        fn should_receive_touch(&self, _recognizer: &UIGestureRecognizer, touch: &UITouch) -> Bool {
            let mut view = touch.view();
            while let Some(candidate) = view {
                if candidate.downcast_ref::<UIButton>().is_some() {
                    return Bool::NO;
                }
                view = candidate.superview();
            }
            Bool::YES
        }
    }
);

/// The sheet is a bottom-attached surface: only its two upper corners are
/// visible.  Reapply once after Day's surface patch so its child cannot turn
/// the edge square during the first UIKit layout pass.
pub(crate) fn configure_card_surface(view: &UIView, mtm: MainThreadMarker) {
    fn apply(view: &UIView) {
        view.setTag(CARD_TAG);
        let layer = view.layer();
        layer.setCornerRadius(26.0);
        unsafe { layer.setCornerCurve(kCACornerCurveContinuous) };
        layer.setMaskedCorners(
            CACornerMask::LayerMinXMinYCorner | CACornerMask::LayerMaxXMinYCorner,
        );
        layer.setMasksToBounds(true);
    }

    apply(view);
    let weak = dispatch2::MainThreadBound::new(Weak::new(view), mtm);
    dispatch2::DispatchQueue::main().exec_async(move || {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let Some(view) = weak.get(mtm).load() else {
            return;
        };
        apply(&view);
        // Day installs its drag recognizer during realization.  This second
        // pass sees it and only filters button-originated touches; it leaves
        // the established whole-card drag behavior intact everywhere else.
        SHEET_PAN_DELEGATE.with(|delegate| {
            for recognizer in view.gestureRecognizers().into_iter().flatten() {
                if recognizer
                    .downcast_ref::<UIPanGestureRecognizer>()
                    .is_some()
                {
                    recognizer.setDelegate(Some(ProtocolObject::from_ref(&**delegate)));
                }
            }
        });
    });
}

pub(crate) fn configure_cover(view: &UIView, _mtm: MainThreadMarker) {
    let mut next = view.nextResponder();
    while let Some(responder) = next {
        if let Some(vc) = responder.downcast_ref::<UIViewController>() {
            if vc.class().name().to_bytes() != b"DayCoverVC" {
                break;
            }
            if let Some(root) = vc.view() {
                root.setTag(COVER_TAG);
            }
            install_presentation_hook();
            configure_presentation(vc);
            return;
        }
        next = responder.nextResponder();
    }
    crate::ios::nslog::nslog("[Modal] ERROR: dim is not attached to DayCoverVC");
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    struct ModalTransitionDelegate;

    unsafe impl NSObjectProtocol for ModalTransitionDelegate {}

    unsafe impl UIViewControllerTransitioningDelegate for ModalTransitionDelegate {
        #[unsafe(method_id(animationControllerForPresentedController:presentingController:sourceController:))]
        fn presentation_animator(
            &self,
            presented: &UIViewController,
            _presenting: &UIViewController,
            _source: &UIViewController,
        ) -> Option<Retained<ProtocolObject<dyn UIViewControllerAnimatedTransitioning>>> {
            presented.setModalPresentationStyle(UIModalPresentationStyle::OverFullScreen);
            presented.setTransitioningDelegate(Some(ProtocolObject::from_ref(self)));
            if let Some(root) = presented.view() {
                root.setOpaque(false);
                root.setBackgroundColor(Some(&UIColor::clearColor()));
            }
            Some(ProtocolObject::from_retained(ModalAnimator::new(
                self.mtm(),
                true,
            )))
        }

        #[unsafe(method_id(animationControllerForDismissedController:))]
        fn dismissal_animator(
            &self,
            _dismissed: &UIViewController,
        ) -> Option<Retained<ProtocolObject<dyn UIViewControllerAnimatedTransitioning>>> {
            Some(ProtocolObject::from_retained(ModalAnimator::new(
                self.mtm(),
                false,
            )))
        }
    }
);

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = bool]
    struct ModalAnimator;

    unsafe impl NSObjectProtocol for ModalAnimator {}

    unsafe impl UIViewControllerAnimatedTransitioning for ModalAnimator {
        #[unsafe(method(transitionDuration:))]
        fn duration(
            &self,
            _context: Option<&ProtocolObject<dyn UIViewControllerContextTransitioning>>,
        ) -> f64 {
            DURATION
        }

        #[unsafe(method(animateTransition:))]
        fn animate(&self, context: &ProtocolObject<dyn UIViewControllerContextTransitioning>) {
            animate(context, *self.ivars());
        }
    }
);

impl ModalAnimator {
    fn new(mtm: MainThreadMarker, presenting: bool) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(presenting);
        unsafe { msg_send![super(this), init] }
    }
}

fn motion_views(root: &UIView, out: &mut Vec<Retained<UIView>>) {
    for child in root.subviews() {
        if matches!(child.tag(), CARD_TAG | FILLER_TAG) {
            out.push(child);
        } else {
            motion_views(&child, out);
        }
    }
}

fn settle_initial_layout(view: &UIView) {
    view.layer().removeAllAnimations();
    for child in view.subviews() {
        settle_initial_layout(&child);
    }
}

fn identity_transform() -> CGAffineTransform {
    CGAffineTransform {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        tx: 0.0,
        ty: 0.0,
    }
}

fn animate(context: &ProtocolObject<dyn UIViewControllerContextTransitioning>, presenting: bool) {
    let key = unsafe {
        if presenting {
            UITransitionContextToViewControllerKey
        } else {
            UITransitionContextFromViewControllerKey
        }
    };
    let Some(vc) = context.viewControllerForKey(key) else {
        context.completeTransition(false);
        return;
    };
    let Some(root) = vc.view() else {
        context.completeTransition(false);
        return;
    };
    let container = context.containerView();
    let root_mtm = root.mtm();

    if presenting {
        let final_frame = context.finalFrameForViewController(&vc);
        let frame = if final_frame.size.width > 0.0 && final_frame.size.height > 0.0 {
            final_frame
        } else {
            container.bounds()
        };
        root.setFrame(frame);
        container.addSubview(&root);
        root.layoutIfNeeded();
        settle_initial_layout(&root);
    }

    let mut views = Vec::new();
    motion_views(&root, &mut views);

    let travel = views
        .iter()
        .find(|v| v.tag() == CARD_TAG)
        .map_or(0.0, |card| {
            let frame = card.convertRect_toView(card.bounds(), Some(&root));
            (root.bounds().size.height - frame.origin.y).max(0.0)
        });
    let travel = if travel > 0.0 {
        travel
    } else {
        root.bounds().size.height
    };

    let surfaces: Vec<_> = views
        .into_iter()
        .map(|view| {
            let rest = identity_transform();
            let offscreen = CGAffineTransform {
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: 1.0,
                tx: 0.0,
                ty: travel,
            };
            (view, rest, offscreen)
        })
        .collect();

    let dim = UIColor::colorWithWhite_alpha(0.0, crate::components::modal::DIM_ALPHA);
    let clear = UIColor::clearColor();

    if presenting {
        UIView::performWithoutAnimation(
            &RcBlock::new(|| {
                root.setTransform(identity_transform());
                root.setOpaque(false);
                root.setBackgroundColor(Some(&clear));
                root.setAlpha(1.0);
                for (view, _, offscreen) in &surfaces {
                    view.setTransform(*offscreen);
                }
            }),
            root_mtm,
        );
    }

    let animation_root = root.clone();
    let animation_surfaces = surfaces.clone();
    let animation_dim = dim.clone();
    let animation_clear = clear.clone();
    let animations = RcBlock::new(move || {
        animation_root.setBackgroundColor(Some(if presenting {
            &animation_dim
        } else {
            &animation_clear
        }));
        for (view, rest, offscreen) in &animation_surfaces {
            view.setTransform(if presenting { *rest } else { *offscreen });
        }
    });

    let context = unsafe {
        Retained::retain(
            context as *const _ as *mut ProtocolObject<dyn UIViewControllerContextTransitioning>,
        )
    }
    .expect("live transition context");

    let completion = RcBlock::new(move |_finished: objc2::runtime::Bool| {
        let completed = !context.transitionWasCancelled();
        let visible = presenting == completed;
        UIView::performWithoutAnimation(
            &RcBlock::new(|| {
                if !visible {
                    root.removeFromSuperview();
                }
                for (view, rest, _) in &surfaces {
                    view.setTransform(*rest);
                }
                root.setTransform(identity_transform());
                root.setAlpha(1.0);
                root.setBackgroundColor(Some(if visible { &dim } else { &clear }));
            }),
            root_mtm,
        );
        context.completeTransition(completed);
    });

    UIView::animateWithDuration_delay_options_animations_completion(
        DURATION,
        0.0,
        UIViewAnimationOptions::CurveEaseOut,
        &animations,
        Some(&completion),
        root_mtm,
    );
}
