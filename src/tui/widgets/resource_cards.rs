// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Maps the active tab's resources to reusable [`GridCard`]s for the card grid.

use rust_i18n::t;

use crate::tui::{
    app::{App, ProjectSummary, RegistrySummary, ResourceTab},
    themes::Palette,
    widgets::{
        card_grid::GridCard,
        resource_list::{server_status_view, status_view},
        sidebar::tab_icon
    }
};

/// Builds the local preview cards for a highlighted project: one card per
/// resource type it contains, with the loaded count as the badge.
#[must_use]
pub fn project_preview(project: &ProjectSummary) -> Vec<GridCard> {
    let entries: [(ResourceTab, i32); 7] = [
        (ResourceTab::Servers, project.server_count),
        (ResourceTab::Databases, project.database_count),
        (ResourceTab::S3, project.bucket_count),
        (ResourceTab::Kubernetes, project.cluster_count),
        (ResourceTab::Balancers, project.balancer_count),
        (ResourceTab::DedicatedServers, project.dedicated_count),
        (ResourceTab::Apps, project.app_count)
    ];
    entries
        .into_iter()
        .filter(|(_, count)| *count > 0)
        .map(|(tab, count)| {
            GridCard::new(tab.display_name().into_owned())
                .icon(tab_icon(tab))
                .meta(t!("resource_list.count_resources", n => count).to_string())
        })
        .collect()
}

/// Integral used-disk percentage of a registry, treating a zero disk as free.
const fn registry_used_percent(registry: &RegistrySummary) -> i64 {
    if registry.disk_size <= 0 {
        0
    } else {
        registry.disk_used * 100 / registry.disk_size
    }
}

/// Builds one [`GridCard`] per item on the active tab, in list order.
// JUSTIFY: One arm per resource type; each maps flat fields onto a card.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn build(app: &App, palette: &Palette) -> Vec<GridCard> {
    let icon = tab_icon(app.active_tab);
    let card = |title: &str| GridCard::new(title).icon(icon);
    match app.active_tab {
        ResourceTab::Servers => app
            .servers
            .iter()
            .map(|s| {
                let (_, color, label) = server_status_view(&s.status, palette);
                card(&s.name).status(color, label).meta(format!(
                    "{}c · {} · {}",
                    s.cpu,
                    crate::tui::humanize::megabytes(i64::from(s.ram_mb)),
                    crate::tui::humanize::location(&s.location)
                ))
            })
            .collect(),
        ResourceTab::Databases => app
            .databases
            .iter()
            .map(|d| {
                let (color, label) = status_view(&d.status, palette);
                card(&d.name).status(color, label).meta(format!(
                    "{} · {}",
                    d.engine,
                    crate::tui::humanize::megabytes(d.size_mb)
                ))
            })
            .collect(),
        ResourceTab::S3 => app
            .s3_storages
            .iter()
            .map(|s| {
                card(&s.name).meta(format!(
                    "{} · {} obj",
                    crate::tui::humanize::location(&s.region),
                    s.object_count
                ))
            })
            .collect(),
        ResourceTab::Kubernetes => app
            .k8s_clusters
            .iter()
            .map(|c| {
                let (color, label) = status_view(&c.status, palette);
                card(&c.name)
                    .status(color, label)
                    .meta(format!("v{} · {}c · {} MB", c.version, c.cpu, c.ram_mb))
            })
            .collect(),
        ResourceTab::Projects => app
            .projects
            .iter()
            .map(|p| {
                card(&p.name)
                    .meta(t!("resource_list.count_resources", n => p.resource_count()).to_string())
            })
            .collect(),
        ResourceTab::Balancers => app
            .balancers
            .iter()
            .map(|b| {
                let (color, label) = status_view(&b.status, palette);
                card(&b.name).status(color, label).meta(format!(
                    "{} · {}",
                    b.ip,
                    crate::tui::humanize::location(&b.location)
                ))
            })
            .collect(),
        ResourceTab::Registry => app
            .registries
            .iter()
            .map(|r| {
                card(&r.name).meta(
                    t!("resource_list.disk_used", pct => registry_used_percent(r)).to_string()
                )
            })
            .collect(),
        ResourceTab::Domains => app
            .domains
            .iter()
            .map(|d| {
                let (color, label) = status_view(&d.status, palette);
                let prolong = if d.auto_prolong {
                    t!("resource_list.auto_prolong")
                } else {
                    t!("resource_list.manual_prolong")
                };
                card(&d.name).status(color, label).meta(prolong.to_string())
            })
            .collect(),
        ResourceTab::Firewall => app
            .firewalls
            .iter()
            .map(|f| card(&f.name).meta(f.policy.clone()))
            .collect(),
        ResourceTab::FloatingIps => app
            .floating_ips
            .iter()
            .map(|f| {
                let (color, label) = status_view(&f.status, palette);
                card(&f.ip).status(color, label).meta(f.server_name.clone())
            })
            .collect(),
        ResourceTab::Images => app
            .images
            .iter()
            .map(|i| {
                let (color, label) = status_view(&i.status, palette);
                card(&i.name)
                    .status(color, label)
                    .meta(format!("{} MB", i.size_mb))
            })
            .collect(),
        ResourceTab::NetworkDrives => app
            .network_drives
            .iter()
            .map(|n| {
                let (color, label) = status_view(&n.status, palette);
                card(&n.name)
                    .status(color, label)
                    .meta(format!("{} GB", n.size_gb))
            })
            .collect(),
        ResourceTab::Vpc => app
            .vpcs
            .iter()
            .map(|v| {
                card(&v.name).meta(format!(
                    "{} · {}",
                    v.subnet,
                    crate::tui::humanize::location(&v.location)
                ))
            })
            .collect(),
        ResourceTab::DedicatedServers => app
            .dedicated_servers
            .iter()
            .map(|d| {
                let (color, label) = status_view(&d.status, palette);
                card(&d.name)
                    .status(color, label)
                    .meta(format!("{} · {}", d.cpu, d.ram))
            })
            .collect(),
        ResourceTab::Mail => app
            .mails
            .iter()
            .map(|m| card(&m.name).meta(m.owner.clone()))
            .collect(),
        ResourceTab::Apps => app
            .apps
            .iter()
            .map(|a| {
                let (color, label) = status_view(&a.status, palette);
                card(&a.name).status(color, label).meta(format!(
                    "{} · {}",
                    crate::tui::humanize::location(&a.location),
                    a.framework
                ))
            })
            .collect(),
        ResourceTab::AiAgents => app
            .ai_agents
            .iter()
            .map(|a| {
                let (color, label) = status_view(&a.status, palette);
                card(&a.name)
                    .status(color, label)
                    .meta(format!("{}/{} tok", a.tokens_used, a.tokens_total))
            })
            .collect(),
        ResourceTab::KnowledgeBases => app
            .knowledge_bases
            .iter()
            .map(|k| {
                let (color, label) = status_view(&k.status, palette);
                card(&k.name)
                    .status(color, label)
                    .meta(t!("resource_list.count_docs", n => k.document_count).to_string())
            })
            .collect(),
        ResourceTab::SshKeys => app.ssh_keys.iter().map(|k| card(k)).collect(),
        ResourceTab::Finances => app.finances.iter().map(|f| card(f)).collect()
    }
}
