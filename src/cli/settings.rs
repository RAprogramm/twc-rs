// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Config, auth and account subcommands.

use clap::Subcommand;

use super::LangArg;

/// Configuration subcommands.
#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show the current configuration.
    Show,
    /// Set the API token (for the default profile, or a named one).
    SetToken {
        /// The Timeweb Cloud API token.
        #[arg(long)]
        token: String,

        /// Store the token under this profile name instead of the default.
        #[arg(long)]
        profile: Option<String>
    },
    /// List configured profile names.
    Profiles,
    /// Set the UI language (en or ru).
    SetLanguage {
        /// Language code.
        #[arg(value_enum)]
        language: LangArg
    }
}

/// Authentication subcommands.
#[derive(Subcommand, Debug)]
pub enum AuthCommands {
    /// Run the guided browser authentication flow.
    Flow,
    /// Show current authentication status.
    Status,
    /// Remove stored token from keyring and config.
    Logout,
    /// Accept a token directly (for CI/CD).
    Token {
        /// The API token to store.
        #[arg(long)]
        token: String
    }
}

/// Account subcommands.
#[derive(Subcommand, Debug)]
pub enum AccountCommands {
    /// Show account login, company and balance.
    Show,
    /// Show account auth access restrictions (IP/country allow lists).
    Access
}
