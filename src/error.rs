// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

/// Application-level errors for the twc-rs CLI.
#[derive(Debug)]
pub enum TwcError {
    /// Configuration file not found at expected path.
    ConfigNotFound(String),
    /// Failed to parse configuration file.
    ConfigParse(String),
    /// Failed to write configuration file.
    ConfigWrite(String),
    /// No API token configured.
    TokenMissing,
    /// API request failed.
    Api(String),
    /// I/O operation failed.
    Io(String)
}

impl fmt::Display for TwcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigNotFound(path) => {
                write!(f, "config not found: {path}")
            }
            Self::ConfigParse(msg) => {
                write!(f, "failed to parse config: {msg}")
            }
            Self::ConfigWrite(msg) => {
                write!(f, "failed to write config: {msg}")
            }
            Self::TokenMissing => {
                write!(f, "no API token configured; run `twc-rs config set-token`")
            }
            Self::Api(msg) => write!(f, "API error: {msg}"),
            Self::Io(msg) => write!(f, "I/O error: {msg}")
        }
    }
}

impl std::error::Error for TwcError {}

impl<T> From<timeweb_rs::apis::Error<T>> for TwcError
where
    T: fmt::Debug
{
    fn from(err: timeweb_rs::apis::Error<T>) -> Self {
        match err {
            timeweb_rs::apis::Error::Reqwest(e) => Self::Api(e.to_string()),
            timeweb_rs::apis::Error::Serde(e) => Self::Api(e.to_string()),
            timeweb_rs::apis::Error::Io(e) => Self::Io(e.to_string()),
            timeweb_rs::apis::Error::ResponseError(content) => {
                let msg = content
                    .entity
                    .as_ref()
                    .map_or_else(|| format!("HTTP {}", content.status), |e| format!("{e:?}"));
                Self::Api(msg)
            }
        }
    }
}

impl From<std::io::Error> for TwcError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<toml::de::Error> for TwcError {
    fn from(err: toml::de::Error) -> Self {
        Self::ConfigParse(err.to_string())
    }
}

impl From<toml::ser::Error> for TwcError {
    fn from(err: toml::ser::Error) -> Self {
        Self::ConfigWrite(err.to_string())
    }
}

#[cfg(test)]
mod tests;
