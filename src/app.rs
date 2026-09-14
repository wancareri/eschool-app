use crate::core::colors;
use crate::core::state::ESchoolState;
use crate::model::Scene;
use crate::res;
use crate::Section;
use crate::ui::pages::*;
use day::prelude::*;

/// Build the navigation selector — fully reactive to auth state.
///
/// When **not** authenticated: full-screen login page, no tab bar.
/// When authenticated: tab-bar with Diary / Schedule / Teachers / Settings.
pub(crate) fn build_nav(scene: Scene, primary: bool) -> AnyPiece {
    let state = ESchoolState::ambient();

    column((
        // ── not authenticated: full-screen login ───────────────────────
        when(
            move || !state.is_authenticated.get(),
            move || login_page().any(),
        ),
        // ── authenticated: tab-bar navigation ──────────────────────────
        when(
            move || state.is_authenticated.get(),
            move || {
                let sel = nav(scene.section)
                    .title(res::str::app_title())
                    .sidebar_toggle(true)
                    .item_icon(
                        Section::Diary,
                        res::str::nav_diary(),
                        res::vectors::tab_diary,
                        diary_page,
                    )
                    .icon_tint(colors::NAV_DIARY)
                    .item_icon(
                        Section::Schedule,
                        res::str::nav_schedule(),
                        res::vectors::tab_schedule,
                        schedule_page,
                    )
                    .icon_tint(colors::NAV_SCHEDULE)
                    .item_icon(
                        Section::Teachers,
                        res::str::nav_teachers(),
                        res::vectors::tab_teachers,
                        teachers_page,
                    )
                    .icon_tint(colors::NAV_TEACHERS)
                    .item_icon(
                        Section::Settings,
                        res::str::nav_settings(),
                        res::vectors::tab_settings,
                        settings_page,
                    )
                    .icon_tint(colors::NAV_SETTINGS);

                if primary {
                    sel.id("nav").restore("app.section").any()
                } else {
                    sel.id("nav").local().any()
                }
            },
        ),
    ))
    .grow()
    .any()
}
