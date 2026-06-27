// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Main draw function — composes all widgets into the terminal layout.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs}
};

use super::{
    app::App,
    widgets::{gauges, server_list, sparkline}
};

/// Renders the full dashboard into the given frame area.
pub fn draw(frame: &mut Frame, app: &App) {
    let size = frame.area();

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(10),   // Content
            Constraint::Length(3)  // Status bar
        ])
        .split(size);

    render_tabs(frame, main_chunks[0], app);
    render_content(frame, main_chunks[1], app);
    render_status_bar(frame, main_chunks[2], app);

    if app.show_help {
        render_help_overlay(frame, size);
    }
}

fn render_tabs(frame: &mut Frame, area: Rect, app: &App) {
    let titles: Vec<Line<'static>> = super::app::Tab::names()
        .iter()
        .map(|t| Line::from(Span::styled(*t, Style::default())))
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .title("twc-rs monitor")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
        )
        .select(app.active_tab.index())
        .style(Style::default())
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        );

    frame.render_widget(tabs, area);
}

fn render_content(frame: &mut Frame, area: Rect, app: &App) {
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Server list
            Constraint::Percentage(70)  // Details
        ])
        .split(area);

    server_list::render(frame, content_chunks[0], &app.servers, app.selected);

    let detail_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // Gauges
            Constraint::Percentage(50)  // Sparklines
        ])
        .split(content_chunks[1]);

    let cpu = app.cpu_history.back().copied().unwrap_or(0.0) / 100.0;
    let ram = app.ram_history.back().copied().unwrap_or(0.0) / 100.0;
    let ram_gb = ram * 4.0; // placeholder: assume 4 GB total
    let disk = 0.4; // placeholder
    let disk_gb = 40.0; // placeholder

    gauges::render(
        frame,
        detail_chunks[0],
        cpu,
        ram,
        ram_gb,
        4.0,
        disk,
        disk_gb,
        100.0
    );

    sparkline::render(
        frame,
        detail_chunks[1],
        &app.net_in_history,
        &app.net_out_history
    );
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let left = "↑↓ select  Tab switch  r refresh  ? help  q quit";
    let right = match &app.status_message {
        Some(msg) => msg.clone(),
        None => String::new()
    };
    let line = Line::from(vec![
        Span::styled(left, Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled(right, Style::default().fg(Color::Green)),
    ]);
    let paragraph = Paragraph::new(line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
    );
    frame.render_widget(paragraph, area);
}

fn render_help_overlay(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled(
            "twc-rs monitor — Keyboard Shortcuts",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
        Line::from("  q / Esc      Quit"),
        Line::from("  ↑ ↓ / j k   Navigate server list"),
        Line::from("  Tab          Cycle top-level tabs"),
        Line::from("  r            Force refresh"),
        Line::from("  ?            Toggle this help"),
        Line::from("  Enter        Drill into selected resource"),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
        )
        .style(Style::default().bg(Color::Black));

    let popup_area = Rect {
        x:      area.width / 4,
        y:      area.height / 4,
        width:  area.width / 2,
        height: area.height / 2
    };

    frame.render_widget(ratatui::widgets::Clear, popup_area);
    frame.render_widget(help, popup_area);
}
