use super::*;

#[test]
fn find_free_port_returns_valid_port() {
    let port = find_free_port().unwrap();
    assert!(port > 0);
    let addr = format!("127.0.0.1:{port}");
    let _listener = std::net::TcpListener::bind(&addr).unwrap();
}
