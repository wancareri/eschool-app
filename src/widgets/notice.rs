//! App messages share the existing modal_window shell with network and lesson details.
use crate::app::AppState;
use crate::res;
use day::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NoticeKind {
    Error,
    Success,
    Info,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Notice {
    pub title: String,
    pub message: String,
    pub kind: NoticeKind,
}

impl Notice {
    pub fn error(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            kind: NoticeKind::Error,
        }
    }
}

/// Setters can also be captured by asynchronous completion callbacks.
#[derive(Clone, Copy)]
pub struct Presenter {
    notice: Setter<Option<Notice>>,
    route: Setter<Option<String>>,
}

pub fn presenter(state: AppState) -> Presenter {
    Presenter {
        notice: state.notice.setter(),
        route: state.notice_route.setter(),
    }
}

impl Presenter {
    pub fn show(self, notice: Notice) {
        let ticket = crate::shared::session::ticket();
        day::reactive::on_main(move || {
            if !ticket.is_current() {
                return;
            }
            crate::shared::keyboard::dismiss();
            self.notice.set(Some(notice));
            self.route.set(Some("notice".into()));
        });
    }
}

/// Consume each message once. Clearing its source permits a failed retry to
/// show the same message again, including after a swipe-to-dismiss.
pub fn bind_error(
    state: AppState,
    source: Signal<String>,
    format: impl Fn(&str) -> Notice + 'static,
) {
    let present = presenter(state);
    day::reactive::watch(
        move || source.get(),
        move |message, _| {
            if message.is_empty() {
                return;
            }
            let notice = format(message);
            source.set(String::new());
            present.show(notice);
        },
    );
}

pub fn modal(state: AppState) -> impl Piece {
    super::modal::modal_window(state.notice_route, 0.0, move |_| {
        // Keep the outgoing content readable until the cover finishes hiding,
        // even when logout clears the live payload at the same time.
        let initial = state.notice.get_untracked();
        let fallback_kind = initial.as_ref().map(|n| n.kind).unwrap_or(NoticeKind::Info);
        let fallback_title = initial
            .as_ref()
            .map(|n| n.title.clone())
            .unwrap_or_default();
        let fallback_message = initial.map(|n| n.message).unwrap_or_default();
        let kind = move || state.notice.get().map(|n| n.kind).unwrap_or(fallback_kind);
        column((
            when(
                move || kind() == NoticeKind::Error,
                || {
                    vector(res::vectors::status_error)
                        .frame(28.0, 28.0)
                        .tint(crate::shared::colors::ERROR)
                        .any()
                },
            )
            .otherwise(move || {
                when(
                    move || kind() == NoticeKind::Success,
                    || {
                        vector(res::vectors::check)
                            .frame(28.0, 28.0)
                            .tint(Color::hex(0x34C759))
                            .any()
                    },
                )
                .otherwise(move || {
                    vector(res::vectors::lock)
                        .frame(28.0, 28.0)
                        .tint(move || Color::hex(state.accent_color.get()))
                        .any()
                })
                .any()
            }),
            label(move || {
                state
                    .notice
                    .get()
                    .map(|n| n.title)
                    .unwrap_or_else(|| fallback_title.clone())
            })
            .font(Font::Headline)
            .grow(Dimension::Width),
            label(move || {
                state
                    .notice
                    .get()
                    .map(|n| n.message)
                    .unwrap_or_else(|| fallback_message.clone())
            })
            .font(Font::Subheadline)
            .secondary()
            .grow(Dimension::Width),
            button(res::str::notice_dismiss())
                .prominent()
                .grow(Dimension::Width)
                .action(move || state.notice_route.set(None)),
        ))
        .spacing(14.0)
        .padding(Insets {
            top: 4.0,
            leading: 12.0,
            bottom: 20.0,
            trailing: 12.0,
        })
        .grow(Dimension::Width)
    })
}
