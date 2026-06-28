// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Main draw function — composes all widgets into the terminal layout.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs}
};

use super::app::App;

/// Renders the full dashboard into the given frame area.
pub fn draw(frame: &mut Frame, app: &App) {
    let size = frame.area();
    let palette = get_palette(app);

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Account header
            Constraint::Length(3), // Resource tabs
            Constraint::Min(10),   // Content
            Constraint::Length(3)  // Status bar
        ])
        .split(size);

    render_account_header(frame, main_chunks[0], app, &palette);
    render_tabs(frame, main_chunks[1], app, &palette);
    render_content(frame, main_chunks[2], app, &palette);
    render_status_bar(frame, main_chunks[3], app, &palette);

    if app.show_help {
        render_help_overlay(frame, size, &palette);
    }
}

fn get_palette(app: &App) -> super::themes::Palette {
    app.theme.palette()
}

fn render_account_header(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    palette: &super::themes::Palette
) {
    let account_id = app.account.account_id;
    let balance = &app.account.balance;
    let status = &app.account.status;

    let line = Line::from(vec![
        Span::styled(
            "twc-rs",
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD)
        ),
        Span::raw("  "),
        Span::styled(
            format!("Account: {account_id:.0}"),
            Style::default().fg(palette.header)
        ),
        Span::raw("  "),
        Span::styled(
            format!("Balance: {balance}"),
            Style::default().fg(palette.success)
        ),
        Span::raw("  "),
        Span::styled(
            format!("Status: {status}"),
            Style::default().fg(palette.warning)
        ),
    ]);

    let paragraph = Paragraph::new(line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette.border))
    );
    frame.render_widget(paragraph, area);
}

fn render_tabs(frame: &mut Frame, area: Rect, app: &App, palette: &super::themes::Palette) {
    let titles: Vec<Line<'static>> = super::app::ResourceTab::names()
        .iter()
        .map(|t| Line::from(Span::styled(*t, Style::default())))
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .title(" Resources ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(palette.border))
        )
        .select(app.active_tab.index())
        .style(Style::default())
        .highlight_style(
            Style::default()
                .fg(palette.tab_active)
                .add_modifier(Modifier::BOLD)
        );

    frame.render_widget(tabs, area);
}

fn render_content(frame: &mut Frame, area: Rect, app: &App, _palette: &super::themes::Palette) {
    if app.is_loading {
        let spinner = super::widgets::spinner::current_frame();
        let text = Line::from(vec![spinner, Span::raw(" Loading resources...")]);
        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(" Loading "))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(paragraph, area);
        return;
    }

    app.widgets.render_all(frame, area, app);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App, palette: &super::themes::Palette) {
    let left = "k/j ↑↓  h/l ←→  Tab tabs  g first  $ last  r refresh  ? help  q quit";
    let right = match (&app.error_message, &app.status_message) {
        (Some(err), _) => err.clone(),
        (_, Some(msg)) => msg.clone(),
        _ => String::new()
    };
    let color = if app.error_message.is_some() {
        palette.error
    } else {
        palette.success
    };
    let line = Line::from(vec![
        Span::styled(left, Style::default().fg(palette.dim)),
        Span::raw("  "),
        Span::styled(right, Style::default().fg(color)),
    ]);
    let paragraph = Paragraph::new(line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette.border))
    );
    frame.render_widget(paragraph, area);
}

fn render_help_overlay(frame: &mut Frame, area: Rect, palette: &super::themes::Palette) {
    let help_text = vec![
        Line::from(Span::styled(
            "twc-rs dashboard — Keyboard Shortcuts",
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
        Line::from("  q / Esc      Quit"),
        Line::from("  ↑ ↓ / k j   Navigate resource list"),
        Line::from("  h / ←       Move left"),
        Line::from("  l / →       Move right"),
        Line::from("  Tab          Cycle resource tabs"),
        Line::from("  g            Go to first"),
        Line::from("  $            Go to last"),
        Line::from("  r            Force refresh"),
        Line::from("  ?            Toggle this help"),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(palette.accent))
        )
        .style(Style::default().bg(palette.bg));

    let popup_area = Rect {
        x:      area.width / 4,
        y:      area.height / 4,
        width:  area.width / 2,
        height: area.height / 2
    };

    frame.render_widget(ratatui::widgets::Clear, popup_area);
    frame.render_widget(help, popup_area);
}
