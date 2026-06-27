// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Server list widget — left panel with selectable items.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState}
};

use crate::tui::app::ServerSummary;

/// Renders the server list panel.
pub fn render(frame: &mut Frame, area: Rect, servers: &[ServerSummary], selected: usize) {
    let items: Vec<ListItem> = servers
        .iter()
        .enumerate()
        .map(|(i, server)| {
            let prefix = if i == selected { "▶ " } else { "  " };
            let status_color = match server.status.as_str() {
                "running" => Color::Green,
                "stopped" => Color::Red,
                "stopping" => Color::Yellow,
                _ => Color::Gray
            };
            let line = Line::from(vec![
                Span::raw(prefix),
                Span::styled(&server.name, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(&server.status, Style::default().fg(status_color)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let title = format!("Servers ({})", servers.len());
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol("");

    let mut state = ListState::default();
    state.select(Some(selected));

    frame.render_stateful_widget(list, area, &mut state);
}
