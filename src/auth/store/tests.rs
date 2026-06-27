use super::*;

#[test]
fn save_and_load_from_config() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");

    save_token("test-token-abc", &path).unwrap();
    let loaded = load_token(&path).unwrap();
    assert_eq!(loaded, "test-token-abc");
}

#[test]
fn delete_token_removes_from_config() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");

    save_token("test-token-abc", &path).unwrap();
    delete_token(&path).unwrap();
    let result = load_token(&path);
    assert!(result.is_err());
}

#[test]
fn delete_token_returns_error_when_empty() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");

    let result = delete_token(&path);
    assert!(result.is_err());
}
