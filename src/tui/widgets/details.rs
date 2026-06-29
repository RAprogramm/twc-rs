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
///
/// # Arguments
///
/// * `frame` - The render frame.
/// * `area` - The area to render in.
/// * `app` - The application state.
/// * `border_color` - Color for the panel border.
pub fn render(frame: &mut Frame, area: Rect, app: &App, border_color: Color) {
    let palette = app.theme.palette();

    let text = match app.active_tab {
        ResourceTab::Servers => render_server_details(app, palette),
        ResourceTab::Databases => render_database_details(app, palette),
        ResourceTab::S3 => render_s3_details(app, palette),
        ResourceTab::Kubernetes => render_k8s_details(app, palette),
        ResourceTab::Projects => render_project_details(app, palette),
        ResourceTab::Balancers => render_generic_details("balancers", palette),
        ResourceTab::Registry => render_generic_details("registries", palette),
        ResourceTab::Domains => render_generic_details("domains", palette),
        ResourceTab::Firewall => render_generic_details("firewalls", palette),
        ResourceTab::FloatingIps => render_generic_details("floating_ips", palette),
        ResourceTab::Images => render_generic_details("images", palette),
        ResourceTab::NetworkDrives => render_generic_details("network_drives", palette),
        ResourceTab::Vpc => render_generic_details("vpcs", palette),
        ResourceTab::DedicatedServers => render_generic_details("dedicated_servers", palette),
        ResourceTab::Mail => render_generic_details("mails", palette),
        ResourceTab::Apps => render_generic_details("apps", palette),
        ResourceTab::AiAgents => render_generic_details("ai_agents", palette),
        ResourceTab::KnowledgeBases => render_generic_details("knowledge_bases", palette),
        ResourceTab::SshKeys => render_generic_details("ssh_keys", palette),
        ResourceTab::Finances => render_generic_details("finances", palette)
    };

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Line::from(Span::styled(
                " Details ",
                Style::default()
                    .fg(palette.title)
                    .add_modifier(Modifier::BOLD)
            )))
    );
    frame.render_widget(paragraph, area);
}

fn render_server_details(app: &App, palette: crate::tui::themes::Palette) -> Vec<Line<'static>> {
    if app.servers.is_empty() {
        return vec![Line::from(Span::styled(
            "No servers available",
            Style::default().fg(palette.dim)
        ))];
    }

    let server = &app.servers[app.selected.min(app.servers.len() - 1)];
    vec![
        Line::from(Span::styled(
            format!("Name: {}", server.name),
            Style::default().fg(palette.fg)
        )),
        Line::from(Span::styled(
            format!("ID: {}", server.id),
            Style::default().fg(palette.fg)
        )),
        Line::from(Span::styled(
            format!("Status: {}", server.status),
            Style::default().fg(palette.success)
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Resources:",
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD)
        )),
        Line::from(Span::styled(
            format!("  CPU: {} cores", server.cpu),
            Style::default().fg(palette.fg)
        )),
        Line::from(Span::styled(
            format!("  RAM: {} MB", server.ram_mb),
            Style::default().fg(palette.fg)
        )),
        Line::from(Span::styled(
            format!("  Disk: {} GB", server.disk_gb),
            Style::default().fg(palette.fg)
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("Location: {}", server.location),
            Style::default().fg(palette.warning)
        )),
    ]
}

fn render_database_details(app: &App, palette: crate::tui::themes::Palette) -> Vec<Line<'static>> {
    if app.databases.is_empty() {
        return vec![Line::from(Span::styled(
            "No databases available",
            Style::default().fg(palette.dim)
        ))];
    }

    let db = &app.databases[app.selected.min(app.databases.len() - 1)];
    vec![
        Line::from(Span::styled(
            format!("Name: {}", db.name),
            Style::default().fg(palette.fg)
        )),
        Line::from(Span::styled(
            format!("ID: {}", db.id),
            Style::default().fg(palette.fg)
        )),
        Line::from(Span::styled(
            format!("Engine: {}", db.engine),
            Style::default().fg(palette.accent)
        )),
        Line::from(Span::styled(
            format!("Status: {}", db.status),
            Style::default().fg(palette.success)
        )),
        Line::from(Span::styled(
            format!("Size: {} MB", db.size_mb),
            Style::default().fg(palette.fg)
        )),
    ]
}

