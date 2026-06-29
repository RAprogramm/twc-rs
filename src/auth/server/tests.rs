use super::*;

fn find_free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

#[test]
fn serve_once_receives_token() {
    use std::{
        io::{Read, Write},
        sync::mpsc
    };

    let (tx, rx) = mpsc::sync_channel(1);
    let port = find_free_port();

    let handle = std::thread::spawn(move || {
        serve_once(port, tx).unwrap();
    });

    // Small delay to let server start
    std::thread::sleep(std::time::Duration::from_millis(50));

    let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
    let request = "POST /token HTTP/1.1\r\n\
                    Host: localhost\r\n\
                    Connection: close\r\n\
                    Content-Length: 16\r\n\
                    \r\n\
                    my-api-token-123";
    stream.write_all(request.as_bytes()).unwrap();

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    assert!(response.contains("200"));

    let token = rx.recv().unwrap();
    assert_eq!(token, "my-api-token-123");

    handle.join().unwrap();
}

#[test]
fn serve_once_serves_html() {
    use std::{
        io::{Read, Write},
        sync::mpsc
    };

    let (tx, _rx) = mpsc::sync_channel(1);
    let port = find_free_port();

    let handle = std::thread::spawn(move || {
        let _ = serve_once(port, tx);
    });

    std::thread::sleep(std::time::Duration::from_millis(50));

    let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
    let request = "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    stream.write_all(request.as_bytes()).unwrap();

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    assert!(response.contains("twc-rs"));

    let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
    let request = "POST /token HTTP/1.1\r\n\
                    Host: localhost\r\n\
                    Connection: close\r\n\
                    Content-Length: 8\r\n\
                    \r\n\
                    shutdown";
    stream.write_all(request.as_bytes()).unwrap();

    handle.join().ok();
}
