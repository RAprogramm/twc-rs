use super::*;

#[test]
fn auth_error_display_browser_failed() {
    let err = AuthError::BrowserFailed;
    assert!(err.to_string().contains("could not open browser"));
}

#[test]
fn auth_error_display_timeout() {
    let err = AuthError::Timeout;
    assert!(err.to_string().contains("timed out"));
}

#[test]
fn auth_error_display_invalid_token() {
    let err = AuthError::InvalidToken;
    assert!(err.to_string().contains("invalid or expired"));
}

#[test]
fn auth_error_display_store_failed() {
    let err = AuthError::StoreFailed("disk full".to_string());
    assert!(err.to_string().contains("disk full"));
}

#[test]
fn auth_error_display_network() {
    let err = AuthError::Network("connection refused".to_string());
    assert!(err.to_string().contains("connection refused"));
}

#[test]
fn auth_error_display_server() {
    let err = AuthError::Server("bind failed".to_string());
    assert!(err.to_string().contains("bind failed"));
}

#[test]
fn auth_error_is_std_error() {
    let err: Box<dyn std::error::Error> =
        Box::new(AuthError::Timeout);
    assert!(err.to_string().contains("timed out"));
}
