// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Simple loading spinner for TUI.

use std::time::Instant;

use ratatui::text::Span;

/// Loading spinner frames using Nerd Fonts compatible characters.
const FRAMES: &[&str] = &["◜", "◠", "◝", "◞", "◡", "◟"];

static START_TIME: std::sync::LazyLock<Instant> = std::sync::LazyLock::new(Instant::now);

/// Returns the current spinner frame based on elapsed time.
#[must_use]
pub fn current_frame() -> Span<'static> {
    let elapsed = Instant::now().duration_since(*START_TIME).as_millis();
    let index = (elapsed / 100) as usize % FRAMES.len();
    Span::raw(FRAMES[index])
}
