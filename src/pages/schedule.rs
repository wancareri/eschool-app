use crate::app::AppState;
use crate::features;
use crate::widgets;
use crate::shared::utils;
use crate::res;
use day::prelude::*;
use day_piece_pullrefresh::pull_to_refresh;
use std::sync::atomic::{AtomicI32, AtomicU64, Ordering};

const PAD: f64 = 16.0;
const DAY_SWIPE_THRESHOLD: f64 = 36.0;
const DAY_AXIS_LOCK: f64 = 8.0;
const DAY_EDGE_DAMP: f64 = 0.3;

// Generation/pending scheme like the diary week pager: taps and swipes chain
// off the target still in flight, and a stale landing never stomps drag_x.
static DAY_SWIPE_GEN: AtomicU64 = AtomicU64::new(0);
static DAY_PENDING: AtomicI32 = AtomicI32::new(-1);

pub fn render() -> impl Piece {
    let state = AppState::ambient();
    let refreshing = Signal::new(false);

    zstack((
        pull_to_refresh(refreshing, scroll(column((
                    column((
                        label(move || res::str::schedule_title().format())
                            .font(Font::LargeTitle)
                            .align(TextAlign::Center),
                        label("Уроки дня")
                            .font(Font::Subheadline)
                            .secondary()
                            .align(TextAlign::Center),
                    ))
                    .spacing(6.0)
                    .padding(Insets { top: 8.0, leading: PAD, bottom: 12.0, trailing: PAD }),

                    when(
                        move || !state.is_authenticated.get(),
                        || column((
                            spacer(),
                            label("Войдите для просмотра")
                                .font(Font::Body).secondary().align(TextAlign::Center),
                            spacer(),
                        )).grow(),
                    ),
                    when(
                        move || state.is_authenticated.get() && state.schedule_loading.get(),
                        move || column((
                            spacer(),
                            widgets::spinner::render(state, 11.0),
                            label("  Загрузка…").font(Font::Caption).secondary(),
                            spacer(),
                        )).align(HAlign::Center).grow(),
                    ),
                    when(
                        move || state.is_authenticated.get() && !state.schedule_loading.get(),
                        move || schedule_content(state),
                    ),
                ))
                .spacing(0.0)
                .grow()))
                .on_refresh(move || {
                    let state = AppState::ambient();
                    let done = refreshing.setter();
                    if state.is_authenticated.get() {
                        features::diary::load_all(state);
                    }
                    done.set(false);
                })
                .grow(),

        widgets::conn_status::render()
            .padding(Insets { top: 16.0, leading: 16.0, bottom: 0.0, trailing: 0.0 }),
    ))
    .align(Alignment::TopLeading)
    .grow()
}

fn schedule_content(state: AppState) -> impl Piece {
    column((
        when(
            move || !state.timetable_days.get().is_empty(),
            move || day_pager(state),
        ),
        when(
            move || state.timetable_days.get().is_empty(),
            || label("Расписание не найдено")
                .font(Font::Body)
                .secondary()
                .align(TextAlign::Center)
                .padding(Insets { top: 60.0, leading: PAD, bottom: 60.0, trailing: PAD }),
        ),
    ))
    .spacing(0.0)
    .grow()
}

fn day_pager(state: AppState) -> impl Piece {
    let page_width = Signal::new(crate::pages::diary::get_screen_width());
    let drag_x = Signal::new(0.0);
    let w = page_width.get();
    let s_prev = state;
    let s_cur = state;
    let s_next = state;
    column((
        day_header(state, page_width, drag_x),
        zstack((
            when(
                move || s_prev.schedule_day.get() > 1,
                move || day_page(s_prev, -1, w).translation(move || drag_x.get() - w, 0.0),
            ),
            when(
                move || s_next.schedule_day.get() < 7,
                move || day_page(s_next, 1, w).translation(move || drag_x.get() + w, 0.0),
            ),
            day_page(s_cur, 0, w).translation(move || drag_x.get(), 0.0),
        ))
        .align(Alignment::TopLeading)
        .width(w)
        .on_drag(day_drag(drag_x.clone(), page_width.clone(), state.clone())),
    ))
    .spacing(0.0)
    .width(w)
}

