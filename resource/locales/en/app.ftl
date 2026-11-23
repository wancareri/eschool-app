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

# E-schools specific
diary_title = Diary
schedule_title = Schedule
teachers_title = Teachers
teachers_search = Find a subject or teacher
teachers_empty = No teachers yet
teachers_no_matches = Nothing found
teachers_empty_description = Data will appear after the diary synchronizes.
teachers_count_prefix = Subjects
teachers_group_prefix = Group
teachers_try_search = Change the search and try again.
settings_title = Settings
settings_tab_basics = General
settings_tab_appearance = Appearance
settings_tab_security = Security
settings_tab_dev = Dev
parent_child_prefix = Student
settings_profile = Profile
settings_class = Class
settings_colors = Colors
settings_accent_color = Accent color
settings_color_blue = Blue
settings_color_green = Green
settings_color_purple = Purple
settings_color_orange = Orange
settings_color_red = Red
settings_feedback = Feedback
settings_haptics = Haptics for actions
settings_haptics_hint = Feel week swipes and section changes
settings_lock = App lock
settings_pin_login = Require PIN at sign-in
settings_pin_active = PIN is active
settings_change = Change
settings_security = Security
settings_biometric_login = Sign in with Face ID or Touch ID
settings_biometric_unavailable = Biometrics aren’t available on this device
settings_enable_pin_first = Enable the PIN first
settings_account = Account
settings_logout = Sign out
settings_dev_tools = Developer tools
settings_version = Version
settings_refresh_token = Refresh token
settings_clear_cache = Clear cache
settings_reload_data = Reload data
settings_app_logs = Application logs
settings_copy_logs = Copy logs
settings_clear = Clear
settings_logs_copied = Logs copied
settings_logs_cleared = Logs cleared
settings_log_entries = Buffered entries
settings_logs = Logs
settings_about = About OpenSchool
settings_information = Information

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

auth_error_login_title = Enter your username
auth_error_login_message = Use your school username or temporary code.

auth_error_password_title = Enter your password
auth_error_password_message = Enter the password for this account.

auth_error_forbidden_title = Access restricted
auth_error_forbidden_message = Contact your school to check access to this account.

auth_error_timeout_title = The server took too long
auth_error_timeout_message = Try again in a moment.

auth_error_server_title = Diary temporarily unavailable
auth_error_server_message = Please try again later.

auth_error_network_title = Couldn’t connect
auth_error_network_message = Check your internet connection and try again.

auth_error_credentials_title = Check your sign-in details
auth_error_credentials_message = The username or password is incorrect. You can edit them and try again.

auth_error_generic_title = Couldn’t sign in
auth_error_generic_message = Your details are still here. Please try again.

# Shared messages and connection status
notice_dismiss = Got it
notice_retry = Try again
notice_error_title = Couldn’t complete the action
notice_error_message = Please try again. If the problem continues, check your internet connection.
notice_success_title = Done
notice_password_updated = Your password has been updated. Use the new password the next time you sign in.
notice_pin_title = Set up a PIN first
notice_pin_message = Set up a PIN before enabling Face ID or Touch ID sign-in.
status_connecting_title = Updating data
status_connecting_message = Loading data from the school diary.
status_offline_title = Server unreachable
status_error_title = Connection error
status_error_message = Couldn’t connect to the server. Saved data may be out of date.
status_ready_title = Data updated
status_ready_message = Connected. The latest downloaded data is available.

# Credential keyboard diagnostics
autofill_diagnostics_title = Input diagnostics
autofill_diagnostics_hint = After reproducing the Passwords issue, copy the log. It contains only field and keyboard settings, without your username or password.
autofill_diagnostics_copy = Copy input diagnostics
autofill_diagnostics_empty = No entries yet. Open the sign-in form and switch keyboard layouts.
autofill_diagnostics_copied = Diagnostics copied

permission_biometrics = Use biometrics to unlock your account
permission_notifications = To notify you when grades change
