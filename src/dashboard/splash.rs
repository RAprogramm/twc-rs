// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Startup splash screen drawn while the first data load runs.

pub(crate) fn draw_splash(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>
) {
    use ratatui::{
        layout::{Constraint, Direction, Layout},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph}
    };

    let _ = terminal.draw(|f| {
        let size = f.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3)
            ])
            .split(size);

        // Header
        let header = Line::from(vec![
            Span::styled(
                "twc-rs",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            ),
            Span::raw(" v"),
            Span::raw(env!("CARGO_PKG_VERSION")),
        ]);
        let header_widget = Paragraph::new(header).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
        );
        f.render_widget(header_widget, chunks[0]);

        // ASCII art + loading
        let ascii_art = vec![
            Line::from(""),
            Line::from(Span::styled(
                "    ╔══════════════════════════════════╗",
                Style::default().fg(Color::Cyan)
            )),
            Line::from(Span::styled(
                "    ║        Timeweb Cloud CLI          ║",
                Style::default().fg(Color::Cyan)
            )),
            Line::from(Span::styled(
                "    ╚══════════════════════════════════╝",
                Style::default().fg(Color::Cyan)
            )),
            Line::from(""),
            Line::from(Span::styled(
                "    Loading resources...",
                Style::default().fg(Color::Yellow)
            )),
            Line::from(Span::styled(
                "    (this may take a moment on first run)",
                Style::default().fg(Color::DarkGray)
            )),
        ];
        let art_widget = Paragraph::new(ascii_art).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
        );
        f.render_widget(art_widget, chunks[1]);

        // Status bar
        let status = Line::from(Span::styled(
            "Fetching account, servers, databases, S3, k8s, projects...",
            Style::default().fg(Color::DarkGray)
        ));
        let status_widget = Paragraph::new(status).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
        );
        f.render_widget(status_widget, chunks[2]);
    });
}
