use super::*;

#[test]
fn fmt_id_rounds_correctly() {
    assert_eq!(fmt_id(1.0), "1");
    assert_eq!(fmt_id(42.7), "43");
    assert_eq!(fmt_id(0.4), "0");
}

#[test]
fn fingerprint_short_key() {
    let fp = fingerprint("ssh-rsa AAAA");
    assert_eq!(fp, "AAAA");
}

#[test]
fn fingerprint_empty_body() {
    let fp = fingerprint("");
    assert_eq!(fp, "-");
}

#[test]
fn fingerprint_single_word() {
    let fp = fingerprint("only-one-word");
    assert_eq!(fp, "-");
}

#[test]
fn fingerprint_long_key() {
    let key_body = "AAAA".repeat(20);
    let key = format!("ssh-rsa {key_body} user@host");
    let fp = fingerprint(&key);
    let parts: Vec<&str> = fp.split(':').collect();
    assert_eq!(parts.len(), 16);
    for part in &parts {
        assert_eq!(part.len(), 2);
        u8::from_str_radix(part, 16).expect("should be valid hex");
    }
}

#[test]
fn fingerprint_odd_length_base64() {
    let key = "ssh-rsa AABBB user@host";
    let fp = fingerprint(&key);
    assert!(!fp.is_empty());
}

#[test]
fn ssh_key_row_display() {
    let row = SshKeyRow {
        id:          "10".to_string(),
        name:        "dev-key".to_string(),
        fingerprint: "aa:bb:cc".to_string(),
        is_default:  "true".to_string(),
    };
    let display = row.to_string();
    assert!(display.contains("10"));
    assert!(display.contains("dev-key"));
    assert!(display.contains("aa:bb:cc"));
    assert!(display.contains("true"));
}

#[tokio::test]
async fn list_with_fake_token_returns_api_error() {
    let config = timeweb_rs::authenticated("fake-token");
    let result = list(&config, OutputFormat::Table).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn list_json_format_fake_token() {
    let config = timeweb_rs::authenticated("fake-token");
    let result = list(&config, OutputFormat::Json).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn list_quiet_format_fake_token() {
    let config = timeweb_rs::authenticated("fake-token");
    let result = list(&config, OutputFormat::Quiet).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn add_with_fake_token_returns_error() {
    let config = timeweb_rs::authenticated("fake-token");
    let result = add(&config, "my-key", None, false).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn add_with_file_fake_token() {
    let dir = tempfile::tempdir().unwrap();
    let key_path = dir.path().join("id_rsa.pub");
    std::fs::write(&key_path, "ssh-rsa AAAA test@host").unwrap();
    let config = timeweb_rs::authenticated("fake-token");
    let result = add(&config, "my-key", Some(key_path.to_str().unwrap()), false).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn add_with_nonexistent_file() {
    let config = timeweb_rs::authenticated("fake-token");
    let result = add(&config, "my-key", Some("/nonexistent/path"), false).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn delete_with_fake_token_returns_api_error() {
    let config = timeweb_rs::authenticated("fake-token");
    let result = delete(&config, 42).await;
    assert!(result.is_err());
}