fn day_title(state: AppState) -> String {
    let dow = state.schedule_day.get();
    let count = state
        .timetable_days
        .get()
        .into_iter()
        .find(|d| d.day_of_week == dow)
        .map(|d| d.timetable_slots.iter().filter(|ts| !ts.slots.is_empty()).count())
        .unwrap_or(0);
    let name = utils::weekday_name(dow);
    if count == 0 {
        name.to_string()
    } else {
        format!("{}  ·  {} ур.", name, count)
    }
}

fn day_header(state: AppState, page_width: Signal<f64>, drag_x: Signal<f64>) -> impl Piece {
    let s1 = state;
    let pw1 = page_width;
    let dx1 = drag_x;
    let s2 = state;
    let pw2 = page_width;
    let dx2 = drag_x;
    let sh = state;
    row((
        button("<")
            .action(move || day_go(s1, pw1, dx1, -1))
            .id("day-prev")
            .frame(44.0, 36.0),
        spacer().grow(),
        label(move || day_title(sh))
            .font(Font::Headline)
            .align(TextAlign::Center)
            .id("day-hdr"),
        spacer().grow(),
        button(">")
            .action(move || day_go(s2, pw2, dx2, 1))
            .id("day-next")
            .frame(44.0, 36.0),
    ))
    .spacing(8.0)
    .padding(Insets { top: 0.0, leading: 16.0, bottom: 6.0, trailing: 16.0 })
}

fn day_go(state: AppState, page_width: Signal<f64>, drag_x: Signal<f64>, dir: i32) {
    start_day_swipe(state, drag_x, page_width, dir, 180);
}

fn start_day_swipe(
    state: AppState,
    drag_x: Signal<f64>,
    page_width: Signal<f64>,
    dir: i32,
    dur_ms: u32,
) {
    let pending = DAY_PENDING.load(Ordering::Relaxed);
    let base = if pending >= 0 { pending } else { state.schedule_day.get() as i32 };
    let new_dow = base + dir;
    if new_dow < 1 || new_dow > 7 {
        if pending < 0 {
            with_animation(AnimSpec::ease_out(180), || {
                drag_x.set(0.0);
            });
        }
        return;
    }
    DAY_PENDING.store(new_dow, Ordering::Relaxed);
    let my_gen = DAY_SWIPE_GEN.fetch_add(1, Ordering::Relaxed) + 1;
    let w = page_width.get();
    let target = if dir > 0 { -w } else { w };
    with_animation(AnimSpec::ease_out(dur_ms), || {
        drag_x.set(target);
    });
    spawn_day_slide(drag_x.setter(), dur_ms, new_dow, my_gen);
}

fn spawn_day_slide(
    set_drag: day::reactive::Setter<f64>,
    dur_ms: u32,
    new_dow: i32,
    my_gen: u64,
) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(dur_ms as u64));
        day::reactive::on_main(move || {
            if DAY_SWIPE_GEN.load(Ordering::Relaxed) != my_gen {
                return;
            }
            day::reactive::batch(|| {
                if let Some(state) = AppState::get_main() {
                    state.schedule_day.set(new_dow as u32);
                }
                set_drag.set(0.0);
            });
            DAY_PENDING.store(-1, Ordering::Relaxed);
        });
    });
}

fn day_drag(
    drag_x: Signal<f64>,
    page_width: Signal<f64>,
    state: AppState,
) -> impl Fn(Drag) + 'static {
    let axis: Signal<Option<bool>> = Signal::new(None);
    move |drag: Drag| {
        let dx = drag.translation.x;
        let dy = drag.translation.y;
        match drag.phase {
            DragPhase::Began => {
                axis.set(None);
                drag_x.set(0.0);
            }
            DragPhase::Changed => {
                let mut horiz = axis.get();
                if horiz.is_none() && (dx.abs() > DAY_AXIS_LOCK || dy.abs() > DAY_AXIS_LOCK) {
                    horiz = Some(dx.abs() >= dy.abs() * 0.5);
                    axis.set(horiz);
                }
                if horiz == Some(false) && dx.abs() > 20.0 && dx.abs() > dy.abs() * 1.0 {
                    horiz = Some(true);
                    axis.set(horiz);
                }
                if horiz == Some(true) {
                    let pending = DAY_PENDING.load(Ordering::Relaxed);
                    let base = if pending >= 0 { pending } else { state.schedule_day.get() as i32 };
                    let mut x = dx;
                    if base <= 1 && x > 0.0 {
                        x *= DAY_EDGE_DAMP;
                    }
                    if base >= 7 && x < 0.0 {
                        x *= DAY_EDGE_DAMP;
                    }
                    drag_x.set(x);
                }
            }
            DragPhase::Ended => {
                let mut was_horiz = axis.get() == Some(true);
                let mut actual_dx = drag_x.get();
                if !was_horiz
                    && axis.get().is_none()
                    && dx.abs() >= DAY_SWIPE_THRESHOLD
                    && dx.abs() > dy.abs()
                {
                    was_horiz = true;
                    actual_dx = dx;
                    drag_x.set(dx);
                }
                axis.set(None);
                if was_horiz && actual_dx.abs() >= DAY_SWIPE_THRESHOLD {
                    let dir = if actual_dx < 0.0 { 1 } else { -1 };
                    start_day_swipe(state, drag_x, page_width, dir, 140);
                    return;
                }
                if was_horiz {
                    with_animation(AnimSpec::ease_out(180), move || {
                        drag_x.set(0.0);
                    });
                }
            }
        }
    }
}

