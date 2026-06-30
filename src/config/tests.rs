use std::fs;

use serial_test::serial;

use super::*;

struct XdgGuard {
    original: Option<String>
}

impl XdgGuard {
    fn set(dir: &std::path::Path) -> Self {
        let original = std::env::var("XDG_CONFIG_HOME").ok();
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", dir);
        }
        Self {
            original
        }
    }
}

impl Drop for XdgGuard {
    fn drop(&mut self) {
        unsafe {
            match &self.original {
                Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
                None => std::env::remove_var("XDG_CONFIG_HOME")
            }
        }
    }
}

#[test]
fn default_config_has_no_token() {
    let cfg = AppConfig::default();
    assert!(cfg.token.is_none());
}

#[test]
fn path_returns_valid_toml_path() {
    let path = AppConfig::path().expect("config path should resolve");
    assert!(path.to_string_lossy().ends_with("config.toml"));
}

#[test]
fn path_contains_twc_rs_dir() {
    let path = AppConfig::path().expect("config path should resolve");
    let path_str = path.to_string_lossy();
    assert!(
        path_str.contains("twc-rs"),
        "Path should contain twc-rs dir: {path_str}"
    );
}

#[test]
#[serial]
fn load_returns_default_when_no_file() {
    let dir = tempfile::tempdir().unwrap();
    let _guard = XdgGuard::set(dir.path());
    let cfg = AppConfig::load().expect("load should succeed");
    assert!(cfg.token.is_none());
}

#[test]
#[serial]
fn load_returns_default_when_dir_exists_but_no_file() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join("twc-rs");
    fs::create_dir_all(&config_dir).unwrap();
    let _guard = XdgGuard::set(dir.path());
    let cfg = AppConfig::load().expect("load should succeed");
    assert!(cfg.token.is_none());
}

#[test]
#[serial]
fn load_parses_valid_toml_with_token() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join("twc-rs");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.toml"),
        "token = \"my-secret-token\"\n"
    )
    .unwrap();
    let _guard = XdgGuard::set(dir.path());
    let cfg = AppConfig::load().expect("load should succeed");
    assert_eq!(cfg.token.as_deref(), Some("my-secret-token"));
}

#[test]
#[serial]
fn load_returns_config_parse_error_for_invalid_toml() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join("twc-rs");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("config.toml"), "this is not [[valid toml").unwrap();
    let _guard = XdgGuard::set(dir.path());
    let err = AppConfig::load().unwrap_err();
    assert!(
        err.to_string().contains("failed to parse config"),
        "Expected parse error, got: {err}"
    );
}

#[test]
#[serial]
fn save_and_load_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let _guard = XdgGuard::set(dir.path());
    let cfg = AppConfig {
        token: Some("roundtrip-token".to_string()),
        ..AppConfig::default()
    };
    cfg.save().expect("save should succeed");
    let loaded = AppConfig::load().expect("load should succeed");
    assert_eq!(loaded.token.as_deref(), Some("roundtrip-token"));
    let path = AppConfig::path().unwrap();
    assert!(path.exists());
}

#[test]
#[serial]
fn save_creates_parent_directories() {
    let dir = tempfile::tempdir().unwrap();
    let _guard = XdgGuard::set(dir.path());
    let cfg = AppConfig {
        token: Some("test".to_string()),
        ..AppConfig::default()
    };
    cfg.save().expect("save should succeed");
    let path = AppConfig::path().unwrap();
    assert!(path.parent().unwrap().exists());
}

#[test]
#[serial]
fn load_without_token_field() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join("twc-rs");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("config.toml"), "").unwrap();
    let _guard = XdgGuard::set(dir.path());
    let cfg = AppConfig::load().expect("load should succeed");
    assert!(cfg.token.is_none());
}

#[test]
#[serial]
fn save_overwrites_existing_config() {
    let dir = tempfile::tempdir().unwrap();
    let _guard = XdgGuard::set(dir.path());
    let cfg1 = AppConfig {
        token: Some("first".to_string()),
        ..AppConfig::default()
    };
    cfg1.save().unwrap();
    let cfg2 = AppConfig {
        token: Some("second".to_string()),
        ..AppConfig::default()
    };
    cfg2.save().unwrap();
    let loaded = AppConfig::load().unwrap();
    assert_eq!(loaded.token.as_deref(), Some("second"));
}

