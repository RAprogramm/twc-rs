// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Local HTTP server that serves a token paste page and receives the token.

use std::{sync::mpsc::SyncSender, time::Duration};

/// Minimal self-contained HTML page for token paste.
const TOKEN_PAGE: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>twc-rs auth</title>
  <style>
    body { font-family: monospace; max-width: 480px;
           margin: 60px auto; padding: 0 16px; }
    textarea { width: 100%; height: 80px; font-family: monospace; }
    button { margin-top: 8px; padding: 6px 20px; cursor: pointer; }
    #msg { margin-top: 12px; color: green; }
  </style>
</head>
<body>
  <h2>twc-rs — paste your API token</h2>
  <p>Get it from
      <a href="https://timeweb.cloud/my/api-keys" target="_blank">
        timeweb.cloud/my/api-keys
    </a>
  </p>
  <textarea id="t" placeholder="eyJ..."></textarea><br>
  <button onclick="save()">Save token</button>
  <div id="msg"></div>
  <script>
    async function save() {
      const r = await fetch('/token', {
        method:'POST', body: document.getElementById('t').value
      });
      if (r.ok) document.getElementById('msg').textContent =
        '✓ Token received. You can close this tab.';
    }
  </script>
</body>
</html>"#;

/// Starts a local HTTP server on the given port, serves the token page
/// once, and sends the received token through the channel.
///
/// # Errors
///
/// Returns `Err` when the server fails to bind or accept requests.
#[allow(clippy::needless_pass_by_value)]
pub fn serve_once(port: u16, tx: SyncSender<String>) -> Result<(), String> {
    let addr = format!("127.0.0.1:{port}");
    let server =
        tiny_http::Server::http(&addr).map_err(|e| format!("failed to bind {addr}: {e}"))?;

    let timeout = Duration::from_mins(5);
    let mut received = false;

    while let Ok(Some(mut request)) = server.recv_timeout(timeout) {
        match request.url() {
            "/" => {
                let response = tiny_http::Response::from_string(TOKEN_PAGE).with_header(
                    tiny_http::Header::from_bytes(
                        &b"Content-Type"[..],
                        &b"text/html; charset=utf-8"[..]
                    )
                    .expect("valid header")
                );
                let _ = request.respond(response);
            }
            "/token" => {
                let mut body = String::new();
                let _ = request.as_reader().read_to_string(&mut body);
                let token = body.trim().to_string();

                let response = tiny_http::Response::from_string("ok");
                let _ = request.respond(response);

                if !token.is_empty() {
                    let _ = tx.send(token);
                    received = true;
                    break;
                }
            }
            _ => {
                let response = tiny_http::Response::from_string("not found").with_status_code(404);
                let _ = request.respond(response);
            }
        }
    }

    if !received {
        return Err("timeout: no token received".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
