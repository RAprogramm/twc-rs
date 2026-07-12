// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Reusable adaptive card grid.
//!
//! One renderer, shared by every card surface (the resource list, a project's
//! contents, and any future one): given a list of [`GridCard`]s it sizes the
//! columns to the terminal width and the longest label, lays the cards out to
//! fill the width, and scrolls to keep the selection visible. No caller
//! hard-codes a column count or a card width.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph}
};

use crate::tui::themes::Palette;

/// A single card: an optional icon, a title, an optional colored status badge
/// and a secondary metric line. Every card surface maps its data to this shape.
#[derive(Debug, Clone)]
pub struct GridCard {
    /// Leading glyph (may be empty).
    pub icon:   String,
    /// Primary label.
    pub title:  String,
    /// Optional colored status badge `(color, label)`.
    pub status: Option<(Color, String)>,
    /// Secondary metric line (may be empty).
    pub meta:   String
}

impl GridCard {
    /// A card with only a title.
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            icon:   String::new(),
            title:  title.into(),
            status: None,
            meta:   String::new()
        }
    }

    /// Sets the leading icon.
    #[must_use]
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = icon.into();
        self
    }

    /// Sets the colored status badge.
    #[must_use]
    pub fn status(mut self, color: Color, label: impl Into<String>) -> Self {
        self.status = Some((color, label.into()));
        self
    }

    /// Sets the secondary metric line.
    #[must_use]
    pub fn meta(mut self, meta: impl Into<String>) -> Self {
        self.meta = meta.into();
        self
    }
}

/// Smallest width a card cell may shrink to before dropping a column.
const MIN_CELL_W: u16 = 16;
/// Widest a single card cell is sized to fit its label before columns stop
/// growing (extra width past this is shared out by [`Constraint::Fill`]).
const MAX_CELL_W: u16 = 34;
/// Cells a card spends on its border plus the icon and its trailing spaces.
const LABEL_OVERHEAD: u16 = 6;
/// Horizontal gap between card cells, in cells.
const HGAP: u16 = 2;
/// Vertical gap between card rows, in rows.
const VGAP: u16 = 1;
/// Fixed card height: border, title, status, meta, border.
const CARD_H: u16 = 5;
/// Upper bound on the number of grid columns.
const MAX_COLS: usize = 8;

/// Number of grid columns for a grid `avail` cells wide.
///
/// Cards are sized wide enough to fit the `longest_label` (capped at
/// [`MAX_CELL_W`]), then as many columns as fit are used. Rendering and
/// keyboard navigation both derive columns from this so they never disagree.
#[must_use]
pub fn columns(avail: u16, longest_label: usize) -> usize {
    let needed = u16::try_from(longest_label)
        .unwrap_or(MAX_CELL_W)
        .saturating_add(LABEL_OVERHEAD)
        .clamp(MIN_CELL_W, MAX_CELL_W);
    usize::from((avail + HGAP) / (needed + HGAP)).clamp(1, MAX_COLS)
}

/// Character length of the longest card title, for [`columns`].
#[must_use]
pub fn longest_title(cards: &[GridCard]) -> usize {
    cards
        .iter()
        .map(|c| c.title.chars().count())
        .max()
        .unwrap_or(0)
}

/// Renders just the card grid into an already-prepared inner area, for panels
/// that draw their own border and chrome around it.
pub fn render_grid_in(
    frame: &mut Frame,
    inner: Rect,
    cards: &[GridCard],
    selected: usize,
    cols: usize,
    palette: &Palette
) {
    render_grid(frame, inner, cards, selected, cols.max(1), palette);
}

