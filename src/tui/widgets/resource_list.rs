// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Resource list widget — shows selected resources in a scrollable list.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState}
};

use crate::tui::app::{App, ResourceTab};

/// Renders the resource list panel.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = match app.active_tab {
        ResourceTab::Servers => {
            app.servers
                .iter()
                .map(|s| {
                    let status_color = match s.status.as_str() {
                        "Running" => Color::Green,
                        "Stopped" => Color::Red,
                        _ => Color::Yellow,
                    };
                    let line = Line::from(vec![
                        Span::raw(format!("{} ", if s.status == "Running" { "▶" } else { "○" })),
                        Span::styled(&s.name, Style::default().fg(Color::White)),
                        Span::raw("  "),
                        Span::styled(
                            format!("[{}]", s.status),
                            Style::default().fg(status_color)
                        ),
                    ]);
                    ListItem::new(line)
                })
                .collect()
        }
        ResourceTab::Databases => {
            app.databases
                .iter()
                .map(|d| {
                    let line = Line::from(vec![
                        Span::raw("● "),
                        Span::styled(&d.name, Style::default().fg(Color::White)),
                        Span::raw("  "),
                        Span::styled(
                            format!("[{}]", d.engine),
                            Style::default().fg(Color::Cyan)
                        ),
                    ]);
                    ListItem::new(line)
                })
                .collect()
        }
        ResourceTab::S3 => {
            app.s3_storages
                .iter()
                .map(|s| {
                    let line = Line::from(vec![
                        Span::raw("📦 "),
                        Span::styled(&s.name, Style::default().fg(Color::White)),
                        Span::raw("  "),
                        Span::styled(&s.region, Style::default().fg(Color::Yellow)),
                    ]);
                    ListItem::new(line)
                })
                .collect()
        }
        ResourceTab::Kubernetes => {
            app.k8s_clusters
                .iter()
                .map(|c| {
                    let line = Line::from(vec![
                        Span::raw("☸ "),
                        Span::styled(&c.name, Style::default().fg(Color::White)),
                        Span::raw("  "),
                        Span::styled(
                            format!("[v{}]", c.version),
                            Style::default().fg(Color::Magenta)
                        ),
                    ]);
                    ListItem::new(line)
                })
                .collect()
        }
        ResourceTab::Projects => {
            app.projects
                .iter()
                .map(|p| {
                    let line = Line::from(vec![
                        Span::raw("📁 "),
                        Span::styled(&p.name, Style::default().fg(Color::White)),
                        Span::raw("  "),
                        Span::styled(
                            format!("[{} servers]", p.server_count),
                            Style::default().fg(Color::DarkGray)
                        ),
                    ]);
                    ListItem::new(line)
                })
                .collect()
        }
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Resources "))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol(">> ");

    let mut state = ListState::default();
    state.select(Some(app.selected));
    frame.render_stateful_widget(list, area, &mut state);
}
