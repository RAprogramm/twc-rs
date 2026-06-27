// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

/// Output format for CLI results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Human-readable table (default).
    #[default]
    Table,
    /// Machine-readable JSON.
    Json,
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
            "quiet" | "q" => Ok(Self::Quiet),
            _ => Err(format!(
                "unknown output format: {s} \
                 (expected table, json, or quiet)"
            ))
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Table => write!(f, "table"),
            Self::Json => write!(f, "json"),
            Self::Quiet => write!(f, "quiet")
        }
    }
}

#[cfg(test)]
mod tests;