fn render_s3_details(app: &App, palette: crate::tui::themes::Palette) -> Vec<Line<'static>> {
    if app.s3_storages.is_empty() {
        return vec![Line::from(Span::styled(
            "No S3 storages available",
            Style::default().fg(palette.dim)
        ))];
    }

    let storage = &app.s3_storages[app.selected.min(app.s3_storages.len() - 1)];
    vec![
        Line::from(Span::styled(
            format!("Name: {}", storage.name),
            Style::default().fg(palette.fg)
        )),
        Line::from(Span::styled(
            format!("ID: {}", storage.id),
            Style::default().fg(palette.fg)
        )),
        Line::from(Span::styled(
            format!("Region: {}", storage.region),
            Style::default().fg(palette.warning)
        )),
        Line::from(Span::styled(
            format!("Size: {} KB", storage.size_bytes / 1024),
            Style::default().fg(palette.fg)
        )),
        Line::from(Span::styled(
            format!("Buckets: {}", storage.bucket_count),
            Style::default().fg(palette.fg)
        )),
    ]
}

fn render_k8s_details(app: &App, palette: crate::tui::themes::Palette) -> Vec<Line<'static>> {
    if app.k8s_clusters.is_empty() {
        return vec![Line::from(Span::styled(
            "No Kubernetes clusters available",
            Style::default().fg(palette.dim)
        ))];
    }

    let cluster = &app.k8s_clusters[app.selected.min(app.k8s_clusters.len() - 1)];
    vec![
        Line::from(Span::styled(
            format!("Name: {}", cluster.name),
            Style::default().fg(palette.fg)
        )),
        Line::from(Span::styled(
            format!("ID: {}", cluster.id),
            Style::default().fg(palette.fg)
        )),
        Line::from(Span::styled(
            format!("Version: v{}", cluster.version),
            Style::default().fg(palette.accent)
        )),
        Line::from(Span::styled(
            format!("Status: {}", cluster.status),
            Style::default().fg(palette.success)
        )),
        Line::from(Span::styled(
            format!("Nodes: {}", cluster.node_count),
            Style::default().fg(palette.fg)
        )),
    ]
}

fn render_project_details(app: &App, palette: crate::tui::themes::Palette) -> Vec<Line<'static>> {
    if app.projects.is_empty() {
        return vec![Line::from(Span::styled(
            "No projects available",
            Style::default().fg(palette.dim)
        ))];
    }

    let project = &app.projects[app.selected.min(app.projects.len() - 1)];
    vec![
        Line::from(Span::styled(
            format!("Name: {}", project.name),
            Style::default().fg(palette.fg)
        )),
        Line::from(Span::styled(
            format!("ID: {}", project.id),
            Style::default().fg(palette.fg)
        )),
        Line::from(Span::styled(
            format!("Servers: {}", project.server_count),
            Style::default().fg(palette.fg)
        )),
    ]
}

fn render_generic_details(
    resource: &str,
    palette: crate::tui::themes::Palette
) -> Vec<Line<'static>> {
    vec![Line::from(Span::styled(
        format!("No {resource} data available"),
        Style::default().fg(palette.dim)
    ))]
}

/// Widget wrapper for the details panel.
pub struct DetailsWidget {
    enabled: bool
}

impl DetailsWidget {
    /// Creates a new details widget with enabled state.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether the widget is initially visible.
    #[must_use]
    pub const fn new(enabled: bool) -> Self {
        Self {
            enabled
        }
    }
}

impl crate::tui::widgets::Widget for DetailsWidget {
    fn id(&self) -> &'static str {
        "details"
    }

    fn name(&self) -> &'static str {
        "Details"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        let border_color = if app.focus == crate::tui::app::Focus::Details {
            app.theme.palette().accent
        } else {
            app.theme.palette().border
        };
        render(frame, area, app, border_color);
    }
}
