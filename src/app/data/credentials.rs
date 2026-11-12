//! One credential-storage API for every platform supported by Day.

use day_part_keychain::{KeychainError, Protection, Secret};

/// Save a credential in the platform's secure store.
pub fn save(key: &str, value: &str) {
    let secret = Secret::from(value);
    if let Err(error) = day_part_keychain::set(key, &secret, Protection::WhenUnlocked) {
        log::error!("[SecureStore] Could not save credential: {error:?}");
    }
}

/// Load a credential from the platform's secure store.
pub fn load(key: &str) -> Option<String> {
    match day_part_keychain::get(key) {
        Ok(secret) => secret.as_str().map(str::to_owned),
        Err(KeychainError::NotFound) => None,
        Err(error) => {
            log::error!("[SecureStore] Could not load credential: {error:?}");
            None
        }
    }
}

/// Delete a credential from the platform's secure store.
pub fn delete(key: &str) {
    match day_part_keychain::remove(key) {
        Ok(()) | Err(KeychainError::NotFound) => {}
        Err(error) => log::error!("[SecureStore] Could not delete credential: {error:?}"),
    }
}
