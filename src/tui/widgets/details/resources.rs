// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Per-resource renderers for the details panel.

use ratatui::text::Line;
use rust_i18n::t;

use super::{
    accent, chip, empty, heading, kv, kv_field, name_style, rule, section, status_chip, warn
};
use crate::tui::{app::App, themes::Palette, widgets::resource_list::server_status_view};

pub(super) fn render_server_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.servers.is_empty() {
        return empty(&t!("details.no_servers"), palette);
    }

    let server = &app.servers[app.selected_real_index().min(app.servers.len() - 1)];
    let (_, color, label) = server_status_view(&server.status, &palette);
    vec![
        heading(&server.name, palette),
        rule(palette),
        kv(
            &t!("details.id"),
            format!("#{}", server.id),
            accent(palette),
            palette
        ),
        chip(&t!("details.status"), &label, color, palette),
        kv(
            &t!("details.location"),
            server.location.clone(),
            warn(palette),
            palette
        ),
        Line::from(""),
        section(&t!("details.resources"), palette),
        kv(
            &t!("details.cpu"),
            format!("{} {}", server.cpu, t!("details.cores")),
            accent(palette),
            palette
        ),
        kv(
            &t!("details.ram"),
            format!("{} MB", server.ram_mb),
            accent(palette),
            palette
        ),
        kv(
            &t!("details.disk"),
            format!("{} GB", server.disk_gb),
            accent(palette),
            palette
        ),
    ]
}

pub(super) fn render_database_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.databases.is_empty() {
        return empty(&t!("details.no_databases"), palette);
    }

    let db = &app.databases[app.selected_real_index().min(app.databases.len() - 1)];
    let yes_no = |flag: bool| {
        if flag {
            t!("details.yes").into_owned()
        } else {
            t!("details.no").into_owned()
        }
    };
    let mut lines = vec![
        heading(&db.name, palette),
        rule(palette),
        kv(
            &t!("details.id"),
            format!("#{}", db.id),
            accent(palette),
            palette
        ),
        kv(
            &t!("details.engine"),
            db.engine.clone(),
            accent(palette),
            palette
        ),
        status_chip(&t!("details.status"), &db.status, palette),
        kv(
            &t!("details.disk"),
            format!("{} / {} MB", db.disk_used_mb, db.size_mb),
            name_style(palette),
            palette
        ),
    ];
    let optional = [
        (t!("details.location"), db.location.clone()),
        (t!("details.created"), db.created_at.clone()),
        (t!("details.public_ip"), db.public_ip.clone()),
        (t!("details.local_ip"), db.local_ip.clone()),
        (
            t!("details.port"),
            if db.port > 0 {
                db.port.to_string()
            } else {
                String::new()
            }
        ),
        (
            t!("details.preset"),
            if db.preset_id > 0 {
                format!("#{}", db.preset_id)
            } else {
                String::new()
            }
        ),
        (t!("details.hash_type"), db.hash_type.clone())
    ];
    for (label, value) in optional {
        if !value.is_empty() {
            lines.push(kv(&label, value, name_style(palette), palette));
        }
    }
    lines.push(kv(
        &t!("details.local_only"),
        yes_no(db.local_only),
        name_style(palette),
        palette
    ));
    if !db.config.is_empty() {
        lines.push(Line::from(""));
        lines.push(heading(&t!("details.config"), palette));
        lines.push(rule(palette));
        for (key, value) in &db.config {
            lines.push(kv(key, value.clone(), name_style(palette), palette));
        }
    }
    lines
}

pub(super) fn render_s3_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.s3_storages.is_empty() {
        return empty(&t!("details.no_s3"), palette);
    }

    let storage = &app.s3_storages[app.selected_real_index().min(app.s3_storages.len() - 1)];
    vec![
        heading(&storage.name, palette),
        rule(palette),
        kv(
            &t!("details.id"),
            format!("#{}", storage.id),
            accent(palette),
            palette
        ),
        kv(
            &t!("details.region"),
            storage.region.clone(),
            warn(palette),
            palette
        ),
        kv(
            &t!("details.size"),
            format!("{} MB", storage.size_kb / 1024),
            name_style(palette),
            palette
        ),
        kv(
            &t!("details.objects"),
            storage.object_count.to_string(),
            accent(palette),
            palette
        ),
    ]
}

