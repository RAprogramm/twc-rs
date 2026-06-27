use super::*;

#[test]
fn fmt_id_rounds_correctly() {
    assert_eq!(fmt_id(1.0), "1");
    assert_eq!(fmt_id(42.7), "43");
    assert_eq!(fmt_id(0.4), "0");
}

#[test]
fn server_row_display() {
    let row = ServerRow {
        id:       "1".to_string(),
        name:     "test-server".to_string(),
        status:   "Active".to_string(),
        cpu:      "2".to_string(),
        ram:      "4096".to_string(),
        os:       "Ubuntu 22.04".to_string(),
        location: "msk".to_string(),
    };
    let display = row.to_string();
    assert!(display.contains("1"));
    assert!(display.contains("test-server"));
    assert!(display.contains("Active"));
    assert!(display.contains("2"));
    assert!(display.contains("4096"));
    assert!(display.contains("Ubuntu 22.04"));
    assert!(display.contains("msk"));
}

#[tokio::test]
async fn list_with_fake_token_returns_api_error() {
    let config = timeweb_rs::authenticated("fake-token");
    let result = list(&config, None, None, OutputFormat::Table).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn list_json_format_fake_token() {
    let config = timeweb_rs::authenticated("fake-token");
    let result = list(&config, None, None, OutputFormat::Json).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn list_quiet_format_fake_token() {
    let config = timeweb_rs::authenticated("fake-token");
    let result = list(&config, None, None, OutputFormat::Quiet).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn list_with_limit_offset_fake_token() {
    let config = timeweb_rs::authenticated("fake-token");
    let result = list(&config, Some(10), Some(5), OutputFormat::Table).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn info_with_fake_token_returns_api_error() {
    let config = timeweb_rs::authenticated("fake-token");
    let result = info(&config, 42, OutputFormat::Table).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn info_json_format_fake_token() {
    let config = timeweb_rs::authenticated("fake-token");
    let result = info(&config, 1, OutputFormat::Json).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn info_quiet_format_fake_token() {
    let config = timeweb_rs::authenticated("fake-token");
    let result = info(&config, 1, OutputFormat::Quiet).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn delete_with_fake_token_returns_api_error() {
    let config = timeweb_rs::authenticated("fake-token");
    let result = delete(&config, 42).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn reboot_with_fake_token_returns_api_error() {
    let config = timeweb_rs::authenticated("fake-token");
    let result = reboot(&config, 42).await;
    assert!(result.is_err());
}
