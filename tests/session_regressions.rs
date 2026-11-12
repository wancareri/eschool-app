#![cfg(feature = "mock")]

use day::prelude::*;
use dayapp::{
    AppState,
    app::data::{cache, session, credentials as secure},
    pages::diary::data as diary,
    pages::login::auth,
};
use std::{
    collections::{HashMap, VecDeque},
    sync::{LazyLock, Mutex},
};

type Callback = Box<dyn FnOnce() + Send>;
static QUEUE: LazyLock<Mutex<VecDeque<Callback>>> = LazyLock::new(|| Mutex::new(VecDeque::new()));

fn deliver() {
    loop {
        let callback = QUEUE.lock().unwrap().pop_front();
        match callback {
            Some(f) => f(),
            None => break,
        }
    }
}

#[test]
fn switching_identity_and_logout_reject_queued_replies_and_clear_state() {
    // Run in a subprocess so prefs use a temporary directory without changing
    // the environment of the multithreaded Rust test harness.
    if std::env::var_os("ESCHOOL_ISOLATED_TEST").is_none() {
        let prefs =
            std::env::temp_dir().join(format!("eschool-session-tests-{}", std::process::id()));
        std::fs::create_dir_all(&prefs).unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .env("ESCHOOL_ISOLATED_TEST", "1")
            .env("XDG_CONFIG_HOME", &prefs)
            .arg("--exact")
            .arg("switching_identity_and_logout_reject_queued_replies_and_clear_state")
            .arg("--nocapture")
            .output()
            .unwrap();
        let _ = std::fs::remove_dir_all(&prefs);
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    auth::logout_keys();
    day::reactive::install_main_poster(|f| QUEUE.lock().unwrap().push_back(f));
    let (mock, _) = day_mock::MockToolkit::new();
    day_core::launch_with(mock, day_spec::WindowOptions::default(), || {
        AppState::scoped(|state| {
            dayapp::components::notice::bind_error(state, state.error_msg, |message| {
                dayapp::components::notice::Notice::error("Connection", message)
            });
            spacer()
        }).any()
    });
    deliver();
    let state = AppState::get_main().unwrap();
    secure::save("auth.account_id", "test-parent");
    auth::store_ids("school", "class-a", "alice", "Alice", "School");
    day::prefs::remove("cache.v2.legacy_notes_migrated");
    day::prefs::set("cache.lesson_notes_map", r#"{"lesson-a":"legacy note"}"#);
    cache::migrate_legacy_notes();
    assert_eq!(
        cache::load("lesson_notes_map").as_deref(),
        Some(r#"{"lesson-a":"legacy note"}"#)
    );
    cache::save("user_full_name", "Alice");
    state.full_name.set("Alice".into());
    state
        .quarter_marks
        .set(HashMap::from([("Math".into(), vec![8.0])]));
    state
        .year_quarter_data
        .set(vec![("I".into(), state.quarter_marks.get())]);
    state
        .lesson_notes
        .set(HashMap::from([("lesson-a".into(), "private note".into())]));
    let old = session::ticket();
    old.publish(|state| {
        state.full_name.set("late Alice".into());
        cache::save("user_full_name", "late Alice");
    });
    session::invalidate();
    diary::clear_data(state);
    auth::store_ids("school", "class-b", "bob", "Bob", "School");
    assert!(cache::load("user_full_name").is_none());
    cache::migrate_legacy_notes();
    assert!(cache::load("lesson_notes_map").is_none());
    cache::save("user_full_name", "Bob");
    state.full_name.set("Bob".into());
    deliver();
    assert_eq!(state.full_name.get(), "Bob");
    assert_eq!(cache::load("user_full_name").as_deref(), Some("Bob"));
    assert!(state.quarter_marks.get().is_empty());
    assert!(state.year_quarter_data.get().is_empty());
    assert!(state.lesson_notes.get().is_empty());
    auth::store_ids("school", "class-a", "alice", "Alice", "School");
    assert_eq!(cache::load("user_full_name").as_deref(), Some("Alice"));
    secure::save("auth.account_id", "another-account");
    assert!(cache::load("user_full_name").is_none());

    // A dismissed error can be shown again when the same action fails twice.
    state.error_msg.set("Connection failed".into());
    deliver();
    assert!(state.error_msg.get().is_empty());
    assert_eq!(state.notice_route.get().as_deref(), Some("notice"));
    assert_eq!(state.notice.get().unwrap().message, "Connection failed");
    state.notice_route.set(None);
    deliver();
    state.error_msg.set("Connection failed".into());
    deliver();
    assert_eq!(state.notice_route.get().as_deref(), Some("notice"));
    assert_eq!(state.notice.get().unwrap().message, "Connection failed");
    // An old queued presentation must not reopen after logout.
    dayapp::components::notice::presenter(state).show(
        dayapp::components::notice::Notice::error("Old account", "Queued message"),
    );

    secure::save("auth.account_id", "test-parent");
    let old = session::ticket();
    old.publish(|state| {
        state.full_name.set("late after logout".into());
        state.week_cache.set(HashMap::from([(0, Vec::new())]));
    });
    state.settings_tab.set(2);
    state.current_section.set(dayapp::Section::Settings);
    state.loading.set(true);
    state.error_msg.set("old login error".into());
    auth::logout(state);
    deliver();
    assert_eq!(state.settings_tab.get(), 0);
    assert_eq!(state.current_section.get(), dayapp::Section::Diary);
    assert!(!state.loading.get());
    assert!(state.error_msg.get().is_empty());
    assert!(!state.is_authenticated.get());
    assert!(state.notice_route.get().is_none());
    assert!(state.notice.get().is_none());
    assert!(state.show_network_modal.get().is_none());
    assert!(state.full_name.get().is_empty());
    assert!(state.week_cache.get().is_empty());
    assert!(state.lesson_notes.get().is_empty());
    assert!(state.quarter_all_marks.get().is_empty());
    assert!(state.lesson_attachments.get().is_empty());
    assert_eq!(
        auth::get_stored_ids(),
        (String::new(), String::new(), String::new())
    );
    auth::store_ids("school", "class-a", "alice", "Alice", "School");
    secure::save("auth.account_id", "test-parent");
    assert!(cache::load("user_full_name").is_none());
    auth::logout_keys();
}