pub(super) fn render_k8s_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.k8s_clusters.is_empty() {
        return empty(&t!("details.no_k8s"), palette);
    }

    let cluster = &app.k8s_clusters[app.selected_real_index().min(app.k8s_clusters.len() - 1)];
    vec![
        heading(&cluster.name, palette),
        rule(palette),
        kv(
            &t!("details.id"),
            format!("#{}", cluster.id),
            accent(palette),
            palette
        ),
        kv(
            &t!("details.version"),
            format!("v{}", cluster.version),
            accent(palette),
            palette
        ),
        status_chip(&t!("details.status"), &cluster.status, palette),
        Line::from(""),
        section(&t!("details.resources"), palette),
        kv(
            &t!("details.cpu"),
            format!("{} {}", cluster.cpu, t!("details.cores")),
            name_style(palette),
            palette
        ),
        kv(
            &t!("details.ram"),
            format!("{} MB", cluster.ram_mb),
            name_style(palette),
            palette
        ),
        kv(
            &t!("details.disk"),
            format!("{} GB", cluster.disk_gb),
            name_style(palette),
            palette
        ),
    ]
}

pub(super) fn render_project_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.projects.is_empty() {
        return empty(&t!("details.no_projects"), palette);
    }

    let project = &app.projects[app.selected_real_index().min(app.projects.len() - 1)];
    let mut lines = vec![
        heading(&project.name, palette),
        rule(palette),
        kv(
            &t!("details.id"),
            format!("#{}", project.id),
            accent(palette),
            palette
        ),
        Line::from(""),
        section(&t!("details.resources"), palette),
    ];
    let counts = [
        (t!("details.servers"), project.server_count),
        (t!("details.databases"), project.database_count),
        (t!("details.buckets"), project.bucket_count),
        (t!("details.clusters"), project.cluster_count),
        (t!("details.balancers"), project.balancer_count),
        (t!("details.dedicated"), project.dedicated_count),
        (t!("details.apps"), project.app_count)
    ];
    let mut shown = 0;
    for (label, count) in counts {
        if count > 0 {
            lines.push(kv(&label, count.to_string(), name_style(palette), palette));
            shown += 1;
        }
    }
    if shown == 0 {
        lines.push(kv(
            &t!("details.total"),
            "0".to_string(),
            name_style(palette),
            palette
        ));
    }
    lines
}

pub(super) fn render_string_details(
    items: &[String],
    app: &App,
    label: &str,
    palette: Palette
) -> Vec<Line<'static>> {
    if items.is_empty() {
        return empty(&t!("details.no_entries", name => label), palette);
    }
    let item = &items[app.selected_real_index().min(items.len() - 1)];
    vec![
        heading(label, palette),
        rule(palette),
        kv(
            &t!("details.value"),
            item.clone(),
            name_style(palette),
            palette
        ),
    ]
}

pub(super) fn render_balancer_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.balancers.is_empty() {
        return empty(&t!("details.no_balancers"), palette);
    }
    let b = &app.balancers[app.selected_real_index().min(app.balancers.len() - 1)];
    vec![
        heading(&b.name, palette),
        rule(palette),
        kv(
            &t!("details.id"),
            format!("#{}", b.id),
            accent(palette),
            palette
        ),
        status_chip(&t!("details.status"), &b.status, palette),
        kv(&t!("details.ip"), b.ip.clone(), warn(palette), palette),
        kv(
            &t!("details.location"),
            b.location.clone(),
            warn(palette),
            palette
        ),
    ]
}

pub(super) fn render_registry_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.registries.is_empty() {
        return empty(&t!("details.no_registries"), palette);
    }
    let r = &app.registries[app.selected_real_index().min(app.registries.len() - 1)];
    vec![
        heading(&r.name, palette),
        rule(palette),
        kv(
            &t!("details.id"),
            format!("#{}", r.id),
            accent(palette),
            palette
        ),
        kv(
            &t!("details.disk_used"),
            format!("{}%", disk_used_percent(r.disk_used, r.disk_size)),
            name_style(palette),
            palette
        ),
    ]
}

/// Computes an integral used-disk percentage, treating a zero-sized disk as
/// fully free.
pub(super) fn disk_used_percent(used: i64, size: i64) -> i64 {
    if size <= 0 { 0 } else { used * 100 / size }
}

