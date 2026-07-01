use std::sync::Arc;

use keyring_core::CredentialStore;
use serial_test::serial;

use super::*;

/// Swaps the process-global keyring store for a test double and restores the
/// previous store on drop.
///
/// The keyring crate installs its platform store lazily on the first
/// `Entry::new` via a `Once`, so each constructor first primes that `Once`
/// (otherwise the real store would overwrite our test store mid-test).
struct KeyringGuard {
    previous: Option<Arc<CredentialStore>>
}

impl KeyringGuard {
    fn prime_once() {
        let _ = keyring::Entry::new("twc-rs-test-prime", "prime");
    }

    fn with_mock() -> Self {
        Self::prime_once();
        let previous = keyring_core::get_default_store();
        keyring_core::set_default_store(keyring_core::mock::Store::new().unwrap());
        Self {
            previous
        }
    }

    fn without_store() -> Self {
        Self::prime_once();
        let previous = keyring_core::unset_default_store();
        Self {
            previous
        }
    }
}

impl Drop for KeyringGuard {
    fn drop(&mut self) {
        match self.previous.take() {
            Some(store) => keyring_core::set_default_store(store),
            None => {
                keyring_core::unset_default_store();
            }
        }
    }
}

#[test]
#[serial]
fn save_and_load_via_keyring() {
    let _guard = KeyringGuard::with_mock();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");

    save_token("kr-token", &path).unwrap();
    assert!(
        !path.exists(),
        "token should be stored in keyring, not the config file"
    );
    assert_eq!(load_token(&path).unwrap(), "kr-token");

    delete_token(&path).unwrap();
    assert!(load_token(&path).is_err());
}

#[test]
#[serial]
fn delete_via_keyring_then_error_when_empty() {
    let _guard = KeyringGuard::with_mock();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");

    let err = delete_token(&path).unwrap_err();
    assert!(err.to_string().contains("no token found to delete"));
}

#[test]
#[serial]
fn save_and_load_from_config_fallback() {
    let _guard = KeyringGuard::without_store();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");

    save_token("cfg-token", &path).unwrap();
    assert!(
        path.exists(),
        "keyring is unavailable, so config must be used"
    );
    assert_eq!(load_token(&path).unwrap(), "cfg-token");
}

#[test]
#[serial]
fn delete_token_removes_from_config_fallback() {
    let _guard = KeyringGuard::without_store();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");

    save_token("cfg-token", &path).unwrap();
    delete_token(&path).unwrap();
    assert!(load_token(&path).is_err());
}

#[test]
#[serial]
fn delete_token_returns_error_when_empty() {
    let _guard = KeyringGuard::without_store();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");

    let result = delete_token(&path);
    assert!(result.is_err());
}
