// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::{
    fs,
    path::{Path, PathBuf}
};

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

/// Dashboard customization preferences, persisted in the config file.
#[cfg(feature = "tui")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DashboardPrefs {
    /// IDs of widgets the user has hidden from the layout.
    #[serde(default)]
    pub hidden_widgets: Vec<String>,

    /// Resource-list panel width, as a percentage of the content area.
    #[serde(default = "default_list_width")]
    pub list_width_pct: u16
}

#[cfg(feature = "tui")]
const fn default_list_width() -> u16 {
    40
}

#[cfg(feature = "tui")]
impl Default for DashboardPrefs {
    fn default() -> Self {
        Self {
            hidden_widgets: Vec::new(),
            list_width_pct: default_list_width()
        }
    }
}

/// File-based configuration stored at `~/.config/twc-rs/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Timeweb Cloud API token for the default profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    /// Named profiles, mapping a profile name to its API token. Selected with
    /// `--profile <name>` (or the `TWC_PROFILE` env var).
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub profiles: std::collections::HashMap<String, String>,

    /// TUI color theme.
    #[cfg(feature = "tui")]
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
    pub refresh_interval: u64,

    /// Dashboard layout customization.
    #[cfg(feature = "tui")]
    #[serde(default)]
    pub dashboard: DashboardPrefs
}

const fn default_refresh_interval() -> u64 {
    5
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            token: None,
            profiles: std::collections::HashMap::new(),
            #[cfg(feature = "tui")]
            theme: crate::tui::themes::Theme::default(),
            output: OutputPreference::Table,
            default_region: None,
            refresh_interval: 5,
            #[cfg(feature = "tui")]
            dashboard: DashboardPrefs::default()
        }
    }
}

impl AppConfig {
    /// Returns the token for the given profile, or the default token when no
    /// profile is named.
    ///
    /// # Errors
    ///
    /// Returns [`TwcError::ConfigNotFound`] when the named profile does not
    /// exist.
    pub fn token_for(&self, profile: Option<&str>) -> Result<Option<String>, TwcError> {
        match profile {
            Some(name) => self.profiles.get(name).cloned().map(Some).ok_or_else(|| {
                TwcError::ConfigNotFound(format!("profile '{name}' not found in config"))
            }),
            None => Ok(self.token.clone())
        }
    }

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
        self.save_to(&Self::path()?)
    }

    /// Persists the configuration to a specific path.
    ///
    /// # Overview
    ///
    /// Creates parent directories as needed, then writes the TOML file to
    /// `path`. Used by callers that manage their own config location (and by
    /// tests, to avoid touching the real user config).
    ///
    /// # Errors
    ///
    /// Returns [`TwcError::ConfigWrite`] on serialization or I/O failure.
    pub fn save_to(&self, path: &Path) -> Result<(), TwcError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                TwcError::ConfigWrite(format!("failed to create dir {}: {e}", parent.display()))
            })?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content).map_err(|e| {
            TwcError::ConfigWrite(format!("failed to write {}: {e}", path.display()))
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
