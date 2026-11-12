use eschool_api::entities::{TimeActivity, WeekActivity};

pub(super) fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Keep school-year boundaries supplied by the API; calendar years differ
/// from academic years between January and August.
pub(super) fn academic_year(year: u32, month: u32) -> (u32, u32) {
    let start = if month < 9 {
        year.saturating_sub(1)
    } else {
        year
    };
    (start, start + 1)
}

/// Assign weeks using actual period IDs/dates, never an assumed nine-week term.
pub(super) fn week_quarters(
    weeks: &[WeekActivity],
    periods: &[TimeActivity],
) -> Vec<Option<usize>> {
    let mut quarters: Vec<_> = periods
        .iter()
        .filter(|p| {
            let kind = p
                .activity_type
                .as_deref()
                .unwrap_or_default()
                .to_uppercase();
            let name =
                format!("{} {}", p.title, p.summary.as_deref().unwrap_or_default()).to_lowercase();
            kind.contains("QUARTER")
                || (!kind.contains("VACATION")
                    && !name.contains("каникул")
                    && (name.contains("четверт")
                        || name.contains("чвэрц")
                        || name.contains("quarter")))
        })
        .collect();
    quarters.sort_by_key(|p| {
        p.start_ts
            .or(p.start_date)
            .map(normalize_ts)
            .or_else(|| {
                weeks
                    .iter()
                    .filter(|w| w.time_activity_id == p.uuid)
                    .map(|w| normalize_ts(w.start_ts))
                    .min()
            })
            .unwrap_or(u64::MAX)
    });
    if quarters.len() == 4 {
        return weeks
            .iter()
            .map(|w| {
                quarters
                    .iter()
                    .position(|p| p.uuid == w.time_activity_id)
                    .or_else(|| {
                        let start = normalize_ts(w.start_ts);
                        quarters.iter().position(|p| {
                            match (p.start_ts.or(p.start_date), p.end_ts.or(p.end_date)) {
                                (Some(a), Some(b)) => {
                                    normalize_ts(a) <= start && start <= normalize_ts(b)
                                }
                                _ => false,
                            }
                        })
                    })
            })
            .collect();
    }
    // Some API versions expose only the period ID on each week. Four distinct
    // parents ordered by their first week also describe the academic quarters.
    let mut parents: Vec<(String, u64)> = Vec::new();
    for w in weeks {
        if w.time_activity_id.is_empty() {
            continue;
        }
        if let Some(p) = parents.iter_mut().find(|p| p.0 == w.time_activity_id) {
            p.1 = p.1.min(normalize_ts(w.start_ts));
        } else {
            parents.push((w.time_activity_id.clone(), normalize_ts(w.start_ts)));
        }
    }
    parents.sort_by_key(|p| p.1);
    weeks
        .iter()
        .map(|w| {
            if parents.len() == 4 {
                parents.iter().position(|p| p.0 == w.time_activity_id)
            } else {
                None
            }
        })
        .collect()
}

/// School API timestamps may arrive as seconds or milliseconds — everything
/// compares in ms, so anything below ~2001-09 in magnitude is seconds.
pub(super) fn normalize_ts(ts: u64) -> u64 {
    if ts < 1_000_000_000_000 {
        ts * 1000
    } else {
        ts
    }
}

/// The week the app opens on: the one containing `now_ms`; when none does
/// (a gap between school years, a stale cache), the most recent week that has
/// already started — never a silent fall back to week 0.
pub(super) fn find_current_week(weeks: &[WeekActivity], now_ms: u64) -> Option<usize> {
    weeks
        .iter()
        .position(|w| {
            let start = normalize_ts(w.start_ts);
            let end = normalize_ts(w.end_ts);
            start <= now_ms && now_ms <= end
        })
        .or_else(|| {
            weeks
                .iter()
                .enumerate()
                .filter(|(_, w)| normalize_ts(w.start_ts) <= now_ms)
                .map(|(i, _)| i)
                .last()
        })
}
