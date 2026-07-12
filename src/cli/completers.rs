// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Dynamic shell-completion value sources backed by the live API.

use std::{ffi::OsStr, time::Duration};

use clap_complete::engine::CompletionCandidate;
use timeweb_rs::{apis::apps_api, authenticated};

use crate::config::AppConfig;

/// Hard deadline for a completion network round-trip.
const COMPLETION_TIMEOUT: Duration = Duration::from_secs(3);

/// Resolves the API token without any interactive prompting or output.
///
/// Resolution order matches the CLI's non-interactive path: the `TWC_TOKEN`
/// environment variable, the config file, then the OS keyring.
fn silent_token() -> Option<String> {
    if let Ok(token) = std::env::var("TWC_TOKEN")
        && !token.is_empty()
    {
        return Some(token);
    }
    if let Ok(config) = AppConfig::load()
        && let Ok(Some(token)) = config.token_for(None)
    {
        return Some(token);
    }
    #[cfg(feature = "auth")]
    if let Ok(config_path) = AppConfig::path()
        && let Ok(token) = crate::auth::load_token(&config_path)
    {
        return Some(token);
    }
    None
}

/// Completes an app selector with live app names and IDs.
///
/// Fetches the account's apps and offers each one twice — by name (annotated
/// with its ID and status) and by numeric ID (annotated with its name). Any
/// failure — missing token, network error, timeout — yields no candidates so
/// the shell silently falls back to its default behavior.
#[must_use]
pub fn complete_app(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(prefix) = current.to_str() else {
        return Vec::new();
    };
    let Some(token) = silent_token() else {
        return Vec::new();
    };
    let config = authenticated(token);
    let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    else {
        return Vec::new();
    };
    let Ok(Ok(resp)) = runtime.block_on(async {
        tokio::time::timeout(COMPLETION_TIMEOUT, apps_api::get_apps(&config)).await
    }) else {
        return Vec::new();
    };

    let mut candidates = Vec::with_capacity(resp.apps.len() * 2);
    for app in &resp.apps {
        let id = app.id.to_string();
        if app.name.starts_with(prefix) {
            candidates.push(
                CompletionCandidate::new(app.name.as_str())
                    .help(Some(format!("{id} · {:?}", app.status).into()))
            );
        }
        if id.starts_with(prefix) {
            candidates
                .push(CompletionCandidate::new(id.as_str()).help(Some(app.name.clone().into())));
        }
    }
    candidates
}
