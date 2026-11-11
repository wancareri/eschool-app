//! Direction arbitration before Day's one-finger drag can cancel a scroll.
use objc2::runtime::ProtocolObject;
use objc2::{MainThreadMarker, MainThreadOnly, define_class, msg_send, rc::Retained, rc::Weak};
use objc2_foundation::{NSObject, NSObjectProtocol};
use objc2_ui_kit::{
    UIGestureRecognizer, UIGestureRecognizerDelegate, UIPanGestureRecognizer, UIScrollView, UIView,
};

thread_local! {
    // UIGestureRecognizer.delegate is weak. This delegate has no view references.
    static HORIZONTAL: Retained<HorizontalPagerDelegate> = unsafe {
        msg_send![HorizontalPagerDelegate::alloc(MainThreadMarker::new().unwrap()), init]
    };
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    struct HorizontalPagerDelegate;

    unsafe impl NSObjectProtocol for HorizontalPagerDelegate {}
    unsafe impl UIGestureRecognizerDelegate for HorizontalPagerDelegate {
        #[unsafe(method(gestureRecognizerShouldBegin:))]
        fn should_begin(&self, recognizer: &UIGestureRecognizer) -> bool {
            recognizer
                .downcast_ref::<UIPanGestureRecognizer>()
                .is_some_and(|pan| {
                    let view = pan.view();
                    let velocity = pan.velocityInView(view.as_deref());
                    // Equal/vertical travel fails while still Possible, releasing
                    // the ancestor scroll pan and UIRefreshControl immediately.
                    velocity.x.abs() > velocity.y.abs()
                })
        }
    }
);

fn prioritize_pager(view: &UIView, pan: &UIPanGestureRecognizer) {
    let mut ancestor = view.superview();
    while let Some(parent) = ancestor {
        if let Some(scroll) = parent.downcast_ref::<UIScrollView>() {
            scroll
                .panGestureRecognizer()
                .requireGestureRecognizerToFail(pan);
            break;
        }
        ancestor = parent.superview();
    }
}

pub fn horizontal_pager(view: &UIView, mtm: MainThreadMarker) {
    for recognizer in view.gestureRecognizers().into_iter().flatten() {
        let Some(pan) = recognizer.downcast_ref::<UIPanGestureRecognizer>() else {
            continue;
        };
        pan.setMaximumNumberOfTouches(1);
        HORIZONTAL.with(|delegate| pan.setDelegate(Some(ProtocolObject::from_ref(&**delegate))));
        prioritize_pager(view, pan);
    }
    // During a child build the scroll may not have attached its content yet.
    // A weak reference also makes a disposed/rebuilt page a harmless no-op.
    let weak = dispatch2::MainThreadBound::new(Weak::new(view), mtm);
    dispatch2::DispatchQueue::main().exec_async(move || {
        let mtm = MainThreadMarker::new().unwrap();
        let Some(view) = weak.get(mtm).load() else {
            return;
        };
        for recognizer in view.gestureRecognizers().into_iter().flatten() {
            if let Some(pan) = recognizer.downcast_ref::<UIPanGestureRecognizer>() {
                prioritize_pager(&view, pan);
            }
        }
    });
}
