//! PIN code lock — SHA-256 hashing, save/load/verify via Keychain.

use crate::shared::{nslog, secure};
use sha2::{Digest, Sha256};

const PIN_HASH_KEY: &str = "auth.pin_hash";
const PIN_SALT_KEY: &str = "auth.pin_salt";
const PIN_ENABLED_KEY: &str = "auth.pin_enabled";
const LAST_BG_KEY: &str = "app.last_background";
const LOCK_TIMEOUT_SECS: u64 = 300; // 5 minutes

/// Check if PIN lock is enabled.
pub fn is_enabled() -> bool {
    secure::load(PIN_ENABLED_KEY)
        .map(|v| v == "true")
        .unwrap_or(false)
}

/// Save a 4-digit PIN (hashes with random salt).
pub fn save_pin(pin: &str) {
    let salt = random_salt();
    let hash = hash_pin(pin, &salt);
    secure::save(PIN_HASH_KEY, &hash);
    secure::save(PIN_SALT_KEY, &salt);
    secure::save(PIN_ENABLED_KEY, "true");
    nslog::nslog("[PIN] Saved");
}

/// Delete PIN from secure storage.
pub fn delete_pin() {
    secure::delete(PIN_HASH_KEY);
    secure::delete(PIN_SALT_KEY);
    secure::save(PIN_ENABLED_KEY, "false");
    nslog::nslog("[PIN] Deleted");
}

/// Verify a PIN input against the stored hash.
pub fn verify(pin: &str) -> bool {
    let stored_hash = match secure::load(PIN_HASH_KEY) {
        Some(h) => h,
        None => return false,
    };
    let salt = match secure::load(PIN_SALT_KEY) {
        Some(s) => s,
        None => return false,
    };
    let computed = hash_pin(pin, &salt);
    let ok = computed == stored_hash;
    nslog::nslog(&format!("[PIN] Verify: {ok}"));
    ok
}

// ── Auto-lock ───────────────────────────────────────────────────────────

/// Save current timestamp as "last active" (call when app enters background).
pub fn save_last_background() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    day::prefs::set(LAST_BG_KEY, &now.to_string());
    nslog::nslog(&format!("[PIN] Background saved: {now}"));
}

/// Check if auto-lock should trigger (>5 min since last background).
pub fn should_auto_lock() -> bool {
    let last = day::prefs::get(LAST_BG_KEY)
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let elapsed = now.saturating_sub(last);
    let should_lock = elapsed > LOCK_TIMEOUT_SECS;
    if should_lock {
        nslog::nslog(&format!("[PIN] Auto-lock: elapsed {elapsed}s > {LOCK_TIMEOUT_SECS}s"));
    }
    should_lock
}

// ── Hashing ─────────────────────────────────────────────────────────────

/// Generate a random 16-byte hex salt.
fn random_salt() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..16).map(|_| rng.r#gen()).collect();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// SHA-256(pin + salt) → hex string.
fn hash_pin(pin: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(pin.as_bytes());
    hasher.update(salt.as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{b:02x}")).collect()
}
