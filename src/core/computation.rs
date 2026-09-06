use crate::model::KINDS;

pub(crate) fn item_summary(items: &[crate::model::Item]) -> (usize, usize, usize) {
    let total = items.len();
    let done = items.iter().filter(|i| i.done).count();
    let pending = total - done;
    (total, done, pending)
}

pub(crate) fn kind_name(kind: usize) -> &'static str {
    KINDS.get(kind).copied().unwrap_or("Unknown")
}

pub(crate) fn average_rating(items: &[crate::model::Item]) -> f64 {
    if items.is_empty() {
        return 0.0;
    }
    let sum: u32 = items.iter().map(|i| i.rating as u32).sum();
    sum as f64 / items.len() as f64
}
