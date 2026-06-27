use super::*;

#[test]
fn display_config_not_found() {
    let err = TwcError::ConfigNotFound("/tmp/test.toml".to_string());
    assert_eq!(err.to_string(), "config not found: /tmp/test.toml");
}

#[test]
fn display_config_parse() {
    let err = TwcError::ConfigParse("bad toml".to_string());
    assert_eq!(err.to_string(), "failed to parse config: bad toml");
}

#[test]
fn display_config_write() {
    let err = TwcError::ConfigWrite("disk full".to_string());
    assert_eq!(err.to_string(), "failed to write config: disk full");
}

#[test]
fn display_token_missing() {
    let err = TwcError::TokenMissing;
    assert_eq!(
        err.to_string(),
        "no API token configured; run `twc-rs config set-token`"
    );
}

#[test]
fn display_api() {
    let err = TwcError::Api("timeout".to_string());
    assert_eq!(err.to_string(), "API error: timeout");
}

#[test]
fn display_io() {
    let err = TwcError::Io("permission denied".to_string());
    assert_eq!(err.to_string(), "I/O error: permission denied");
}

#[test]
fn from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
    let twc_err: TwcError = io_err.into();
    assert!(twc_err.to_string().contains("file missing"));
}

#[test]
fn from_toml_de_error() {
    let toml_err = toml::from_str::<super::super::config::AppConfig>("not valid = [").unwrap_err();
    let twc_err: TwcError = toml_err.into();
    assert!(twc_err.to_string().contains("failed to parse config"));
}

#[test]
fn from_toml_de_error_invalid_syntax() {
    let toml_err = toml::from_str::<super::super::config::AppConfig>("= = =").unwrap_err();
    let twc_err: TwcError = toml_err.into();
    assert!(twc_err.to_string().contains("failed to parse config"));
}

#[test]
fn from_toml_ser_error() {
    use serde::Serialize;
    #[derive(Serialize)]
    struct NanHolder(f64);
    let holder = NanHolder(f64::NAN);
    let ser_err = toml::to_string(&holder).unwrap_err();
    let twc_err: TwcError = ser_err.into();
    assert!(twc_err.to_string().contains("failed to write config"));
}

#[test]
fn from_timeweb_serde_error() {
    let serde_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
    let api_err: timeweb_rs::apis::Error<String> = timeweb_rs::apis::Error::Serde(serde_err);
    let twc_err: TwcError = api_err.into();
    assert!(twc_err.to_string().contains("API error"));
}

#[test]
fn from_timeweb_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::Other, "timeweb io");
    let api_err: timeweb_rs::apis::Error<String> = timeweb_rs::apis::Error::Io(io_err);
    let twc_err: TwcError = api_err.into();
    assert!(twc_err.to_string().contains("I/O error"));
}

#[test]
fn from_timeweb_response_error_with_entity() {
    use timeweb_rs::apis::ResponseContent;
    let content = ResponseContent::<String> {
        status:  reqwest::StatusCode::UNAUTHORIZED,
        content: String::new(),
        entity:  Some("unauthorized".to_string())
    };
    let api_err: timeweb_rs::apis::Error<String> = timeweb_rs::apis::Error::ResponseError(content);
    let twc_err: TwcError = api_err.into();
    assert!(twc_err.to_string().contains("API error"));
    assert!(twc_err.to_string().contains("unauthorized"));
}

#[test]
fn from_timeweb_response_error_without_entity() {
    use timeweb_rs::apis::ResponseContent;
    let content = ResponseContent::<String> {
        status:  reqwest::StatusCode::NOT_FOUND,
        content: String::new(),
        entity:  None
    };
    let api_err: timeweb_rs::apis::Error<String> = timeweb_rs::apis::Error::ResponseError(content);
    let twc_err: TwcError = api_err.into();
    assert!(twc_err.to_string().contains("API error"));
    assert!(twc_err.to_string().contains("404"));
}

#[test]
fn error_trait_is_implemented() {
    let err = TwcError::TokenMissing;
    let _dyn_err: &dyn std::error::Error = &err;
}

#[test]
fn error_source_is_none_for_response_error() {
    use timeweb_rs::apis::ResponseContent;
    let content = ResponseContent::<String> {
        status:  reqwest::StatusCode::BAD_REQUEST,
        content: String::new(),
        entity:  None
    };
    let api_err: timeweb_rs::apis::Error<String> = timeweb_rs::apis::Error::ResponseError(content);
    let twc_err: TwcError = api_err.into();
    let dyn_err: &dyn std::error::Error = &twc_err;
    assert!(dyn_err.source().is_none());
}