pub(super) fn render_domain_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.domains.is_empty() {
        return empty(&t!("details.no_domains"), palette);
    }
    let d = &app.domains[app.selected_real_index().min(app.domains.len() - 1)];
    vec![
        heading(&d.name, palette),
        rule(palette),
        kv(
            &t!("details.id"),
            format!("#{}", d.id),
            accent(palette),
            palette
        ),
        status_chip(&t!("details.status"), &d.status, palette),
        kv(
            &t!("details.auto_prolong"),
            if d.auto_prolong {
                t!("details.yes")
            } else {
                t!("details.no")
            }
            .to_string(),
            name_style(palette),
            palette
        ),
    ]
}

pub(super) fn render_firewall_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.firewalls.is_empty() {
        return empty(&t!("details.no_firewalls"), palette);
    }
    let f = &app.firewalls[app.selected_real_index().min(app.firewalls.len() - 1)];
    vec![
        heading(&f.name, palette),
        rule(palette),
        kv(
            &t!("details.id"),
            format!("#{}", f.id),
            accent(palette),
            palette
        ),
        kv(
            &t!("details.policy"),
            f.policy.clone(),
            name_style(palette),
            palette
        ),
    ]
}

pub(super) fn render_floating_ip_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.floating_ips.is_empty() {
        return empty(&t!("details.no_floating_ips"), palette);
    }
    let f = &app.floating_ips[app.selected_real_index().min(app.floating_ips.len() - 1)];
    vec![
        heading(&f.ip, palette),
        rule(palette),
        kv(
            &t!("details.id"),
            format!("#{}", f.id),
            accent(palette),
            palette
        ),
        status_chip(&t!("details.status"), &f.status, palette),
        kv(
            &t!("details.bound_to"),
            f.server_name.clone(),
            name_style(palette),
            palette
        ),
    ]
}

pub(super) fn render_image_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.images.is_empty() {
        return empty(&t!("details.no_images"), palette);
    }
    let i = &app.images[app.selected_real_index().min(app.images.len() - 1)];
    vec![
        heading(&i.name, palette),
        rule(palette),
        kv(
            &t!("details.id"),
            format!("#{}", i.id),
            accent(palette),
            palette
        ),
        status_chip(&t!("details.status"), &i.status, palette),
        kv(
            &t!("details.size"),
            format!("{} MB", i.size_mb),
            name_style(palette),
            palette
        ),
    ]
}

pub(super) fn render_network_drive_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.network_drives.is_empty() {
        return empty(&t!("details.no_network_drives"), palette);
    }
    let n = &app.network_drives[app.selected_real_index().min(app.network_drives.len() - 1)];
    vec![
        heading(&n.name, palette),
        rule(palette),
        kv(
            &t!("details.id"),
            format!("#{}", n.id),
            accent(palette),
            palette
        ),
        status_chip(&t!("details.status"), &n.status, palette),
        kv(
            &t!("details.size"),
            format!("{} GB", n.size_gb),
            name_style(palette),
            palette
        ),
    ]
}

pub(super) fn render_vpc_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.vpcs.is_empty() {
        return empty(&t!("details.no_vpcs"), palette);
    }
    let v = &app.vpcs[app.selected_real_index().min(app.vpcs.len() - 1)];
    vec![
        heading(&v.name, palette),
        rule(palette),
        kv(
            &t!("details.id"),
            format!("#{}", v.id),
            accent(palette),
            palette
        ),
        kv(
            &t!("details.subnet"),
            v.subnet.clone(),
            name_style(palette),
            palette
        ),
        kv(
            &t!("details.location"),
            v.location.clone(),
            warn(palette),
            palette
        ),
    ]
}

pub(super) fn render_dedicated_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.dedicated_servers.is_empty() {
        return empty(&t!("details.no_dedicated"), palette);
    }
    let d = &app.dedicated_servers[app
        .selected_real_index()
        .min(app.dedicated_servers.len() - 1)];
    vec![
        heading(&d.name, palette),
        rule(palette),
        kv(
            &t!("details.id"),
            format!("#{}", d.id),
            accent(palette),
            palette
        ),
        status_chip(&t!("details.status"), &d.status, palette),
        kv(&t!("details.ip"), d.ip.clone(), warn(palette), palette),
        Line::from(""),
        section(&t!("details.resources"), palette),
        kv(
            &t!("details.cpu"),
            d.cpu.clone(),
            name_style(palette),
            palette
        ),
        kv(
            &t!("details.ram"),
            d.ram.clone(),
            name_style(palette),
            palette
        ),
        kv(
            &t!("details.disk"),
            d.disk.clone(),
            name_style(palette),
            palette
        ),
    ]
}

