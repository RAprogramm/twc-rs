// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Orchestrates the full interactive browser auth sequence.

use std::{path::Path, sync::mpsc, thread, time::Duration};

use timeweb_rs::{apis::account_api, authenticated};

use super::{server::serve_once, store};

const TOKEN_PAGE_URL: &str = "https://timeweb.cloud/my/api-keys";
const TIMEOUT_SECS: u64 = 300;

/// Runs the full interactive browser auth sequence.
///
/// # Overview
///
/// 1. Starts a local HTTP server on an ephemeral port.
/// 2. Opens the browser to the Timeweb token page.
/// 3. Waits for the user to paste a token.
/// 4. Verifies the token via the API.
/// 5. Saves the token to keyring (or config file fallback).
///
/// # Errors
///
/// Returns [`super::AuthError`] on any failure in the flow.
pub fn run_auth_flow(config_path: &Path) -> Result<(), super::AuthError> {
    let (tx, rx) = mpsc::sync_channel(1);

    let port = find_free_port()?;
    let server_handle = thread::spawn(move || serve_once(port, tx));

    println!("  Opening Timeweb Cloud token page in your browser...");
    println!("  If the browser did not open, go to:\n  {TOKEN_PAGE_URL}\n");
    println!("  Create an API token, then paste it below.\n");

    if let Err(e) = open::that(TOKEN_PAGE_URL) {
        eprintln!("  Could not open browser: {e}");
        println!("  Please visit the URL manually.");
    }

    let token = rx
        .recv_timeout(Duration::from_secs(TIMEOUT_SECS))
        .map_err(|_| super::AuthError::Timeout)?;

    server_handle.join().ok();

    println!("  Verifying token...");

    verify_token_sync(&token)?;

    store::save_token(&token, config_path)?;

    println!("  ✓ Authenticated successfully.");
    println!("  Token saved to system keyring.");
    Ok(())
}

/// Accepts a token directly (non-interactive mode for CI/CD).
///
/// # Errors
///
/// Returns [`super::AuthError`] when verification or storage fails.
pub fn accept_token_direct(token: &str, config_path: &Path) -> Result<(), super::AuthError> {
    verify_token_sync(token)?;
    store::save_token(token, config_path)?;
    println!("  ✓ Token verified and saved.");
    Ok(())
}

/// Prints the current authentication status.
///
/// # Errors
///
/// Returns [`super::AuthError`] when no valid token is found.
pub fn show_status(config_path: &Path) -> Result<(), super::AuthError> {
    let token = store::load_token(config_path)?;
    verify_token_sync(&token)?;
    let masked = mask_token(&token);
    println!("  ✓ Authenticated. Token: {masked}");
    Ok(())
}

/// Logs out by removing stored tokens.
///
/// # Errors
///
/// Returns [`super::AuthError`] when deletion fails.
pub fn logout(config_path: &Path) -> Result<(), super::AuthError> {
    store::delete_token(config_path)?;
    println!("  ✓ Logged out. Token removed from keyring and config.");
    Ok(())
}

#[expect(dead_code)]
async fn verify_token(token: &str) -> Result<(), super::AuthError> {
    let config = authenticated(token.to_string());
    account_api::get_account_status(&config)
        .await
        .map_err(|e| match e {
            timeweb_rs::apis::Error::ResponseError(content) if content.status == 401 => {
                super::AuthError::InvalidToken
            }
            _ => super::AuthError::Network(e.to_string())
        })?;
    Ok(())
}

fn verify_token_sync(token: &str) -> Result<(), super::AuthError> {
    let config = authenticated(token.to_string());
    // Run in a separate thread to avoid nested tokio runtime issues
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| super::AuthError::Network(e.to_string()))?;
        rt.block_on(async {
            account_api::get_account_status(&config)
                .await
                .map_err(|e| match e {
                    timeweb_rs::apis::Error::ResponseError(content) if content.status == 401 => {
                        super::AuthError::InvalidToken
                    }
                    _ => super::AuthError::Network(e.to_string())
                })
        })
    })
    .join()
    .map_err(|e| super::AuthError::Network(format!("{e:?}")))?
    .map_err(|e| super::AuthError::Network(e.to_string()))?;
    Ok(())
}

fn mask_token(token: &str) -> String {
    let len = token.len();
    if len <= 8 {
        return "*".repeat(len);
    }
    let first = &token[..4];
    let last = &token[len - 4..];
    format!("{first}***{last}")
}

pub fn find_free_port() -> Result<u16, super::AuthError> {
    std::net::TcpListener::bind("127.0.0.1:0")
        .map_err(|e| super::AuthError::Server(format!("no free port available: {e}")))
        .and_then(|l| {
            l.local_addr()
                .map(|a| a.port())
                .map_err(|e| super::AuthError::Server(e.to_string()))
        })
}

#[cfg(test)]
mod tests;
