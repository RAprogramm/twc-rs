// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Details widget — shows information about the selected resource.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph}
};

use crate::tui::app::{App, ResourceTab};

/// Renders the details panel for the selected resource.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let text = match app.active_tab {
        ResourceTab::Servers => render_server_details(app),
        ResourceTab::Databases => render_database_details(app),
        ResourceTab::S3 => render_s3_details(app),
        ResourceTab::Kubernetes => render_k8s_details(app),
        ResourceTab::Projects => render_project_details(app)
    };

    let paragraph =
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(" Details "));
    frame.render_widget(paragraph, area);
}

fn render_server_details(app: &App) -> Vec<Line<'static>> {
    if app.servers.is_empty() {
        return vec![Line::from(Span::styled(
            "No servers available",
            Style::default().fg(Color::DarkGray)
        ))];
    }

    let server = &app.servers[app.selected.min(app.servers.len() - 1)];
    vec![
        Line::from(Span::styled(
            format!("Name: {}", server.name),
            Style::default().fg(Color::White)
        )),
        Line::from(Span::styled(
            format!("ID: {}", server.id),
            Style::default().fg(Color::White)
        )),
        Line::from(Span::styled(
            format!("Status: {}", server.status),
            Style::default().fg(Color::Green)
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Resources:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        )),
        Line::from(Span::styled(
            format!("  CPU: {} cores", server.cpu),
            Style::default().fg(Color::White)
        )),
        Line::from(Span::styled(
            format!("  RAM: {} MB", server.ram_mb),
            Style::default().fg(Color::White)
        )),
        Line::from(Span::styled(
            format!("  Disk: {} GB", server.disk_gb),
            Style::default().fg(Color::White)
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("Location: {}", server.location),
            Style::default().fg(Color::Yellow)
        )),
    ]
}

fn render_database_details(app: &App) -> Vec<Line<'static>> {
    if app.databases.is_empty() {
        return vec![Line::from(Span::styled(
            "No databases available",
            Style::default().fg(Color::DarkGray)
        ))];
    }

    let db = &app.databases[app.selected.min(app.databases.len() - 1)];
    vec![
        Line::from(Span::styled(
            format!("Name: {}", db.name),
            Style::default().fg(Color::White)
        )),
        Line::from(Span::styled(
            format!("ID: {}", db.id),
            Style::default().fg(Color::White)
        )),
        Line::from(Span::styled(
            format!("Engine: {}", db.engine),
            Style::default().fg(Color::Cyan)
        )),
        Line::from(Span::styled(
            format!("Status: {}", db.status),
            Style::default().fg(Color::Green)
        )),
        Line::from(Span::styled(
            format!("Size: {} MB", db.size_mb),
            Style::default().fg(Color::White)
        )),
    ]
}

fn render_s3_details(app: &App) -> Vec<Line<'static>> {
    if app.s3_storages.is_empty() {
        return vec![Line::from(Span::styled(
            "No S3 storages available",
            Style::default().fg(Color::DarkGray)
        ))];
    }

    let storage = &app.s3_storages[app.selected.min(app.s3_storages.len() - 1)];
    vec![
        Line::from(Span::styled(
            format!("Name: {}", storage.name),
            Style::default().fg(Color::White)
        )),
        Line::from(Span::styled(
            format!("ID: {}", storage.id),
            Style::default().fg(Color::White)
        )),
        Line::from(Span::styled(
            format!("Region: {}", storage.region),
            Style::default().fg(Color::Yellow)
        )),
        Line::from(Span::styled(
            format!("Size: {} KB", storage.size_bytes / 1024),
            Style::default().fg(Color::White)
        )),
        Line::from(Span::styled(
            format!("Buckets: {}", storage.bucket_count),
            Style::default().fg(Color::White)
        )),
    ]
}

fn render_k8s_details(app: &App) -> Vec<Line<'static>> {
    if app.k8s_clusters.is_empty() {
        return vec![Line::from(Span::styled(
            "No Kubernetes clusters available",
            Style::default().fg(Color::DarkGray)
        ))];
    }

    let cluster = &app.k8s_clusters[app.selected.min(app.k8s_clusters.len() - 1)];
    vec![
        Line::from(Span::styled(
            format!("Name: {}", cluster.name),
            Style::default().fg(Color::White)
        )),
        Line::from(Span::styled(
            format!("ID: {}", cluster.id),
            Style::default().fg(Color::White)
        )),
        Line::from(Span::styled(
            format!("Version: v{}", cluster.version),
            Style::default().fg(Color::Magenta)
        )),
        Line::from(Span::styled(
            format!("Status: {}", cluster.status),
            Style::default().fg(Color::Green)
        )),
        Line::from(Span::styled(
            format!("Nodes: {}", cluster.node_count),
            Style::default().fg(Color::White)
        )),
    ]
}

fn render_project_details(app: &App) -> Vec<Line<'static>> {
    if app.projects.is_empty() {
        return vec![Line::from(Span::styled(
            "No projects available",
            Style::default().fg(Color::DarkGray)
        ))];
    }

    let project = &app.projects[app.selected.min(app.projects.len() - 1)];
    vec![
        Line::from(Span::styled(
            format!("Name: {}", project.name),
            Style::default().fg(Color::White)
        )),
        Line::from(Span::styled(
            format!("ID: {}", project.id),
            Style::default().fg(Color::White)
        )),
        Line::from(Span::styled(
            format!("Servers: {}", project.server_count),
            Style::default().fg(Color::White)
        )),
    ]
}
