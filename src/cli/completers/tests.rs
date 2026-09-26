use std::ffi::OsStr;

use serial_test::serial;

use super::*;

struct EnvGuard {
    xdg_original: Option<String>,
    twc_token_original: Option<String>
}

impl EnvGuard {
    fn set(xdg: &std::path::Path) -> Self {
        let xdg_original = std::env::var("XDG_CONFIG_HOME").ok();
        let twc_token_original = std::env::var("TWC_TOKEN").ok();

        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", xdg);
            std::env::remove_var("TWC_TOKEN");
        }

        Self {
            xdg_original,
            twc_token_original
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        unsafe {
            match &self.xdg_original {
                Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
                None => std::env::remove_var("XDG_CONFIG_HOME")
            }
            match &self.twc_token_original {
                Some(v) => std::env::set_var("TWC_TOKEN", v),
                None => std::env::remove_var("TWC_TOKEN")
            }
        }
    }
}

#[test]
fn complete_app_returns_empty_for_non_utf8() {
    let result = complete_app(OsStr::new("\u{FFFD}"));
    assert!(result.is_empty());
}

#[test]
#[serial]
fn complete_app_returns_empty_without_token() {
    let dir = tempfile::tempdir().unwrap();
    let _guard = EnvGuard::set(dir.path());

    let result = complete_app(OsStr::new("app"));
    assert!(result.is_empty());
}

#[test]
#[serial]
fn complete_app_returns_empty_for_empty_prefix() {
    let dir = tempfile::tempdir().unwrap();
    let _guard = EnvGuard::set(dir.path());

    let result = complete_app(OsStr::new(""));
    assert!(result.is_empty());
}
