// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Details widget — shows information about the selected resource.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph}
};

use crate::tui::{
    app::{App, ResourceTab},
    themes::Palette,
    widgets::resource_list::server_status_view
};

const KEY_WIDTH: usize = 10;
const RULE_WIDTH: usize = 32;

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
            .border_type(BorderType::Rounded)
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

/// Builds the bold heading line shown at the top of a populated panel.
fn heading(name: &str, palette: Palette) -> Line<'static> {
    Line::from(Span::styled(
        name.to_string(),
        Style::default()
            .fg(palette.title)
            .add_modifier(Modifier::BOLD)
    ))
}

/// Builds a dim horizontal rule used to separate sections.
fn rule(palette: Palette) -> Line<'static> {
    Line::from(Span::styled(
        "\u{2500}".repeat(RULE_WIDTH),
        Style::default().fg(palette.dim)
    ))
}

/// Builds a dim, bold section header line.
fn section(label: &str, palette: Palette) -> Line<'static> {
    Line::from(Span::styled(
        label.to_string(),
        Style::default()
            .fg(palette.header)
            .add_modifier(Modifier::BOLD)
    ))
}

/// Builds a key/value row, dimming the key via the palette's dim color.
fn kv(key: &str, value: String, value_style: Style, palette: Palette) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{key:<KEY_WIDTH$}"),
            Style::default().fg(palette.dim)
        ),
        Span::styled(value, value_style),
    ])
}

/// Builds a status row rendered as a colored `\u{25CF} label` chip.
fn chip(key: &str, label: &str, color: Color, palette: Palette) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{key:<KEY_WIDTH$}"),
            Style::default().fg(palette.dim)
        ),
        Span::styled(
            format!("\u{25CF} {label}"),
            Style::default().fg(color).add_modifier(Modifier::BOLD)
        ),
    ])
}

/// Builds a centered, dim empty-state notice.
fn empty(message: &str, palette: Palette) -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {message}"),
            Style::default()
                .fg(palette.dim)
                .add_modifier(Modifier::ITALIC)
        )),
    ]
}

fn accent(palette: Palette) -> Style {
    Style::default().fg(palette.accent)
}

fn name_style(palette: Palette) -> Style {
    Style::default().fg(palette.fg).add_modifier(Modifier::BOLD)
}

fn warn(palette: Palette) -> Style {
    Style::default().fg(palette.warning)
}

fn render_server_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.servers.is_empty() {
        return empty("No servers available", palette);
    }

    let server = &app.servers[app.selected_real_index().min(app.servers.len() - 1)];
    let (_, color, label) = server_status_view(&server.status, &palette);
    vec![
        heading(&server.name, palette),
        rule(palette),
        kv("ID", format!("#{}", server.id), accent(palette), palette),
        chip("Status", &label, color, palette),
        kv("Location", server.location.clone(), warn(palette), palette),
        Line::from(""),
        section("Resources", palette),
        kv(
            "CPU",
            format!("{} cores", server.cpu),
            accent(palette),
            palette
        ),
        kv(
            "RAM",
            format!("{} MB", server.ram_mb),
            accent(palette),
            palette
        ),
        kv(
            "Disk",
            format!("{} GB", server.disk_gb),
            accent(palette),
            palette
        ),
    ]
}

fn render_database_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.databases.is_empty() {
        return empty("No databases available", palette);
    }

    let db = &app.databases[app.selected_real_index().min(app.databases.len() - 1)];
    vec![
        heading(&db.name, palette),
        rule(palette),
        kv("ID", format!("#{}", db.id), accent(palette), palette),
        kv("Engine", db.engine.clone(), accent(palette), palette),
        chip(
            "Status",
            &db.status.to_lowercase(),
            palette.success,
            palette
        ),
        kv(
            "Size",
            format!("{} MB", db.size_mb),
            name_style(palette),
            palette
        ),
    ]
}

fn render_s3_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.s3_storages.is_empty() {
        return empty("No S3 storages available", palette);
    }

    let storage = &app.s3_storages[app.selected_real_index().min(app.s3_storages.len() - 1)];
    vec![
        heading(&storage.name, palette),
        rule(palette),
        kv("ID", format!("#{}", storage.id), accent(palette), palette),
        kv("Region", storage.region.clone(), warn(palette), palette),
        kv(
            "Size",
            format!("{} KB", storage.size_bytes / 1024),
            name_style(palette),
            palette
        ),
        kv(
            "Buckets",
            storage.bucket_count.to_string(),
            accent(palette),
            palette
        ),
    ]
}

fn render_k8s_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.k8s_clusters.is_empty() {
        return empty("No Kubernetes clusters available", palette);
    }

    let cluster = &app.k8s_clusters[app.selected_real_index().min(app.k8s_clusters.len() - 1)];
    vec![
        heading(&cluster.name, palette),
        rule(palette),
        kv("ID", format!("#{}", cluster.id), accent(palette), palette),
        kv(
            "Version",
            format!("v{}", cluster.version),
            accent(palette),
            palette
        ),
        chip(
            "Status",
            &cluster.status.to_lowercase(),
            palette.success,
            palette
        ),
        kv(
            "Nodes",
            cluster.node_count.to_string(),
            name_style(palette),
            palette
        ),
    ]
}

fn render_project_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.projects.is_empty() {
        return empty("No projects available", palette);
    }

    let project = &app.projects[app.selected_real_index().min(app.projects.len() - 1)];
    vec![
        heading(&project.name, palette),
        rule(palette),
        kv("ID", format!("#{}", project.id), accent(palette), palette),
        kv(
            "Servers",
            project.server_count.to_string(),
            name_style(palette),
            palette
        ),
    ]
}

fn render_generic_details(resource: &str, palette: Palette) -> Vec<Line<'static>> {
    empty(&format!("No {resource} data available"), palette)
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