#[test]
fn serialization_roundtrip_via_string() {
    let cfg = AppConfig {
        token: Some("my-secret-token".to_string()),
        ..AppConfig::default()
    };
    let toml_str = toml::to_string(&cfg).unwrap();
    let loaded: AppConfig = toml::from_str(&toml_str).unwrap();
    assert_eq!(loaded.token.as_deref(), Some("my-secret-token"));
}

#[test]
fn serialization_roundtrip_empty() {
    let cfg = AppConfig::default();
    let toml_str = toml::to_string(&cfg).unwrap();
    let loaded: AppConfig = toml::from_str(&toml_str).unwrap();
    assert!(loaded.token.is_none());
}

#[test]
fn deserialize_without_token_field() {
    let cfg: AppConfig = toml::from_str("").unwrap();
    assert!(cfg.token.is_none());
}

#[test]
fn deserialize_with_token() {
    let cfg: AppConfig = toml::from_str("token = \"abc123\"\n").unwrap();
    assert_eq!(cfg.token.as_deref(), Some("abc123"));
}

#[test]
fn deserialize_invalid_toml() {
    let result = toml::from_str::<AppConfig>("not [[valid");
    assert!(result.is_err());
}

#[test]
fn serialize_produces_valid_toml() {
    let cfg = AppConfig {
        token: Some("tok".to_string()),
        ..AppConfig::default()
    };
    let toml_str = toml::to_string(&cfg).unwrap();
    assert!(toml_str.contains("token"));
    assert!(toml_str.contains("tok"));
}

#[test]
fn serialize_skips_none_token() {
    let cfg = AppConfig::default();
    let toml_str = toml::to_string(&cfg).unwrap();
    assert!(!toml_str.contains("token"));
}

#[test]
fn save_and_load_roundtrip_filesystem() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("config.toml");
    let cfg = AppConfig {
        token: Some("roundtrip-token".to_string()),
        ..AppConfig::default()
    };
    let content = toml::to_string(&cfg).unwrap();
    fs::write(&file_path, content).unwrap();
    let read_back = fs::read_to_string(&file_path).unwrap();
    let loaded: AppConfig = toml::from_str(&read_back).unwrap();
    assert_eq!(loaded.token.as_deref(), Some("roundtrip-token"));
}

#[test]
fn save_creates_parent_directories_filesystem() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join("twc-rs");
    fs::create_dir_all(&config_dir).unwrap();
    let file_path = config_dir.join("config.toml");
    let cfg = AppConfig {
        token: Some("test".to_string()),
        ..AppConfig::default()
    };
    let content = toml::to_string(&cfg).unwrap();
    fs::write(&file_path, content).unwrap();
    assert!(file_path.exists());
    assert!(file_path.parent().unwrap().exists());
}

#[test]
fn load_from_filesystem_invalid_toml() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("config.toml");
    fs::write(&file_path, "this is not [[valid toml").unwrap();
    let content = fs::read_to_string(&file_path).unwrap();
    let result = toml::from_str::<AppConfig>(&content);
    assert!(result.is_err());
}

#[test]
fn deserialize_with_extra_fields_ignored() {
    let cfg: AppConfig = toml::from_str("token = \"abc\"\nother_field = true\n").unwrap();
    assert_eq!(cfg.token.as_deref(), Some("abc"));
}

#[test]
fn clone_config() {
    let cfg = AppConfig {
        token: Some("cloned".to_string()),
        ..AppConfig::default()
    };
    let cloned = cfg;
    assert_eq!(cloned.token.as_deref(), Some("cloned"));
}

#[test]
fn debug_format() {
    let cfg = AppConfig {
        token: Some("debug".to_string()),
        ..AppConfig::default()
    };
    let debug_str = format!("{cfg:?}");
    assert!(debug_str.contains("AppConfig"));
    assert!(debug_str.contains("debug"));
}
