use day::prelude::*;

/// `#RRGGBB` → color.
#[allow(clippy::ptr_arg)]
pub fn color_of(s: &String) -> Color {
    let h = s.trim_start_matches('#');
    u32::from_str_radix(h, 16)
        .ok()
        .filter(|_| h.len() == 6)
        .map(Color::hex)
        .unwrap_or(Color::hex(0x3B82F6))
}

/// color → `#RRGGBB`.
pub fn hex_of(c: &Color) -> String {
    let (r, g, b) = (
        (c.r * 255.0).round() as u8,
        (c.g * 255.0).round() as u8,
        (c.b * 255.0).round() as u8,
    );
    format!("#{r:02X}{g:02X}{b:02X}")
}

/// ISO string → DayDate.
#[allow(clippy::ptr_arg)]
pub fn date_of(s: &String) -> day_piece_datetime::DayDate {
    day_piece_datetime::DayDate::parse_iso(s).unwrap_or_else(|| {
        debug!("date {s:?} is not ISO-8601 — showing today");
        day_piece_datetime::DayDate::today()
    })
}

/// DayDate → ISO string.
pub fn iso_of(d: &day_piece_datetime::DayDate) -> String {
    format!("{:04}-{:02}-{:02}", d.year, d.month, d.day)
}
