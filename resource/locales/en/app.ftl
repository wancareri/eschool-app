# Eschool App — UI strings (https://daybrite.dev/docs/localization). Add a locale by dropping a
# sibling folder (e.g. locales/fr/app.ftl) and translating — the generated
# res::locales::install() in src/lib.rs picks up every locale directory by itself.
#
# The appearance and language rows on the Settings page label THEMSELVES from Day's own catalog,
# so there are no keys for them here.

app_title = ESchool

nav_diary = Diary
nav_schedule = Schedule
nav_teachers = Teachers
nav_settings = Settings
nav_login = Account

# Menus
menu_file = File
menu_edit = Edit

# Commands (kept for model.rs compatibility)
cmd_add = New Item
cmd_delete = Delete
cmd_done = Done
cmd_show_done = Show Finished
cmd_sidebar = Toggle Sidebar

# Template items (kept for model.rs compatibility)
item_none = Select an item
item_kind_note = Note
item_kind_task = Task
item_kind_idea = Idea

section_basics = Basics
section_details = Details
section_notes = Notes

field_name = Name
field_name_hint = Name this item…
field_count = Count
field_date = Date
field_kind = Kind
field_done = Done
field_rating = Rating
field_color = Color

# E-schools specific
diary_title = Diary
schedule_title = Schedule
teachers_title = Teachers
settings_title = Settings

auth_appearance = Appearance
auth_language = Language

auth_options = Sign-in settings
auth_theme_system = System
auth_theme_light = Light
auth_theme_dark = Dark
auth_username_title = Sign in
auth_password_title = Enter password
auth_intro_subtitle = School diary
auth_intro_description = Your timetable, grades and homework, always at hand
auth_start = Get started
auth_username_placeholder = Username or code
auth_password_placeholder = Password
auth_remember = Remember sign-in
auth_continue = Continue
auth_sign_in = Sign in
auth_signing_in = Signing in…
auth_step_username = Step 1 of 2 · Username
auth_step_password = Step 2 of 2 · Password
auth_setup_title = New password
