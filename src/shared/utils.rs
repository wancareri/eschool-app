//! Date formatting, grade colors, and other utility functions.

use day::prelude::Color;

use crate::shared::colors;

/// Short weekday label from a 1-based day index (1 = Monday).
pub fn weekday_short(dow: u32) -> &'static str {
    match dow {
        1 => "Пн",
        2 => "Вт",
        3 => "Ср",
        4 => "Чт",
        5 => "Пт",
        6 => "Сб",
        _ => "Вс",
    }
}

/// Weekday name from a 1-based day index (1 = Monday).
pub fn weekday_name(dow: u32) -> &'static str {
    match dow {
        1 => "Понедельник",
        2 => "Вторник",
        3 => "Среда",
        4 => "Четверг",
        5 => "Пятница",
        6 => "Суббота",
        _ => "Воскресенье",
    }
}

/// Render a timestamp (epoch millis) as "D.MM".
pub fn format_date_short(ts: u64) -> String {
    // e-schools API timestamps represent midnight in Europe/Minsk (UTC+3, offset +10800s).
    // In UTC, this is 21:00 of the preceding day. Adding 12 hours (43_200_000 ms) lands squarely
    // in midday of the intended calendar day regardless of local timezone/UTC offset.
    let epoch_days = ((ts + 12 * 3600 * 1000) / 86_400_000) as i64;
    let d = day_piece_datetime::DayDate::from_epoch_days(epoch_days);
    format!("{}.{:02}", d.day, d.month)
}

/// Full header: "Понедельник, 07.09".
pub fn format_date_header(dow: u32, ts: u64) -> String {
    let epoch_days = ((ts + 12 * 3600 * 1000) / 86_400_000) as i64;
    let calc_dow = ((epoch_days + 3) % 7 + 1) as u32;
    let dow_name = weekday_name(if (1..=7).contains(&calc_dow) { calc_dow } else { dow });
    format!("{}, {}", dow_name, format_date_short(ts))
}

/// Current weekday as a 1-based index (1 = Monday), computed from the system clock
/// using the same Europe/Minsk day-rollover convention as the timestamp formatters.
pub fn today_dow() -> u32 {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let epoch_days = ((secs + 12 * 3600) / 86_400) as i64;
    (((epoch_days + 3) % 7 + 1) as u32).clamp(1, 7)
}

pub fn grade_color(mark: &str) -> Color {
    match mark.trim().parse::<u32>() {
        Ok(9..=10) => colors::GRADE_EXCELLENT,
        Ok(7..=8)  => colors::GRADE_GOOD,
        Ok(5..=6)  => colors::GRADE_SATISFACTORY,
        Ok(3..=4)  => colors::GRADE_POOR,
        Ok(1..=2)  => colors::GRADE_FAILING,
        _          => colors::GRAY_400,
    }
}

pub fn avg_grade_color(avg: f64) -> Color {
    if avg >= 8.5 {
        colors::GRADE_EXCELLENT
    } else if avg >= 6.5 {
        colors::GRADE_GOOD
    } else if avg >= 4.5 {
        colors::GRADE_SATISFACTORY
    } else if avg >= 2.5 {
        colors::GRADE_POOR
    } else {
        colors::GRADE_FAILING
    }
}

pub fn parse_marks(mark_str: &str) -> Vec<f64> {
    let mut result = Vec::new();
    for part in mark_str.split(|c: char| c == '/' || c == ',' || c == ';' || c == ' ' || c == '(' || c == ')') {
        let trimmed = part.trim();
        if let Ok(val) = trimmed.parse::<f64>() {
            if val >= 1.0 && val <= 100.0 {
                result.push(val);
            }
        }
    }
    result
}

