// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! The overview landing screen: Projects and Services zones rendered as a
//! responsive grid of cards that flex to fill the whole available area, each
//! showing a Nerd Font icon, a name and a count.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
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

/// Smallest width a card cell may shrink to before dropping a column.
const MIN_CELL_W: u16 = 18;
/// Horizontal gap between card cells, in cells.
const GAP: u16 = 2;
/// Upper bound on the number of grid columns.
const MAX_COLS: usize = 8;

/// Number of grid columns the overview renders with at the given outer width.
///
/// Both [`render`] and keyboard navigation derive their column count from this
/// so the two never disagree.
#[must_use]
pub fn columns_for(width: u16) -> usize {
    let usable = width.saturating_sub(2);
    usize::from((usable + GAP) / (MIN_CELL_W + GAP)).clamp(1, MAX_COLS)
}

/// Renders the overview screen into `area`, laying out both zones as flex grids
/// that consume the full width and height.
pub fn render(frame: &mut Frame, area: Rect, app: &App, palette: Palette) {
    let projects = app.overview_project_cards();
    let services = app.overview_service_cards();
    let cols = columns_for(area.width);

    let mut zones: Vec<(String, &[OverviewCard])> = Vec::with_capacity(2);
    if !projects.is_empty() {
        zones.push((t!("overview.projects").into_owned(), projects.as_slice()));
    }
    zones.push((t!("overview.services").into_owned(), services.as_slice()));

    let weights: Vec<Constraint> = zones
        .iter()
        .map(|(_, cards)| {
            let rows = cards.len().div_ceil(cols).max(1);
            Constraint::Fill(u16::try_from(rows + 1).unwrap_or(u16::MAX))
        })
        .collect();

    let zone_areas = Layout::vertical(weights).spacing(1).split(area);

    let mut index = 0usize;
    for ((title, cards), zone) in zones.iter().zip(zone_areas.iter()) {
        render_zone(
            frame,
            *zone,
            title,
            cards,
            cols,
            &mut index,
            app.overview_selected,
            palette
        );
    }
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
    palette: Palette
) {
    if area.height < 2 || cards.is_empty() {
        *index += cards.len();
        return;
    }

    let header = Paragraph::new(Line::from(Span::styled(
        format!("  {}", title.to_uppercase()),
        Style::default()
            .fg(palette.header)
            .add_modifier(Modifier::BOLD)
    )));
    let vsplit = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(area);
    frame.render_widget(header, vsplit[0]);

    let rows = cards.len().div_ceil(cols).max(1);
    let row_areas = Layout::vertical(vec![Constraint::Fill(1); rows])
        .spacing(1)
        .split(vsplit[1]);

    for (r, row_area) in row_areas.iter().enumerate() {
        let cells = Layout::horizontal(vec![Constraint::Fill(1); cols])
            .spacing(GAP)
            .split(*row_area);
        for (c, cell) in cells.iter().enumerate() {
            let i = r * cols + c;
            let Some(card) = cards.get(i) else {
                break;
            };
            render_card(frame, *cell, card, *index + i == selected, palette);
        }
    }

    *index += cards.len();
}

fn render_card(
    frame: &mut Frame,
    rect: Rect,
    card: &OverviewCard,
    selected: bool,
    palette: Palette
) {
    if rect.height < 3 || rect.width < 6 {
        return;
    }
    let border = if selected {
        palette.accent
    } else {
        palette.border
    };
    let icon = NERD_ICONS
        .get(card.icon.index())
        .copied()
        .unwrap_or("\u{25A0}");

    let inner_w = usize::from(rect.width.saturating_sub(4));
    let title_line = Line::from(vec![
        Span::styled(format!("{icon}  "), Style::default().fg(palette.accent)),
        Span::styled(
            truncate(&card.label, inner_w.saturating_sub(3)),
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

    let mut lines = vec![title_line];
    let pad = usize::from(rect.height).saturating_sub(3);
    for _ in 0..pad {
        lines.push(Line::from(""));
    }
    lines.push(count_line);

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
