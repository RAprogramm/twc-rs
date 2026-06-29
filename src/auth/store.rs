// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Token storage with keyring (primary) and config file (fallback).

use std::path::Path;

use crate::config::AppConfig;

const SERVICE_NAME: &str = "twc-rs";
const ACCOUNT_NAME: &str = "token";

/// Saves the token to the OS keyring, falling back to the config file.
///
/// # Errors
///
/// Returns `Err` when both keyring and config file write fail.
pub fn save_token(token: &str, config_path: &Path) -> Result<(), super::AuthError> {
    if let Err(e) = save_to_keyring(token) {
        eprintln!("Keyring unavailable ({e}), saving to config file.");
        save_to_config(token, config_path)?;
    }
    Ok(())
}

/// Loads the token from the OS keyring, falling back to the config file.
///
/// # Errors
///
/// Returns `Err` when no token is found in either store.
pub fn load_token(config_path: &Path) -> Result<String, super::AuthError> {
    if let Ok(token) = load_from_keyring() {
        return Ok(token);
    }
    load_from_config(config_path)
}

/// Deletes the token from both keyring and config file.
///
/// # Errors
///
/// Returns `Err` when neither store contains a token.
pub fn delete_token(config_path: &Path) -> Result<(), super::AuthError> {
    let keyring_ok = delete_from_keyring().is_ok();
    let config_ok = delete_from_config(config_path).is_ok();
    if !keyring_ok && !config_ok {
        return Err(super::AuthError::StoreFailed(
            "no token found to delete".to_string()
        ));
    }
    Ok(())
}

fn save_to_keyring(token: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE_NAME, ACCOUNT_NAME).map_err(|e| e.to_string())?;
    entry.set_password(token).map_err(|e| e.to_string())?;
    Ok(())
}

fn load_from_keyring() -> Result<String, String> {
    let entry = keyring::Entry::new(SERVICE_NAME, ACCOUNT_NAME).map_err(|e| e.to_string())?;
    entry.get_password().map_err(|e| e.to_string())
}

fn delete_from_keyring() -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE_NAME, ACCOUNT_NAME).map_err(|e| e.to_string())?;
    entry.delete_credential().map_err(|e| e.to_string())?;
    Ok(())
}

fn save_to_config(token: &str, config_path: &Path) -> Result<(), super::AuthError> {
    let mut cfg = load_config_file(config_path).unwrap_or_default();
    cfg.token = Some(token.to_string());
    cfg.save_to(config_path)
        .map_err(|e| super::AuthError::StoreFailed(e.to_string()))
}

fn load_from_config(config_path: &Path) -> Result<String, super::AuthError> {
    let cfg = load_config_file(config_path)
        .ok_or_else(|| super::AuthError::StoreFailed("failed to read config file".to_string()))?;
    cfg.token
        .ok_or_else(|| super::AuthError::StoreFailed("no token in config".to_string()))
}

fn delete_from_config(config_path: &Path) -> Result<(), super::AuthError> {
    let mut cfg = load_config_file(config_path)
        .ok_or_else(|| super::AuthError::StoreFailed("failed to read config file".to_string()))?;
    if cfg.token.is_none() {
        return Err(super::AuthError::StoreFailed(
            "no token in config".to_string()
        ));
    }
    cfg.token = None;
    cfg.save_to(config_path)
        .map_err(|e| super::AuthError::StoreFailed(e.to_string()))
}

fn load_config_file(config_path: &Path) -> Option<AppConfig> {
    if config_path.exists() {
        let content = std::fs::read_to_string(config_path).ok()?;
        toml::from_str(&content).ok()
    } else {
        Some(AppConfig::default())
    }
}

#[cfg(test)]
mod tests;