pub(super) fn render_mail_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.mails.is_empty() {
        return empty(&t!("details.no_mail"), palette);
    }
    let m = &app.mails[app.selected_real_index().min(app.mails.len() - 1)];
    let mut lines = vec![heading(&m.name, palette), rule(palette)];
    if !m.owner.is_empty() {
        lines.push(kv(
            &t!("details.owner"),
            m.owner.clone(),
            name_style(palette),
            palette
        ));
    }
    if !m.comment.is_empty() {
        lines.push(kv(
            &t!("details.comment"),
            m.comment.clone(),
            accent(palette),
            palette
        ));
    }
    lines
}

pub(super) fn render_app_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.apps.is_empty() {
        return empty(&t!("details.no_apps"), palette);
    }
    let a = &app.apps[app.selected_real_index().min(app.apps.len() - 1)];
    let mut lines = vec![
        heading(&a.name, palette),
        rule(palette),
        kv(
            &t!("details.id"),
            format!("#{}", a.id),
            accent(palette),
            palette
        ),
        status_chip(&t!("details.status"), &a.status, palette),
        kv_field(&t!("details.type"), &a.app_type, accent(palette), palette),
        kv_field(
            &t!("details.framework"),
            &a.framework,
            name_style(palette),
            palette
        ),
        kv_field(
            &t!("details.language"),
            &a.language,
            name_style(palette),
            palette
        ),
        kv_field(&t!("details.ip"), &a.ip, warn(palette), palette),
        kv_field(&t!("details.location"), &a.location, warn(palette), palette),
        Line::from(""),
        section(&t!("details.repository"), palette),
        kv_field(&t!("details.branch"), &a.branch, accent(palette), palette),
        kv_field(
            &t!("details.commit"),
            &a.commit,
            name_style(palette),
            palette
        ),
        kv(
            &t!("details.auto_deploy"),
            if a.auto_deploy {
                t!("details.yes")
            } else {
                t!("details.no")
            }
            .to_string(),
            name_style(palette),
            palette
        ),
    ];

    if !a.comment.is_empty() {
        lines.push(kv_field(
            &t!("details.comment"),
            &a.comment,
            warn(palette),
            palette
        ));
    }

    if !a.domains.is_empty() {
        lines.push(Line::from(""));
        lines.push(section(&t!("details.domains"), palette));
        for domain in &a.domains {
            lines.push(kv("", domain.clone(), accent(palette), palette));
        }
    }

    lines
}

pub(super) fn render_ai_agent_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.ai_agents.is_empty() {
        return empty(&t!("details.no_ai_agents"), palette);
    }
    let a = &app.ai_agents[app.selected_real_index().min(app.ai_agents.len() - 1)];
    vec![
        heading(&a.name, palette),
        rule(palette),
        kv(
            &t!("details.id"),
            format!("#{}", a.id),
            accent(palette),
            palette
        ),
        status_chip(&t!("details.status"), &a.status, palette),
        kv(
            &t!("details.tokens"),
            format!("{} / {}", a.tokens_used, a.tokens_total),
            accent(palette),
            palette
        ),
    ]
}

pub(super) fn render_knowledge_details(app: &App, palette: Palette) -> Vec<Line<'static>> {
    if app.knowledge_bases.is_empty() {
        return empty(&t!("details.no_knowledge"), palette);
    }
    let k = &app.knowledge_bases[app.selected_real_index().min(app.knowledge_bases.len() - 1)];
    vec![
        heading(&k.name, palette),
        rule(palette),
        kv(
            &t!("details.id"),
            format!("#{}", k.id),
            accent(palette),
            palette
        ),
        status_chip(&t!("details.status"), &k.status, palette),
        kv(
            &t!("details.documents"),
            k.document_count.to_string(),
            name_style(palette),
            palette
        ),
    ]
}
