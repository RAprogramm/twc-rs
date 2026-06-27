// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::TwcError;

/// Output format preference.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputPreference {
    /// Human-readable table (default).
    #[default]
    Table,
    /// Machine-readable JSON.
    Json,
    /// Minimal output.
    Quiet
}

/// File-based configuration stored at `~/.config/twc-rs/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Timeweb Cloud API token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    /// TUI color theme.
    #[serde(default)]
    pub theme: crate::tui::themes::Theme,

    /// Default output format.
    #[serde(default, alias = "output")]
    pub output: OutputPreference,

    /// Default region for new servers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_region: Option<String>,

    /// Auto-refresh interval in seconds for TUI monitor.
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval: u64
}

fn default_refresh_interval() -> u64 {
    5
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            token:            None,
            theme:            crate::tui::themes::Theme::default(),
            output:           OutputPreference::Table,
            default_region:   None,
            refresh_interval: 5
        }
    }
}

impl AppConfig {
    /// Returns the path to the configuration file.
    ///
    /// # Overview
    ///
    /// Resolves `~/.config/twc-rs/config.toml` using the `dirs` crate.
    ///
    /// # Errors
    ///
    /// Returns [`TwcError::ConfigNotFound`] when the config directory
    /// cannot be determined by the OS.
    pub fn path() -> Result<PathBuf, TwcError> {
        let dir = dirs::config_dir().ok_or_else(|| {
            TwcError::ConfigNotFound("unable to determine config directory".to_string())
        })?;
        Ok(dir.join("twc-rs").join("config.toml"))
    }

    /// Loads the configuration from disk.
    ///
    /// # Overview
    ///
    /// Reads and deserializes the TOML config file. Returns default
    /// configuration when the file does not exist. Creates the config
    /// file with defaults on first access.
    ///
    /// # Errors
    ///
    /// Returns [`TwcError::ConfigNotFound`] or [`TwcError::ConfigParse`]
    /// on read / deserialization failure.
    pub fn load() -> Result<Self, TwcError> {
        let path = Self::path()?;
        if !path.exists() {
            let cfg = Self::default();
            cfg.save()?;
            return Ok(cfg);
        }
        let content = fs::read_to_string(&path)
            .map_err(|e| TwcError::ConfigNotFound(format!("{}: {e}", path.display())))?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Persists the configuration to disk.
    ///
    /// # Overview
    ///
    /// Creates parent directories as needed, then writes the TOML file.
    ///
    /// # Errors
    ///
    /// Returns [`TwcError::ConfigWrite`] on serialization or I/O failure.
    pub fn save(&self) -> Result<(), TwcError> {
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                TwcError::ConfigWrite(format!("failed to create dir {}: {e}", parent.display()))
            })?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content).map_err(|e| {
            TwcError::ConfigWrite(format!("failed to write {}: {e}", path.display()))
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
