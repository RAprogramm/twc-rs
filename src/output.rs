// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

use serde::Serialize;
use tabled::{Table, Tabled, settings::Style};

use crate::error::TwcError;

/// Renders rows as a clean, modern table (rounded borders, a single header
/// rule, no noisy inter-row separators).
///
/// All CLI list commands use this so the output style stays consistent.
pub fn render_table<T: Tabled>(rows: &[T]) -> String {
    Table::new(rows).with(Style::rounded()).to_string()
}

/// Serializes a value as JSON or YAML for the machine-readable formats.
///
/// Returns `None` for `Table`/`Quiet` (which the caller renders itself), and
/// `Some(string)` for `Json`/`Yaml`. Keeps serialization in one place so every
/// command supports both without duplicating the logic.
///
/// # Errors
///
/// Returns [`TwcError::Api`] if serialization fails.
pub fn serialized<T: Serialize>(
    format: OutputFormat,
    value: &T
) -> Option<Result<String, TwcError>> {
    match format {
        OutputFormat::Json => {
            Some(serde_json::to_string_pretty(value).map_err(|e| TwcError::Api(e.to_string())))
        }
        OutputFormat::Yaml => {
            Some(serde_yml::to_string(value).map_err(|e| TwcError::Api(e.to_string())))
        }
        OutputFormat::Table | OutputFormat::Quiet => None
    }
}

/// Output format for CLI results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Human-readable table (default).
    #[default]
    Table,
    /// Machine-readable JSON.
    Json,
    /// Machine-readable YAML.
    Yaml,
    /// Minimal output — only essential data.
    Quiet
}

impl OutputFormat {
    /// Parses a string into an output format.
    ///
    /// # Overview
    ///
    /// Accepts `"table"`, `"json"`, and `"quiet"` (case-insensitive).
    ///
    /// # Errors
    ///
    /// Returns `Err` for unrecognized format names.
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "table" | "tbl" => Ok(Self::Table),
            "json" | "js" => Ok(Self::Json),
            "yaml" | "yml" => Ok(Self::Yaml),
            "quiet" | "q" => Ok(Self::Quiet),
            _ => Err(format!(
                "unknown output format: {s} \
                 (expected table, json, yaml, or quiet)"
            ))
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Table => write!(f, "table"),
            Self::Json => write!(f, "json"),
            Self::Yaml => write!(f, "yaml"),
            Self::Quiet => write!(f, "quiet")
        }
    }
}

#[cfg(test)]
mod tests;
