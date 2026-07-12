// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! The shared button chip: half-block caps around a filled label.
//!
//! Focused buttons fill with the accent color; idle ones with the border
//! color. Every button in the dashboard renders through this one component.

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span}
};

use crate::tui::themes::Palette;

/// Builds a button chip line.
#[must_use]
pub fn chip(label: &str, focused: bool, palette: &Palette) -> Line<'static> {
    let (bg, fg) = if focused {
        (palette.accent, palette.bg)
    } else {
        (palette.border, palette.fg)
    };
    let cap = Style::default().fg(bg);
    Line::from(vec![
        Span::styled("\u{2590}", cap),
        Span::styled(
            format!(" {label} "),
            Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD)
        ),
        Span::styled("\u{258C}", cap),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::themes::Theme;

    #[test]
    fn focused_chip_fills_with_accent() {
        let palette = Theme::GruvboxDark.palette();
        let focused = chip("Redeploy", true, &palette);
        let idle = chip("Redeploy", false, &palette);
        assert_eq!(focused.spans[1].style.bg, Some(palette.accent));
        assert_eq!(idle.spans[1].style.bg, Some(palette.border));
        assert!(focused.spans[1].content.contains("Redeploy"));
    }
}