fn render_grid(
    frame: &mut Frame,
    inner: Rect,
    cards: &[GridCard],
    selected: usize,
    cols: usize,
    palette: &Palette
) {
    if inner.height < 3 || inner.width < 6 {
        return;
    }
    let rows_total = cards.len().div_ceil(cols);
    let rows_fit = usize::from((inner.height + VGAP) / (CARD_H + VGAP)).max(1);

    let anchor = selected.min(cards.len().saturating_sub(1));
    let selected_row = anchor / cols;
    let first_row = if rows_total <= rows_fit || selected_row < rows_fit {
        0
    } else {
        selected_row + 1 - rows_fit
    };

    let visible_rows = rows_fit.min(rows_total - first_row);
    let row_areas = Layout::vertical(vec![Constraint::Length(CARD_H); visible_rows])
        .spacing(VGAP)
        .split(inner);

    for (ri, row_area) in row_areas.iter().enumerate() {
        let grid_row = first_row + ri;
        let cells = Layout::horizontal(vec![Constraint::Fill(1); cols])
            .spacing(HGAP)
            .split(*row_area);
        for (ci, cell) in cells.iter().enumerate() {
            let idx = grid_row * cols + ci;
            let Some(card) = cards.get(idx) else {
                break;
            };
            render_card(frame, *cell, card, idx == selected, palette);
        }
    }
}

fn render_card(frame: &mut Frame, rect: Rect, card: &GridCard, selected: bool, palette: &Palette) {
    if rect.height < 3 || rect.width < 6 {
        return;
    }
    let border = if selected {
        palette.accent
    } else {
        palette.border
    };
    let inner_w = usize::from(rect.width.saturating_sub(2));

    let title_span = if card.icon.is_empty() {
        Span::styled(
            truncate(&card.title, inner_w),
            Style::default().fg(palette.fg).add_modifier(Modifier::BOLD)
        )
    } else {
        Span::styled(
            truncate(&card.title, inner_w.saturating_sub(3)),
            Style::default().fg(palette.fg).add_modifier(Modifier::BOLD)
        )
    };
    let mut title_line = Vec::with_capacity(2);
    if !card.icon.is_empty() {
        title_line.push(Span::styled(
            format!("{}  ", card.icon),
            Style::default().fg(palette.accent)
        ));
    }
    title_line.push(title_span);

    let mut lines = vec![Line::from(title_line)];
    if let Some((color, label)) = &card.status {
        lines.push(Line::from(Span::styled(
            format!("\u{25CF} {}", truncate(label, inner_w.saturating_sub(2))),
            Style::default().fg(*color)
        )));
    }
    if !card.meta.is_empty() {
        lines.push(Line::from(Span::styled(
            truncate(&card.meta, inner_w),
            Style::default().fg(palette.dim)
        )));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border));
    frame.render_widget(Paragraph::new(lines).block(block), rect);
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('\u{2026}');
    out
}

#[cfg(test)]
mod tests {
    use ratatui::{Terminal, backend::TestBackend};

    use super::*;
    use crate::tui::themes::Theme;

    #[test]
    fn columns_adapt_to_width_and_label() {
        assert_eq!(columns(0, 8), 1);
        assert!(columns(120, 8) > columns(40, 8));
        assert!(columns(120, 30) <= columns(120, 6));
        assert!(columns(4000, 8) <= MAX_COLS);
    }

    #[test]
    fn renders_across_sizes_without_panic() {
        let cards: Vec<GridCard> = (0..12)
            .map(|i| {
                GridCard::new(format!("item-{i}"))
                    .icon("\u{25A0}")
                    .status(Color::Green, "running")
                    .meta("2c · 4096 MB")
            })
            .collect();
        let palette = Theme::GruvboxDark.palette();
        for (w, h) in [(0, 0), (1, 1), (3, 3), (20, 6), (80, 24), (200, 60)] {
            let backend = TestBackend::new(w.max(1), h.max(1));
            let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|frame| {
                    let cols = columns(w, longest_title(&cards));
                    render_grid_in(frame, Rect::new(0, 0, w, h), &cards, 0, cols, &palette);
                })
                .unwrap();
        }
    }
}
