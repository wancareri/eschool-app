cat src/pages/settings.rs | sed -e '/picker(/,/trailing: 20.0 }),/c\
        settings_tabs(current_tab.clone()),' > src/pages/settings.rs.tmp
mv src/pages/settings.rs.tmp src/pages/settings.rs
