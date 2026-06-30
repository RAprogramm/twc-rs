use super::*;

#[test]
fn fmt_id_formats_integers() {
    assert_eq!(fmt_id(1_i64), "1");
    assert_eq!(fmt_id(1_873_345_i64), "1873345");
}

#[test]
fn project_row_display() {
    let row = ProjectRow {
        id:          "5".to_string(),
        name:        "web-app".to_string(),
        description: "Production".to_string(),
        is_default:  "false".to_string()
    };
    let display = row.to_string();
    assert!(display.contains('5'));
    assert!(display.contains("web-app"));
    assert!(display.contains("Production"));
    assert!(display.contains("false"));
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
async fn create_with_fake_token_returns_api_error() {
    let config = timeweb_rs::authenticated("fake-token");
    let result = create(&config, "my-project", None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn create_with_description_fake_token() {
    let config = timeweb_rs::authenticated("fake-token");
    let result = create(&config, "my-project", Some("description")).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn delete_with_fake_token_returns_api_error() {
    let config = timeweb_rs::authenticated("fake-token");
    let result = delete(&config, 42).await;
    assert!(result.is_err());
}
