
use day_part_keychain::{KeychainError, Protection, Secret};

pub fn save(key: &str, value: &str) {
    let secret = Secret::from(value);
    if let Err(error) = day_part_keychain::set(key, &secret, Protection::WhenUnlocked) {
        log::error!("[SecureStore] Could not save credential: {error:?}");
    }
}

pub fn load(key: &str) -> Option<String> {
    match day_part_keychain::get(key) {
        Ok(secret) => secret.as_str().map(str::to_owned),
        Err(error) if is_missing(&error) => None,
        Err(error) => {
            log::error!("[SecureStore] Could not load credential: {error:?}");
            None
        }
    }
}

/// libsecret can report a missing item as a generic platform error instead of
/// `KeychainError::NotFound`. A first launch has no credential by design, so
/// treat that exact response as an unsigned-in state rather than an error.
fn is_missing(error: &KeychainError) -> bool {
    matches!(error, KeychainError::NotFound)
        || matches!(error, KeychainError::Platform(message)
            if message.to_ascii_lowercase().contains("no result found"))
}

pub fn delete(key: &str) {
    match day_part_keychain::remove(key) {
        Ok(()) | Err(KeychainError::NotFound) => {}
        Err(error) => log::error!("[SecureStore] Could not delete credential: {error:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::is_missing;
    use day_part_keychain::KeychainError;

    #[test]
    fn linux_secret_service_empty_result_is_not_a_credential_failure() {
        assert!(is_missing(&KeychainError::NotFound));
        assert!(is_missing(&KeychainError::Platform(
            "Secret Service: no result found".into()
        )));
        assert!(!is_missing(&KeychainError::Platform(
            "Secret Service: permission denied".into()
        )));
    }
}
