//! App-owned transitions for the shared bottom-sheet shell on Day 0.7.1.
//! Configure only this shell's DayCoverVC; PIN setup keeps its normal cover.

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_core_foundation::CGAffineTransform;
use objc2_foundation::{NSObject, NSObjectProtocol};
use objc2_ui_kit::{
    UIColor, UIModalPresentationStyle, UITransitionContextFromViewControllerKey,
    UITransitionContextToViewControllerKey, UIView, UIViewAnimationOptions, UIViewController,
    UIViewControllerAnimatedTransitioning, UIViewControllerContextTransitioning,
    UIViewControllerTransitioningDelegate,
};

pub(super) const CARD_TAG: isize = 0x45534301;
pub(super) const FILLER_TAG: isize = 0x45534302;
const DURATION: f64 = 0.38;

thread_local! {
    // UIViewController.transitioningDelegate is weak. This stateless delegate
    // lives for the main thread; animators retain only one transition's views.
    static DELEGATE: Retained<ModalTransitionDelegate> = {
        let mtm = MainThreadMarker::new().expect("UIKit main thread");
        unsafe { msg_send![ModalTransitionDelegate::alloc(mtm), init] }
    };
}

pub(super) fn configure_cover(view: &UIView, _mtm: MainThreadMarker) {
    let mut next = view.nextResponder();
    while let Some(responder) = next {
        if let Some(vc) = responder.downcast_ref::<UIViewController>() {
            // A mismatched hierarchy must never reconfigure a page/navigation VC.
            if vc.class().name().to_bytes() != b"DayCoverVC" {
                break;
            }
            vc.setModalPresentationStyle(UIModalPresentationStyle::OverFullScreen);
            if let Some(root) = vc.view() {
                root.setOpaque(false);
                root.setBackgroundColor(Some(&UIColor::clearColor()));
            }
            DELEGATE.with(|delegate| {
                vc.setTransitioningDelegate(Some(ProtocolObject::from_ref(&**delegate)));
            });
            return;
        }
        next = responder.nextResponder();
    }
    crate::shared::nslog::nslog("[Modal] ERROR: dim is not attached to DayCoverVC");
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
            _presented: &UIViewController,
            _presenting: &UIViewController,
            _source: &UIViewController,
        ) -> Option<Retained<ProtocolObject<dyn UIViewControllerAnimatedTransitioning>>> {
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
    // Day applies the card's implicit resize animation even to its first
    // zero-to-real frame assignment. Its model geometry is already final;
    // discard only those preparation animations while the root is invisible.
    view.layer().removeAllAnimations();
    for child in view.subviews() {
        settle_initial_layout(&child);
    }
}

fn identity_transform() -> CGAffineTransform {
    CGAffineTransform { a: 1.0, b: 0.0, c: 0.0, d: 1.0, tx: 0.0, ty: 0.0 }
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
        // Prepare the root and surfaces before registering the animation in
        // UIKit's current presentation context.
        let prepare = RcBlock::new(|| {
            root.setAlpha(0.0);
            root.setTransform(identity_transform());
            root.setFrame(context.finalFrameForViewController(&vc));
            container.addSubview(&root);
            root.layoutIfNeeded();
            let mut surfaces = Vec::new();
            motion_views(&root, &mut surfaces);
            for surface in surfaces {
                surface.setTransform(CGAffineTransform {
                    ty: root.bounds().size.height, ..identity_transform()
                });
            }
        });
        UIView::performWithoutAnimation(&prepare, root_mtm);
    }
    root.layoutIfNeeded();
    let context = unsafe {
        Retained::retain(
            context as *const _ as *mut ProtocolObject<dyn UIViewControllerContextTransitioning>,
        )
    }
    .expect("live transition context");
    animate_surfaces(&context, presenting);
}

fn animate_surfaces(
    context: &Retained<ProtocolObject<dyn UIViewControllerContextTransitioning>>,
    presenting: bool,
) {
    let key = unsafe {
        if presenting {
            UITransitionContextToViewControllerKey
        } else {
            UITransitionContextFromViewControllerKey
        }
    };
    let Some(root) = context.viewControllerForKey(key).and_then(|vc| vc.view()) else {
        context.completeTransition(false);
        return;
    };
    let root_mtm = root.mtm();
    // DayNavPageView publishes the safe-area content size during layout. Let Day
    // place the card before measuring the travel; no screen-size/timer guesses.
    root.layoutIfNeeded();
    if presenting {
        settle_initial_layout(&root);
    }
    let mut views = Vec::new();
    motion_views(&root, &mut views);
    // Normalize inner surfaces before measuring. A theme relayout or pager
    // transform must never become part of this vertical-only transition.
    for view in &views {
        view.setTransform(identity_transform());
    }
    let travel = views
        .iter()
        .find(|v| v.tag() == CARD_TAG)
        .map_or(0.0, |card| {
            let frame = card.convertRect_toView(card.bounds(), Some(&root));
            (root.bounds().size.height - frame.origin.y).max(0.0)
        });
    // These are INNER native surfaces. Day owns the outer translation layer for
    // drag/keyboard movement; the animator never overwrites that layer. Dismissal
    // therefore starts at the released position and only travels further down.
    let surfaces: Vec<_> = views
        .into_iter()
        .map(|view| {
            let rest = identity_transform();
            let offscreen = CGAffineTransform {
                a: 1.0, b: 0.0, c: 0.0, d: 1.0, tx: 0.0, ty: travel,
            };
            (view, rest, offscreen)
        })
        .collect();
    let dim = root.backgroundColor().unwrap_or_else(UIColor::clearColor);
    let clear = UIColor::clearColor();
    if presenting {
        root.setBackgroundColor(Some(&clear));
        for (view, _, offscreen) in &surfaces {
            view.setTransform(*offscreen);
        }
        root.setAlpha(1.0);
    }
    let animation_root = root.clone();
    let animation_surfaces = surfaces.clone();
    let animation_dim = dim.clone();
    let animations = RcBlock::new(move || {
        animation_root.setBackgroundColor(Some(if presenting { &animation_dim } else { &clear }));
        for (view, rest, offscreen) in &animation_surfaces {
            view.setTransform(if presenting { *rest } else { *offscreen });
        }
    });
    // UIKit keeps the context alive until completeTransition, and this block
    // additionally retains it across the asynchronous UIView animation.
    let context = context.clone();
    let completion = RcBlock::new(move |_finished: objc2::runtime::Bool| {
        let completed = !context.transitionWasCancelled();
        if presenting != completed {
            root.removeFromSuperview();
        }
        for (view, rest, _) in &surfaces {
            view.setTransform(*rest);
        }
        root.setBackgroundColor(Some(&dim));
        context.completeTransition(completed);
    });
    UIView::animateWithDuration_delay_options_animations_completion(
        DURATION,
        0.0,
        UIViewAnimationOptions::CurveEaseOut | UIViewAnimationOptions::BeginFromCurrentState,
        &animations,
        Some(&completion),
        root_mtm,
    );
}
