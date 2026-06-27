// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Network in/out rolling sparkline widgets.

use std::collections::VecDeque;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Sparkline}
};

/// Renders network-in and network-out sparklines.
pub fn render(frame: &mut Frame, area: Rect, net_in: &VecDeque<u64>, net_out: &VecDeque<u64>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_single(frame, chunks[0], "Net In", net_in, Color::Green);
    render_single(frame, chunks[1], "Net Out", net_out, Color::Blue);
}

fn render_single(frame: &mut Frame, area: Rect, label: &str, data: &VecDeque<u64>, color: Color) {
    let values: Vec<u64> = data.iter().copied().collect();
    let peak = values.iter().copied().max().unwrap_or(0);

    let block = Block::default()
        .title(format!("{label}  peak {peak} B/s"))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let sparkline = Sparkline::default()
        .block(block)
        .data(&values)
        .style(Style::default().fg(color));

    frame.render_widget(sparkline, area);
}
