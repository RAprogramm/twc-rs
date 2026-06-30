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
        ResourceTab::Balancers => render_balancer_details(app, palette),
        ResourceTab::Registry => render_registry_details(app, palette),
        ResourceTab::Domains => render_domain_details(app, palette),
        ResourceTab::Firewall => render_firewall_details(app, palette),
        ResourceTab::FloatingIps => render_floating_ip_details(app, palette),
        ResourceTab::Images => render_image_details(app, palette),
        ResourceTab::NetworkDrives => render_network_drive_details(app, palette),
        ResourceTab::Vpc => render_vpc_details(app, palette),
        ResourceTab::DedicatedServers => render_dedicated_details(app, palette),
        ResourceTab::Mail => render_mail_details(app, palette),
        ResourceTab::Apps => render_app_details(app, palette),
        ResourceTab::AiAgents => render_ai_agent_details(app, palette),
        ResourceTab::KnowledgeBases => render_knowledge_details(app, palette),
        ResourceTab::SshKeys => render_string_details(&app.ssh_keys, app, "SSH key", palette),
        ResourceTab::Finances => render_string_details(&app.finances, app, "Finance", palette)
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

fn render_string_details(
    items: &[String],
    app: &App,
    label: &str,
    palette: Palette
) -> Vec<Line<'static>> {
    if items.is_empty() {
        return empty(&format!("No {label} entries"), palette);
    }
    let item = &items[app.selected_real_index().min(items.len() - 1)];
    vec![
        heading(label, palette),
        rule(palette),
        kv("Value", item.clone(), name_style(palette), palette),
    ]
}

fn render_balancer_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.balancers.is_empty() {
        return empty("No balancers available", palette);
    }
    let b = &app.balancers[app.selected_real_index().min(app.balancers.len() - 1)];
    vec![
        heading(&b.name, palette),
        rule(palette),
        kv("ID", format!("#{}", b.id), accent(palette), palette),
        chip("Status", &b.status.to_lowercase(), palette.success, palette),
        kv("IP", b.ip.clone(), warn(palette), palette),
        kv("Location", b.location.clone(), warn(palette), palette),
    ]
}

fn render_registry_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.registries.is_empty() {
        return empty("No registries available", palette);
    }
    let r = &app.registries[app.selected_real_index().min(app.registries.len() - 1)];
    vec![
        heading(&r.name, palette),
        rule(palette),
        kv("ID", format!("#{}", r.id), accent(palette), palette),
        kv("Region", r.region.clone(), warn(palette), palette),
        kv(
            "Repos",
            r.repository_count.to_string(),
            name_style(palette),
            palette
        ),
    ]
}

fn render_domain_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.domains.is_empty() {
        return empty("No domains available", palette);
    }
    let d = &app.domains[app.selected_real_index().min(app.domains.len() - 1)];
    vec![
        heading(&d.name, palette),
        rule(palette),
        kv("ID", format!("#{}", d.id), accent(palette), palette),
        chip("Status", &d.status.to_lowercase(), palette.success, palette),
        kv(
            "Auto-prolong",
            if d.auto_prolong { "yes" } else { "no" }.to_string(),
            name_style(palette),
            palette
        ),
    ]
}

fn render_firewall_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.firewalls.is_empty() {
        return empty("No firewall groups available", palette);
    }
    let f = &app.firewalls[app.selected_real_index().min(app.firewalls.len() - 1)];
    vec![
        heading(&f.name, palette),
        rule(palette),
        kv("ID", format!("#{}", f.id), accent(palette), palette),
        kv(
            "Rules",
            f.rule_count.to_string(),
            name_style(palette),
            palette
        ),
        kv(
            "Resources",
            f.resource_count.to_string(),
            name_style(palette),
            palette
        ),
    ]
}

fn render_floating_ip_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.floating_ips.is_empty() {
        return empty("No floating IPs available", palette);
    }
    let f = &app.floating_ips[app.selected_real_index().min(app.floating_ips.len() - 1)];
    vec![
        heading(&f.ip, palette),
        rule(palette),
        kv("ID", format!("#{}", f.id), accent(palette), palette),
        chip("Status", &f.status.to_lowercase(), palette.success, palette),
        kv(
            "Bound to",
            f.server_name.clone(),
            name_style(palette),
            palette
        ),
    ]
}

