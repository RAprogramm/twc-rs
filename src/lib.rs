// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! twc-rs library root.
//!
//! The crate is a CLI/TUI application first; the library target exists for
//! the auxiliary binaries and in-crate tests. Only [`cli`] is public API.
//! Every `#[doc(hidden)]` module is an implementation detail: it carries no
//! stability promise and is excluded from semver checking.

rust_i18n::i18n!("locales", fallback = "en");

pub mod cli;

#[doc(hidden)]
pub mod commands;
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod error;
#[doc(hidden)]
pub mod jwt;
#[doc(hidden)]
pub mod output;

#[cfg(feature = "auth")]
#[doc(hidden)]
pub mod auth;

#[cfg(feature = "tui")]
#[doc(hidden)]
pub mod tui;
