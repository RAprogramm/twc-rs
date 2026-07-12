// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! API-token resolution, interactive prompting and persistence for the CLI.

use rust_i18n::t;

#[cfg(feature = "auth")]
use crate::auth;
use crate::{config::AppConfig, error::TwcError};

/// Resolves the API token from CLI flag, environment, or config file.
///
/// # Overview
///
/// Priority order: `--token` flag > `TWC_TOKEN` env var >
/// config file (`~/.config/twc-rs/config.toml`).
///
/// # Errors
///
/// Returns [`TwcError::TokenMissing`] when no token is found.
pub fn resolve_token(cli_token: Option<&str>, profile: Option<&str>) -> Result<String, TwcError> {
    if let Some(token) = cli_token {
        return Ok(token.to_string());
    }

    let app_config = AppConfig::load()?;
    if let Some(token) = app_config.token_for(profile)? {
        return Ok(token);
    }

    Err(TwcError::TokenMissing)
}

/// Resolves the API token, falling back to interactive prompt.
///
/// # Overview
///
/// Calls [`resolve_token`] first. If no token is found, shows an
/// interactive menu to get one. Saves the token to config so the
/// user is never prompted again.
///
/// # Errors
///
/// Only returns error if config file operations fail catastrophically.
pub fn ensure_token(cli_token: Option<&str>, profile: Option<&str>) -> Result<String, TwcError> {
    if let Ok(token) = resolve_token(cli_token, profile) {
        return Ok(token);
    }

    #[cfg(feature = "auth")]
    if profile.is_none()
        && let Ok(config_path) = AppConfig::path()
        && let Ok(token) = auth::load_token(&config_path)
    {
        return Ok(token);
    }

    prompt_and_save_token()
}

/// Shows an interactive prompt to get a token, then saves it.
pub fn prompt_and_save_token() -> Result<String, TwcError> {
    use colored::Colorize as _;
    use dialoguer::Select;

    println!("\n  {}\n", t!("app.no_token_configured").yellow().bold());

    #[cfg(feature = "auth")]
    let options = vec![
        "Create new API key (opens browser)",
        "I have a key — paste it",
    ];
    #[cfg(not(feature = "auth"))]
    let options = vec!["Paste token from clipboard"];

    let selection = Select::new()
        .with_prompt("How to authenticate?")
        .items(&options)
        .default(0)
        .interact()
        .map_err(|e| TwcError::Io(e.to_string()))?;

    let token = match selection {
        0 => {
            #[cfg(feature = "auth")]
            {
                prompt_browser_flow()?
            }
            #[cfg(not(feature = "auth"))]
            prompt_paste_token()?
        }
        #[cfg(feature = "auth")]
        1 => prompt_paste_token()?,
        _ => std::process::exit(0)
    };

    save_token_to_config(&token)?;
    Ok(token)
}

/// Prompts user to paste a token (reads full stdin, shows masked preview).
fn prompt_paste_token() -> Result<String, TwcError> {
    let token: String = dialoguer::Password::new()
        .with_prompt("Paste your API token")
        .allow_empty_password(false)
        .interact()
        .map_err(|e| TwcError::Io(e.to_string()))?;

    let token = token.trim().to_string();
    if token.is_empty() {
        return Err(TwcError::Api("empty token".to_string()));
    }
    let masked = mask_token(&token);
    println!("  \u{2713} Token received: {masked}");
    Ok(token)
}

/// Masks a token for safe display: first 4 + *** + last 4.
fn mask_token(token: &str) -> String {
    let len = token.len();
    if len <= 8 {
        return "*".repeat(len);
    }
    let first = &token[..4];
    let last = &token[len - 4..];
    format!("{first}***{last}")
}

/// Opens browser and runs full local HTTP server auth flow.
#[cfg(feature = "auth")]
fn prompt_browser_flow() -> Result<String, TwcError> {
    let config_path =
        AppConfig::path().unwrap_or_else(|_| std::path::PathBuf::from("config.toml"));
    auth::run_auth_flow(&config_path).map_err(|e| TwcError::Api(e.to_string()))?;
    auth::load_token(&config_path).map_err(|e| TwcError::Api(e.to_string()))
}

/// Saves the token to config file (and keyring if auth feature is on).
pub fn save_token_to_config(token: &str) -> Result<(), TwcError> {
    use colored::Colorize as _;

    #[cfg(feature = "auth")]
    {
        let config_path =
            AppConfig::path().unwrap_or_else(|_| std::path::PathBuf::from("config.toml"));
        if let Err(e) = auth::store::save_token(token, &config_path) {
            eprintln!("  Warning: could not save to keyring: {e}");
        }
    }

    let mut cfg = AppConfig::load()?;
    cfg.token = Some(token.to_string());
    cfg.save()?;

    let masked = mask_token(token);
    println!("\n  {} ({masked})\n", t!("app.token_saved").green().bold());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_token_cli_flag_takes_priority() {
        let result = resolve_token(Some("my-token"), None);
        assert_eq!(result.unwrap(), "my-token");
    }

    #[test]
    fn resolve_token_missing_without_config() {
        let dir = tempfile::tempdir().unwrap();
        let orig = std::env::var("XDG_CONFIG_HOME").ok();
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", dir.path());
        }
        let result = resolve_token(None, None);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("no API token configured"));
        unsafe {
            match orig {
                Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
                None => std::env::remove_var("XDG_CONFIG_HOME")
            }
        }
    }
}