fn day_page(state: AppState, offset: i32, w: f64) -> impl Piece {
    let dow_fn = move || (state.schedule_day.get() as i32 + offset).clamp(1, 7) as u32;
    let day_data = move || {
        let dow = dow_fn();
        state
            .timetable_days
            .get()
            .into_iter()
            .find(|d| d.day_of_week == dow)
    };
    let has_lessons = move || {
        day_data()
            .map(|d| d.timetable_slots.iter().any(|ts| !ts.slots.is_empty()))
            .unwrap_or(false)
    };

    column((
        when(
            move || has_lessons(),
            move || each(
                items(
                    move || day_lesson_rows(state, dow_fn()),
                    |r: &LessonRow| r.key.clone(),
                ),
                move |slot| lesson_table_row(state, slot).any(),
            ),
        ),
        when(
            move || !has_lessons(),
            move || label(move || {
                if dow_fn() >= 6 { "Выходной день" } else { "Нет уроков" }
            })
            .font(Font::Body)
            .secondary()
            .align(TextAlign::Center)
            .padding(Insets { top: 40.0, leading: PAD, bottom: 40.0, trailing: PAD }),
        ),
    ))
    .spacing(0.0)
    .width(w)
}

#[derive(Clone)]
struct LessonRow {
    key: String,
    num: u32,
    time: String,
    subject: String,
}

fn day_lesson_rows(state: AppState, dow: u32) -> Vec<LessonRow> {
    let days = state.timetable_days.get();
    let Some(day) = days.into_iter().find(|d| d.day_of_week == dow) else {
        return Vec::new();
    };
    let mut rows = Vec::new();
    for (i, ts) in day.timetable_slots.iter().enumerate() {
        if ts.slots.is_empty() {
            continue;
        }
        let subjects: Vec<String> = ts.slots.iter().filter_map(|s| s.summary.clone()).collect();
        let subject = if subjects.is_empty() {
            "—".into()
        } else {
            subjects.join(" / ")
        };
        rows.push(LessonRow {
            key: format!("{dow}:{i}:{}", ts.time_of_bells.number),
            num: ts.time_of_bells.number,
            time: format!("{}–{}", ts.time_of_bells.start_time, ts.time_of_bells.end_time),
            subject,
        });
    }
    rows
}

fn lesson_table_row(state: AppState, slot: ItemSlot<LessonRow, String>) -> impl Piece {
    row((
        column((
            label(move || slot.with(|r| r.num.to_string()))
                .font(Font::Footnote)
                .weight(FontWeight::Semibold)
                .color(move || Color::hex(state.accent_color.get()))
                .align(TextAlign::Center)
                .frame(26.0, 16.0),
            label(move || slot.with(|r| r.time.clone()))
                .font(Font::Caption2)
                .secondary()
                .align(TextAlign::Center),
        ))
        .spacing(3.0)
        .align(HAlign::Center),
        label(move || slot.with(|r| r.subject.clone()))
            .font(Font::Body)
            .grow(),
    ))
    .spacing(12.0)
    .align(VAlign::Center)
    .padding(Insets { top: 10.0, leading: 12.0, bottom: 10.0, trailing: 12.0 })
    .background(move || {
        if day::dark_mode() {
            Color::rgba(0.16, 0.16, 0.18, 1.0)
        } else {
            Color::rgba(0.95, 0.95, 0.97, 1.0)
        }
    })
    .corner_radius(10.0)
    .padding(Insets { top: 4.0, leading: PAD, bottom: 4.0, trailing: PAD })
}
