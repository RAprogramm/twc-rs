// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Terminal clipboard over the OSC 52 escape sequence.
//!
//! Modern terminals translate it into a system clipboard write — no external
//! tools or display-server bindings needed.

use std::io::Write;

use base64::{Engine, engine::general_purpose::STANDARD};

/// Sends `text` to the system clipboard through OSC 52.
pub fn copy(text: &str) {
    let payload = STANDARD.encode(text.as_bytes());
    let mut out = std::io::stdout();
    let _ = write!(out, "\x1b]52;c;{payload}\x07");
    let _ = out.flush();
}
