// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! The sidebar: a vertical navigation list with Projects and Services groups,
//! Nerd Font icons, right-aligned count badges and an accent selection bar.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph}
};
use rust_i18n::t;

use crate::tui::{
    app::{App, NavKind, Pane},
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

/// The Nerd Font glyph representing a resource category, shared with the
/// content card grid so both panes use one icon set.
#[must_use]
pub fn tab_icon(tab: crate::tui::app::ResourceTab) -> &'static str {
    NERD_ICONS.get(tab.index()).copied().unwrap_or("\u{25A0}")
}

/// Braille spinner frames for the per-group loading indicator, advanced by
/// the animation tick.
const SPINNER: [&str; 10] = [
    "\u{280B}", "\u{2819}", "\u{2839}", "\u{2838}", "\u{283C}", "\u{2834}", "\u{2826}",
    "\u{2827}", "\u{2807}", "\u{280F}"
];

/// Cells the sidebar spends beside the label: border, selection bar, icon,
/// gaps and the count badge.
const CHROME_W: u16 = 12;
/// Narrowest useful sidebar.
const MIN_W: u16 = 20;
/// Widest sidebar before it steals too much content space.
const MAX_W: u16 = 34;

/// Sidebar width for a terminal `total` cells wide: fits the longest label,
/// clamped, and never more than a third of the terminal.
#[must_use]
pub fn width_for(total: u16, longest_label: usize) -> u16 {
    let by_label = u16::try_from(longest_label)
        .unwrap_or(MAX_W)
        .saturating_add(CHROME_W)
        .clamp(MIN_W, MAX_W);
    by_label.min(total / 3).max(MIN_W).min(total)
}

/// Renders the sidebar navigation into `area`.
pub fn render(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let focused = app.pane == Pane::Sidebar;
    let border = if focused {
        palette.accent
    } else {
        palette.border
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width < 6 || inner.height == 0 {
        return;
    }

    let items = app.nav_items();
    let mut lines: Vec<Line> = Vec::with_capacity(items.len() + 6);
    let mut selected_line = 0usize;
    let mut last_group: Option<u8> = None;

    for (i, item) in items.iter().enumerate() {
        let group = match item.kind {
            NavKind::Create => 0u8,
            NavKind::Project(_) => 1,
            NavKind::Service(_) => 2,
            NavKind::Settings => 3
        };
        if last_group != Some(group) {
            if last_group.is_some() {
                lines.push(Line::from(""));
            }
            if group == 0 {
                last_group = Some(group);
                let selected = i == app.nav_selected;
                if selected {
                    selected_line = lines.len();
                }
                lines.push(nav_line(app, item, selected, focused, inner.width, palette));
                continue;
            }
            let header = match group {
                1 => t!("overview.projects"),
                2 => t!("overview.services"),
                _ => t!("sidebar.settings")
            };
            let loading = match group {
                1 => app.projects_pending > 0,
                2 => app.services_pending > 0,
                _ => false
            };
            let refreshing = !app.initial_cycle_done || app.manual_refresh_spin;
            let mut spans = vec![Span::styled(
                format!(" {}", header.to_uppercase()),
                Style::default()
                    .fg(palette.header)
                    .add_modifier(Modifier::BOLD)
            )];
            if loading && refreshing {
                let frame_idx = usize::try_from(app.anim_tick % SPINNER.len() as u64).unwrap_or(0);
                spans.push(Span::styled(
                    format!(" {}", SPINNER[frame_idx]),
                    Style::default().fg(palette.accent)
                ));
            }
            lines.push(Line::from(spans));
            last_group = Some(group);
        }

        let selected = i == app.nav_selected;
        if selected {
            selected_line = lines.len();
        }
        lines.push(nav_line(app, item, selected, focused, inner.width, palette));
    }

    let visible = usize::from(inner.height);
    let offset = if selected_line < visible {
        0
    } else {
        selected_line + 1 - visible
    };
    let text: Vec<Line> = lines.into_iter().skip(offset).collect();
    frame.render_widget(Paragraph::new(text), inner);
}

/// Builds one navigation row: selection bar, icon, label and a right-aligned
/// count badge, truncated to the sidebar width.
fn nav_line<'a>(
    app: &App,
    item: &crate::tui::app::NavItem,
    selected: bool,
    focused: bool,
    width: u16,
    palette: &Palette
) -> Line<'a> {
    let count = item.count.map(|c| c.to_string()).unwrap_or_default();
    let label_room = usize::from(width)
        .saturating_sub(6)
        .saturating_sub(count.len());
    let label = truncate(&item.label, label_room);
    let pad = usize::from(width)
        .saturating_sub(5 + label.chars().count() + count.len())
        .max(1);

    let bar = if selected { "\u{258E}" } else { " " };
    let bar_color = if selected && focused {
        palette.accent
    } else if selected {
        palette.dim
    } else {
        palette.bg
    };
    let label_style = if selected {
        Style::default().fg(palette.fg).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(palette.fg)
    };
    let count_color = if selected && app.pane == Pane::Sidebar {
        palette.accent
    } else {
        palette.dim
    };

    Line::from(vec![
        Span::styled(bar.to_string(), Style::default().fg(bar_color)),
        Span::styled(
            format!("{} ", item.glyph),
            Style::default().fg(palette.accent)
        ),
        Span::styled(label, label_style),
        Span::raw(" ".repeat(pad)),
        Span::styled(count, Style::default().fg(count_color)),
    ])
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
    use crate::tui::{app::ProjectSummary, themes::Theme};

    #[test]
    fn width_adapts_and_clamps() {
        assert!(width_for(120, 8) >= MIN_W);
        assert!(width_for(120, 60) <= MAX_W);
        assert!(width_for(120, 60) <= 40);
    }

    #[test]
    fn renders_across_sizes_without_panic() {
        let mut app = App::new(5);
        app.projects = (0..3)
            .map(|i| ProjectSummary {
                id: i,
                name: format!("proj-{i}"),
                ..ProjectSummary::default()
            })
            .collect();
        let palette = Theme::GruvboxDark.palette();
        for (w, h) in [(1, 1), (10, 5), (24, 30), (34, 60)] {
            let backend = TestBackend::new(w, h);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|frame| render(frame, Rect::new(0, 0, w, h), &app, &palette))
                .unwrap();
        }
    }
}
