use super::*;

#[test]
fn find_free_port_returns_valid_port() {
    let port = find_free_port().unwrap();
    assert!(port > 0);
    let addr = format!("127.0.0.1:{port}");
    let _listener = std::net::TcpListener::bind(&addr).unwrap();
}

#[test]
fn mask_token_short_token_is_fully_masked() {
    let token = "abc";
    let masked = mask_token(token);
    assert_eq!(masked, "***");
}

#[test]
fn mask_token_exact_length_8_is_fully_masked() {
    let token = "12345678";
    let masked = mask_token(token);
    assert_eq!(masked, "********");
}

#[test]
fn mask_token_long_token_shows_first_and_last_4_chars() {
    let token = "abcdefghijklmnop";
    let masked = mask_token(token);
    assert_eq!(masked, "abcd***mnop");
}

#[test]
fn mask_token_empty_string_returns_empty() {
    let token = "";
    let masked = mask_token(token);
    assert_eq!(masked, "");
}