fn render_image_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.images.is_empty() {
        return empty("No images available", palette);
    }
    let i = &app.images[app.selected_real_index().min(app.images.len() - 1)];
    vec![
        heading(&i.name, palette),
        rule(palette),
        kv("ID", format!("#{}", i.id), accent(palette), palette),
        chip("Status", &i.status.to_lowercase(), palette.success, palette),
        kv(
            "Size",
            format!("{} MB", i.size_mb),
            name_style(palette),
            palette
        ),
    ]
}

fn render_network_drive_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.network_drives.is_empty() {
        return empty("No network drives available", palette);
    }
    let n = &app.network_drives[app.selected_real_index().min(app.network_drives.len() - 1)];
    vec![
        heading(&n.name, palette),
        rule(palette),
        kv("ID", format!("#{}", n.id), accent(palette), palette),
        chip("Status", &n.status.to_lowercase(), palette.success, palette),
        kv(
            "Size",
            format!("{} GB", n.size_gb),
            name_style(palette),
            palette
        ),
    ]
}

fn render_vpc_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.vpcs.is_empty() {
        return empty("No VPCs available", palette);
    }
    let v = &app.vpcs[app.selected_real_index().min(app.vpcs.len() - 1)];
    vec![
        heading(&v.name, palette),
        rule(palette),
        kv("ID", format!("#{}", v.id), accent(palette), palette),
        chip("Status", &v.status.to_lowercase(), palette.success, palette),
        kv(
            "Subnets",
            v.subnet_count.to_string(),
            name_style(palette),
            palette
        ),
    ]
}

fn render_dedicated_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.dedicated_servers.is_empty() {
        return empty("No dedicated servers available", palette);
    }
    let d = &app.dedicated_servers[app
        .selected_real_index()
        .min(app.dedicated_servers.len() - 1)];
    vec![
        heading(&d.name, palette),
        rule(palette),
        kv("ID", format!("#{}", d.id), accent(palette), palette),
        chip("Status", &d.status.to_lowercase(), palette.success, palette),
        section("Resources", palette),
        kv(
            "CPU",
            format!("{} cores", d.cpu),
            name_style(palette),
            palette
        ),
        kv(
            "RAM",
            format!("{} MB", d.ram_mb),
            name_style(palette),
            palette
        ),
        kv(
            "Disk",
            format!("{} GB", d.disk_gb),
            name_style(palette),
            palette
        ),
    ]
}

fn render_mail_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.mails.is_empty() {
        return empty("No mailboxes available", palette);
    }
    let m = &app.mails[app.selected_real_index().min(app.mails.len() - 1)];
    vec![
        heading(&m.name, palette),
        rule(palette),
        kv("ID", format!("#{}", m.id), accent(palette), palette),
        chip("Status", &m.status.to_lowercase(), palette.success, palette),
        kv(
            "Mailboxes",
            m.mailbox_count.to_string(),
            name_style(palette),
            palette
        ),
    ]
}

fn render_app_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.apps.is_empty() {
        return empty("No apps available", palette);
    }
    let a = &app.apps[app.selected_real_index().min(app.apps.len() - 1)];
    vec![
        heading(&a.name, palette),
        rule(palette),
        kv("ID", format!("#{}", a.id), accent(palette), palette),
        chip("Status", &a.status.to_lowercase(), palette.success, palette),
        kv(
            "Deploys",
            a.deploy_count.to_string(),
            name_style(palette),
            palette
        ),
    ]
}

fn render_ai_agent_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.ai_agents.is_empty() {
        return empty("No AI agents available", palette);
    }
    let a = &app.ai_agents[app.selected_real_index().min(app.ai_agents.len() - 1)];
    vec![
        heading(&a.name, palette),
        rule(palette),
        kv("ID", format!("#{}", a.id), accent(palette), palette),
        chip("Status", &a.status.to_lowercase(), palette.success, palette),
        kv("Model", a.model.clone(), accent(palette), palette),
    ]
}

fn render_knowledge_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.knowledge_bases.is_empty() {
        return empty("No knowledge bases available", palette);
    }
    let k = &app.knowledge_bases[app.selected_real_index().min(app.knowledge_bases.len() - 1)];
    vec![
        heading(&k.name, palette),
        rule(palette),
        kv("ID", format!("#{}", k.id), accent(palette), palette),
        chip("Status", &k.status.to_lowercase(), palette.success, palette),
        kv(
            "Documents",
            k.document_count.to_string(),
            name_style(palette),
            palette
        ),
    ]
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
