// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! The bottom status bar: mode indicator, key hints and transient messages.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph}
};
use rust_i18n::t;

use crate::tui::{app::App, themes::Palette};

/// Renders the status bar with mode indicator and available keys.
///
/// # Arguments
///
/// * `frame` - The render frame.
/// * `area` - The status bar area rectangle.
/// * `app` - The application state.
/// * `palette` - The theme color palette.
pub(super) fn render_status_bar(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let tab = app.active_tab.display_name();

    let key = |k: &'static str| {
        Span::styled(
            k,
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD)
        )
    };
    let lbl = |t: String| Span::styled(t, Style::default().fg(palette.dim));

    let mut spans = vec![
        Span::styled(
            format!(" {tab} "),
            Style::default()
                .fg(palette.bg)
                .bg(palette.tab_active)
                .add_modifier(Modifier::BOLD)
        ),
        Span::raw("  "),
        key("\u{21e5}/\u{21e4}"),
        lbl(format!(" {}   ", t!("ui.status_tabs"))),
        key("\u{2190}\u{2192}\u{2191}\u{2193}"),
        lbl(format!(" {}   ", t!("ui.status_move"))),
        key("\u{23ce}"),
        lbl(format!(" {}   ", t!("ui.status_open"))),
        key("/"),
        lbl(format!(" {}   ", t!("ui.status_filter"))),
        key("^K"),
        lbl(format!(" {}   ", t!("ui.status_cmds"))),
        key("?"),
        lbl(format!(" {}   ", t!("ui.status_help"))),
        key("Q"),
        lbl(format!(" {}", t!("ui.status_quit"))),
    ];

    let message = match (&app.error_message, &app.status_message) {
        (Some(err), _) => Some((err.clone(), palette.error)),
        (_, Some(msg)) => Some((msg.clone(), palette.success)),
        _ => None
    };
    if let Some((text, color)) = message {
        spans.push(Span::raw("   "));
        spans.push(Span::styled("● ", Style::default().fg(color)));
        spans.push(Span::styled(text, Style::default().fg(color)));
    }

    let paragraph = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(palette.border))
    );
    frame.render_widget(paragraph, area);
}
