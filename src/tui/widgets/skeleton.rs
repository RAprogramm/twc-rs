// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Skeleton loading-placeholder widget with an animated shimmer sweep.
//!
//! Renders pulsing block bars while data is loading, replacing a plain
//! spinner with a content-shaped placeholder. The animation is driven
//! purely by a `tick` counter, so output is deterministic for a given
//! `(area, rows, tick)` triple.

#![allow(dead_code)]

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph}
};

use crate::tui::themes::Palette;

/// Base glyph for the dim portion of a shimmer bar.
const BASE_GLYPH: char = '░';

/// Glyph for the bright moving highlight of a shimmer bar.
const HIGHLIGHT_GLYPH: char = '█';

/// Glyph for the soft trailing edge around the highlight.
const TRAIL_GLYPH: char = '▒';

/// Number of cells on each side of the highlight that fade out.
const TRAIL_SPAN: u16 = 1;

/// Returns the shimmer highlight column for a bar of the given width at `tick`.
///
/// The highlight sweeps left-to-right and wraps, always staying within
/// `0..width`. A zero width yields `0`.
#[must_use]
fn shimmer_pos(width: u16, tick: u64) -> u16 {
    if width == 0 {
        return 0;
    }
    (tick % u64::from(width)) as u16
}

/// Builds a single shimmer bar line of the given width for the current `tick`.
fn shimmer_line<'a>(width: u16, tick: u64, base: Style, highlight: Style) -> Line<'a> {
    let pos = shimmer_pos(width, tick);
    let mut spans: Vec<Span<'a>> = Vec::with_capacity(usize::from(width));
    for col in 0..width {
        let distance = col.abs_diff(pos);
        let (glyph, style) = if distance == 0 {
            (HIGHLIGHT_GLYPH, highlight)
        } else if distance <= TRAIL_SPAN {
            (TRAIL_GLYPH, highlight)
        } else {
            (BASE_GLYPH, base)
        };
        spans.push(Span::styled(glyph.to_string(), style));
    }
    Line::from(spans)
}

/// Returns the visible width of a bar at `row`, slightly varied so the
/// placeholder resembles real content of differing lengths.
fn bar_width(inner_width: u16, row: usize) -> u16 {
    if inner_width == 0 {
        return 0;
    }
    let shrink = (row as u16 % 4).saturating_mul(2);
    inner_width.saturating_sub(shrink).max(1)
}

/// Renders `rows` shimmer placeholder bars inside `area`, bordered, titled `title`.
pub fn render(
    frame: &mut Frame,
    area: Rect,
    palette: &Palette,
    title: &str,
    rows: usize,
    tick: u64
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(palette.border))
        .title(Span::styled(
            title.to_string(),
            Style::default()
                .fg(palette.title)
                .add_modifier(Modifier::BOLD)
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let base = Style::default().fg(palette.dim);
    let highlight = Style::default()
        .fg(palette.accent)
        .add_modifier(Modifier::BOLD);

    let visible_rows = rows.min(usize::from(inner.height));
    let mut lines: Vec<Line> = Vec::with_capacity(visible_rows);
    for row in 0..visible_rows {
        let width = bar_width(inner.width, row);
        let phase = tick.wrapping_add(row as u64 * 3);
        lines.push(shimmer_line(width, phase, base, highlight));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

#[cfg(test)]
mod tests {
    use ratatui::{Terminal, backend::TestBackend};

    use super::*;
    use crate::tui::themes::Theme;

    #[test]
    fn shimmer_pos_always_within_width() {
        for width in 1u16..=64 {
            for tick in 0u64..256 {
                let pos = shimmer_pos(width, tick);
                assert!(pos < width, "pos {pos} >= width {width} at tick {tick}");
            }
        }
    }

    #[test]
    fn shimmer_pos_zero_width_is_zero() {
        assert_eq!(shimmer_pos(0, 0), 0);
        assert_eq!(shimmer_pos(0, 999), 0);
    }

    #[test]
    fn shimmer_pos_varies_with_tick() {
        let width = 10;
        let a = shimmer_pos(width, 0);
        let b = shimmer_pos(width, 1);
        let c = shimmer_pos(width, 5);
        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_eq!(shimmer_pos(width, 0), shimmer_pos(width, u64::from(width)));
    }

    #[test]
    fn bar_width_never_exceeds_inner_and_at_least_one() {
        for inner in 1u16..=40 {
            for row in 0usize..16 {
                let w = bar_width(inner, row);
                assert!(w >= 1);
                assert!(w <= inner);
            }
        }
    }

    #[test]
    fn bar_width_zero_inner_is_zero() {
        assert_eq!(bar_width(0, 0), 0);
        assert_eq!(bar_width(0, 7), 0);
    }

    fn palette() -> Palette {
        Theme::GruvboxDark.palette()
    }

    #[test]
    fn render_does_not_panic_on_zero_area() {
        let backend = TestBackend::new(20, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let zero_w = Rect::new(0, 0, 0, 5);
                let zero_h = Rect::new(0, 0, 5, 0);
                render(frame, zero_w, &palette(), "Loading", 4, 7);
                render(frame, zero_h, &palette(), "Loading", 4, 7);
            })
            .unwrap();
    }

    #[test]
    fn render_does_not_panic_on_tiny_area() {
        let backend = TestBackend::new(4, 4);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                render(frame, Rect::new(0, 0, 1, 1), &palette(), "L", 3, 2);
                render(frame, Rect::new(0, 0, 2, 2), &palette(), "Load", 8, 11);
            })
            .unwrap();
    }

    #[test]
    fn render_handles_normal_area() {
        let backend = TestBackend::new(40, 12);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                render(frame, Rect::new(0, 0, 40, 12), &palette(), "Loading", 6, 42);
            })
            .unwrap();
    }
}
