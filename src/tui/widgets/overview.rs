// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! The overview landing screen: Projects and Services zones rendered as a
//! responsive grid of cards, each showing a Nerd Font icon, a name and a count.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph}
};
use rust_i18n::t;

use crate::tui::{
    app::{App, OverviewCard},
    themes::Palette
};

/// Nerd Font glyphs per resource category, indexed by [`ResourceTab::index`].
/// Requires a patched Nerd Font in the terminal.
const NERD_ICONS: [&str; 20] = [
    "\u{f0ae1}", // Servers   (md-server-network)
    "\u{f1c0}",  // Databases
    "\u{f0a0}",  // S3
    "\u{f10fe}", // Kubernetes 󱃾
    "\u{f07b}",  // Projects
    "\u{f0e97}", // Balancers 󰺗 (scale-balance)
    "\u{f308}",  // Registry  (docker)
    "\u{f0ac}",  // Domains
    "\u{f132}",  // Firewall  (shield)
    "\u{f0e8}",  // FloatingIps  (sitemap)
    "\u{f03e}",  // Images
    "\u{f0a0}",  // NetworkDrives  (hdd)
    "\u{f0e8}",  // Vpc  (sitemap)
    "\u{f233}",  // DedicatedServers
    "\u{f0e0}",  // Mail
    "\u{f135}",  // Apps  (rocket)
    "\u{f544}",  // AiAgents  (robot)
    "\u{f02d}",  // KnowledgeBases
    "\u{f084}",  // SshKeys
    "\u{f155}"   // Finances
];

/// Fixed inner width of a card in cells (border excluded).
const CARD_W: u16 = 20;
/// Fixed total height of a card in rows (border included).
const CARD_H: u16 = 4;
/// Horizontal gap between cards in cells.
const GAP: u16 = 2;

/// Renders the overview screen into `area`, updating `app.overview_cols` so
/// keyboard navigation matches the rendered grid.
pub fn render(frame: &mut Frame, area: Rect, app: &App, palette: Palette) {
    let projects = app.overview_project_cards();
    let services = app.overview_service_cards();

    let cols = usize::from((area.width.saturating_sub(2) + GAP) / (CARD_W + 2 + GAP)).max(1);

    let mut y = area.y;
    let mut index = 0usize;

    if !projects.is_empty() {
        y = render_zone(
            frame,
            area,
            &t!("overview.projects"),
            &projects,
            cols,
            &mut index,
            app.overview_selected,
            y,
            palette
        );
        y = y.saturating_add(1);
    }

    render_zone(
        frame,
        area,
        &t!("overview.services"),
        &services,
        cols,
        &mut index,
        app.overview_selected,
        y,
        palette
    );
}

/// Number of grid columns the overview last rendered with, for navigation.
#[must_use]
pub fn columns_for(width: u16) -> usize {
    usize::from((width.saturating_sub(2) + GAP) / (CARD_W + 2 + GAP)).max(1)
}

#[allow(clippy::too_many_arguments)]
fn render_zone(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    cards: &[OverviewCard],
    cols: usize,
    index: &mut usize,
    selected: usize,
    start_y: u16,
    palette: Palette
) -> u16 {
    let mut y = start_y;
    if y >= area.bottom() {
        *index += cards.len();
        return y;
    }

    let header = Paragraph::new(Line::from(Span::styled(
        format!("  {}", title.to_uppercase()),
        Style::default()
            .fg(palette.header)
            .add_modifier(Modifier::BOLD)
    )));
    frame.render_widget(header, Rect::new(area.x, y, area.width, 1));
    y = y.saturating_add(1);

    for (i, card) in cards.iter().enumerate() {
        let col = i % cols;
        let row = i / cols;
        let card_x = area.x + 2 + (col as u16) * (CARD_W + 2 + GAP);
        let card_y = y + (row as u16) * (CARD_H + 1);
        if card_y + CARD_H > area.bottom() {
            continue;
        }
        let rect = Rect::new(card_x, card_y, CARD_W + 2, CARD_H);
        render_card(frame, rect, card, *index + i == selected, palette);
    }

    *index += cards.len();
    let rows = cards.len().div_ceil(cols) as u16;
    y + rows * (CARD_H + 1)
}

fn render_card(
    frame: &mut Frame,
    rect: Rect,
    card: &OverviewCard,
    selected: bool,
    palette: Palette
) {
    let border = if selected {
        palette.accent
    } else {
        palette.border
    };
    let icon = NERD_ICONS
        .get(card.icon.index())
        .copied()
        .unwrap_or("\u{25A0}");

    let title_line = Line::from(vec![
        Span::styled(format!("{icon}  "), Style::default().fg(palette.accent)),
        Span::styled(
            truncate(&card.label, usize::from(CARD_W).saturating_sub(4)),
            Style::default().fg(palette.fg).add_modifier(Modifier::BOLD)
        ),
    ]);
    let count_line = Line::from(Span::styled(
        card.count.to_string(),
        Style::default()
            .fg(if selected {
                palette.accent
            } else {
                palette.dim
            })
            .add_modifier(Modifier::BOLD)
    ));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border));
    let paragraph = Paragraph::new(vec![title_line, count_line]).block(block);
    frame.render_widget(paragraph, rect);
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('\u{2026}');
    out
}
