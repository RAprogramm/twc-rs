// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Guided browser-based authentication for Timeweb Cloud.
//!
//! Timeweb has no public `OAuth2` endpoint, so this module implements
//! a "guided browser auth" pattern: opens the token page in the user's
//! browser, starts a local HTTP server to receive the pasted token,
//! verifies it via the API, and stores it in the OS keyring.

mod flow;
mod server;
pub(crate) mod store;

use std::fmt;

pub use flow::{accept_token_direct, logout, run_auth_flow, show_status};
#[allow(unused_imports)]
pub use store::{delete_token, load_token, save_token};

/// Errors that can occur during the authentication flow.
#[derive(Debug)]
#[allow(dead_code)]
pub enum AuthError {
    /// Could not open the browser automatically.
    BrowserFailed,
    /// Timed out waiting for the user to paste the token.
    Timeout,
    /// The provided token is invalid or expired.
    InvalidToken,
    /// Failed to store the token in keyring or config file.
    StoreFailed(String),
    /// Network error during token verification.
    Network(String),
    /// Local HTTP server error.
    Server(String)
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BrowserFailed => {
                write!(f, "could not open browser; visit the URL manually")
            }
            Self::Timeout => {
                write!(f, "timed out waiting for token (5 min limit)")
            }
            Self::InvalidToken => {
                write!(f, "token is invalid or expired")
            }
            Self::StoreFailed(msg) => {
                write!(f, "failed to store token: {msg}")
            }
            Self::Network(msg) => {
                write!(f, "network error: {msg}")
            }
            Self::Server(msg) => {
                write!(f, "local server error: {msg}")
            }
        }
    }
}

impl std::error::Error for AuthError {}
