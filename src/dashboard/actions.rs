// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Dashboard resource actions: dispatch, creation and drill-in views.

use crate::tui;

pub async fn fetch_drill(
    config: &timeweb_rs::apis::configuration::Configuration,
    tab: tui::app::ResourceTab,
    id: i32,
    name: &str
) -> Result<tui::app::DrillView, String> {
    use tui::app::{DrillItem, DrillView, ResourceTab};

    match tab {
        ResourceTab::Projects => {
            let resp = timeweb_rs::apis::projects_api::get_all_project_resources(config, id)
                .await
                .map_err(|e| e.to_string())?;
            let mut items = Vec::new();
            for s in &resp.servers {
                items.push(DrillItem {
                    tab:    tui::app::ResourceTab::Servers,
                    id:     s.id.to_string(),
                    kind:   "Server".to_string(),
                    name:   s.name.clone(),
                    detail: format!("{:?}", s.status)
                });
            }
            for d in &resp.databases {
                items.push(DrillItem {
                    tab:    tui::app::ResourceTab::Databases,
                    id:     d.id.to_string(),
                    kind:   "Database".to_string(),
                    name:   d.name.clone(),
                    detail: d.r#type.clone()
                });
            }
            for b in &resp.buckets {
                items.push(DrillItem {
                    tab:    tui::app::ResourceTab::S3,
                    id:     b.id.to_string(),
                    kind:   "S3 bucket".to_string(),
                    name:   b.name.clone(),
                    detail: String::new()
                });
            }
            for c in &resp.clusters {
                items.push(DrillItem {
                    tab:    tui::app::ResourceTab::Kubernetes,
                    id:     c.id.to_string(),
                    kind:   "Kubernetes".to_string(),
                    name:   c.name.clone(),
                    detail: format!("{:?}", c.status)
                });
            }
            for b in &resp.balancers {
                items.push(DrillItem {
                    tab:    tui::app::ResourceTab::Balancers,
                    id:     b.id.to_string(),
                    kind:   "Balancer".to_string(),
                    name:   b.name.clone(),
                    detail: format!("{:?}", b.status)
                });
            }
            for d in &resp.dedicated_servers {
                items.push(DrillItem {
                    tab:    tui::app::ResourceTab::DedicatedServers,
                    id:     d.id.to_string(),
                    kind:   "Dedicated".to_string(),
                    name:   d.name.clone(),
                    detail: String::new()
                });
            }
            for a in resp.apps.iter().flatten().flatten() {
                items.push(DrillItem {
                    tab:    tui::app::ResourceTab::Apps,
                    id:     a.id.to_string(),
                    kind:   "App".to_string(),
                    name:   a.name.clone(),
                    detail: format!("{:?}", a.status)
                });
            }
            Ok(DrillView {
                title: name.to_string(),
                items,
                selected: 0
            })
        }
        _ => Err("this resource cannot be entered".to_string())
    }
}

/// Performs an in-dashboard resource creation submitted from a create form,
/// logging the outcome. Only resources with a simple create form are handled.
pub async fn perform_create(
    config: &timeweb_rs::apis::configuration::Configuration,
    app: &mut tui::app::App,
    form: tui::app::CreateForm
) {
    use timeweb_rs::{apis::projects_api, models::CreateProject};
    use tui::app::{LogLevel, ResourceTab};

    let field = |i: usize| form.fields.get(i).map(|f| f.value.trim().to_owned());

    let result = match form.tab {
        ResourceTab::Projects => {
            let name = field(0).unwrap_or_default();
            let mut req = CreateProject::new(name);
            if let Some(desc) = field(1).filter(|d| !d.is_empty()) {
                req.description = Some(Some(desc));
            }
            projects_api::create_project(config, req)
                .await
                .map(|r| r.project.name)
                .map_err(|e| e.to_string())
        }
        _ => Err("creation not supported for this resource".to_string())
    };

    match result {
        Ok(name) => {
            let msg = format!("created '{name}'");
            app.log(LogLevel::Success, msg.clone());
            app.status_message = Some(msg);
            app.error_message = None;
        }
        Err(e) => {
            let msg = format!("create failed: {e}");
            app.log(LogLevel::Error, msg.clone());
            app.error_message = Some(msg);
        }
    }
}

/// Maps a `(tab, action)` pair to the matching Timeweb API call.
///
/// Resources with numeric ids parse [`tui::app::PendingAction::resource_id`]
/// back to `i32`; resources addressed by UUID or FQDN pass it (or the
/// resource name, for domains) through as-is.
// JUSTIFY: One arm per resource/key path; splitting would only scatter the flow.
#[allow(clippy::too_many_lines)]
async fn dispatch_action(
    config: &timeweb_rs::apis::configuration::Configuration,
    pending: &tui::app::PendingAction
) -> Result<(), String> {
    use timeweb_rs::apis::{
        ai_agents_api, apps_api, balancers_api, container_registry_api, databases_api,
        dedicated_servers_api, domains_api, firewall_api, floating_ip_api, images_api,
        knowledge_bases_api, kubernetes_api, network_drives_api, projects_api, s3_api,
        servers_api, vpc_api
    };
    use tui::app::{ActionKind, ResourceTab};

    let id = pending.resource_id.as_str();
    let num = || {
        id.parse::<i32>()
            .map_err(|_| format!("invalid numeric id '{id}'"))
    };
    match (pending.tab, pending.kind) {
        (ResourceTab::Servers, ActionKind::Start) => servers_api::start_server(config, num()?)
            .await
            .map_err(|e| e.to_string()),
        (ResourceTab::Servers, ActionKind::Shutdown) => {
            servers_api::shutdown_server(config, num()?)
                .await
                .map_err(|e| e.to_string())
        }
        (ResourceTab::Servers, ActionKind::Reboot) => servers_api::reboot_server(config, num()?)
            .await
            .map_err(|e| e.to_string()),
        (ResourceTab::Servers, ActionKind::Clone) => servers_api::clone_server(config, num()?)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string()),
        (ResourceTab::Servers, ActionKind::Delete) => {
            servers_api::delete_server(config, num()?, None, None)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
        (ResourceTab::Databases, ActionKind::Backup) => {
            databases_api::create_database_backup(config, num()?, None)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
        (ResourceTab::Databases, ActionKind::Delete) => {
            databases_api::delete_database_cluster(config, num()?, None, None)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
        (ResourceTab::S3, ActionKind::Delete) => {
            s3_api::delete_storage(config, num()?, None, None)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
        (ResourceTab::Kubernetes, ActionKind::Delete) => {
            kubernetes_api::delete_cluster(config, num()?, None, None)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
        (ResourceTab::Balancers, ActionKind::Delete) => {
            balancers_api::delete_balancer(config, num()?, None, None)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
        (ResourceTab::Registry, ActionKind::Delete) => {
            container_registry_api::delete_registry(config, num()?)
                .await
                .map_err(|e| e.to_string())
        }
        (ResourceTab::Projects, ActionKind::Delete) => {
            projects_api::delete_project(config, num()?)
                .await
                .map_err(|e| e.to_string())
        }
        (ResourceTab::DedicatedServers, ActionKind::Delete) => {
            dedicated_servers_api::delete_dedicated_server(config, num()?)
                .await
                .map_err(|e| e.to_string())
        }
        (ResourceTab::AiAgents, ActionKind::Delete) => ai_agents_api::delete_agent(config, num()?)
            .await
            .map_err(|e| e.to_string()),
        (ResourceTab::KnowledgeBases, ActionKind::Delete) => {
            knowledge_bases_api::delete_knowledgebase(config, num()?)
                .await
                .map_err(|e| e.to_string())
        }
        (ResourceTab::Apps, ActionKind::Delete) => apps_api::delete_app(config, id)
            .await
            .map_err(|e| e.to_string()),
        (ResourceTab::Domains, ActionKind::Delete) => {
            domains_api::delete_domain(config, &pending.resource_name)
                .await
                .map_err(|e| e.to_string())
        }
        (ResourceTab::Firewall, ActionKind::Delete) => firewall_api::delete_group(config, id)
            .await
            .map_err(|e| e.to_string()),
        (ResourceTab::FloatingIps, ActionKind::Delete) => {
            floating_ip_api::delete_floating_ip(config, id)
                .await
                .map_err(|e| e.to_string())
        }
        (ResourceTab::Images, ActionKind::Delete) => images_api::delete_image(config, id)
            .await
            .map_err(|e| e.to_string()),
        (ResourceTab::NetworkDrives, ActionKind::Delete) => {
            network_drives_api::delete_network_drive(config, id)
                .await
                .map_err(|e| e.to_string())
        }
        (ResourceTab::Vpc, ActionKind::Delete) => vpc_api::delete_vpc(config, id)
            .await
            .map_err(|e| e.to_string()),
        _ => Err("action not supported for this resource".to_string())
    }
}

pub async fn perform_action(
    config: &timeweb_rs::apis::configuration::Configuration,
    app: &mut tui::app::App,
    pending: tui::app::PendingAction
) {
    let result = dispatch_action(config, &pending).await;

    match result {
        Ok(()) => {
            app.error_message = None;
            let msg = format!(
                "{} '{}' (id {}) — ok",
                pending.kind.label(),
                pending.resource_name,
                pending.resource_id
            );
            app.log(tui::app::LogLevel::Success, msg.clone());
            app.status_message = Some(msg);
        }
        Err(e) => {
            let msg = format!(
                "{} '{}' failed: {e}",
                pending.kind.label(),
                pending.resource_name
            );
            app.log(tui::app::LogLevel::Error, msg.clone());
            app.error_message = Some(msg);
        }
    }
}

/// Values a deep-detail fetch needs beyond the resource id itself, resolved
/// from the loaded summaries before the background task is spawned (the task
/// cannot borrow the `App`).
#[derive(Debug, Clone, Default)]
pub struct DetailContext {
    /// Tariff preset of the selected database cluster.
    pub database_preset_id: i32,
    /// Tariff preset of the selected application.
    pub app_preset_id:      i64,
    /// Fully qualified name of the selected domain.
    pub fqdn:               String
}

/// Resolves the per-tab [`DetailContext`] for the resource `id` from the
/// summaries already loaded into the app.
#[must_use]
pub fn detail_context(app: &tui::app::App, tab: tui::app::ResourceTab, id: &str) -> DetailContext {
    use tui::app::ResourceTab;

    let mut context = DetailContext::default();
    match tab {
        ResourceTab::Databases => {
            context.database_preset_id = app
                .databases
                .iter()
                .find(|d| d.id.to_string() == id)
                .map_or(0, |d| d.preset_id);
        }
        ResourceTab::Apps => {
            context.app_preset_id = app
                .apps
                .iter()
                .find(|a| a.id.to_string() == id)
                .map_or(0, |a| a.preset_id);
        }
        ResourceTab::Domains => {
            context.fqdn = app
                .domains
                .iter()
                .find(|d| d.id.to_string() == id)
                .map_or_else(String::new, |d| d.name.clone());
        }
        _ => {}
    }
    context
}

/// Fetches the background deep-detail sections for the resource `id` on
/// `tab`. Tabs without deep-detail endpoints yield no sections, and failed
/// calls are silently skipped so the details panel simply shows fewer
/// sections.
pub async fn fetch_detail_extra(
    config: &timeweb_rs::apis::configuration::Configuration,
    tab: tui::app::ResourceTab,
    id: &str,
    context: DetailContext
) -> tui::app::DetailSections {
    use tui::app::ResourceTab;

    match (tab, id.parse::<i32>()) {
        (ResourceTab::Databases, Ok(num)) => {
            fetch_database_extra(config, num, context.database_preset_id).await
        }
        (ResourceTab::Apps, Ok(num)) => fetch_app_extra(config, num, context.app_preset_id).await,
        (ResourceTab::Servers, Ok(num)) => fetch_server_extra(config, num).await,
        (ResourceTab::Kubernetes, Ok(num)) => fetch_kubernetes_extra(config, num).await,
        (ResourceTab::Balancers, Ok(num)) => fetch_balancer_extra(config, num).await,
        (ResourceTab::Registry, Ok(num)) => fetch_registry_extra(config, num).await,
        (ResourceTab::S3, Ok(num)) => fetch_s3_extra(config, num).await,
        (ResourceTab::Domains, _) if !context.fqdn.is_empty() => {
            fetch_domain_extra(config, &context.fqdn).await
        }
        (ResourceTab::Firewall, _) => fetch_firewall_extra(config, id).await,
        (ResourceTab::Vpc, _) => fetch_vpc_extra(config, id).await,
        _ => Vec::new()
    }
}

/// Row cap for unbounded deep-detail lists.
const MAX_SECTION_ROWS: usize = 10;

/// Caps `rows` at [`MAX_SECTION_ROWS`], appending an ellipsis row naming the
/// hidden remainder, and pushes the section when any rows survive.
fn push_section(
    sections: &mut tui::app::DetailSections,
    title: String,
    rows: &mut Vec<(String, String)>
) {
    use rust_i18n::t;

    if rows.is_empty() {
        return;
    }
    if rows.len() > MAX_SECTION_ROWS {
        let hidden = rows.len() - MAX_SECTION_ROWS;
        rows.truncate(MAX_SECTION_ROWS);
        rows.push((
            String::new(),
            t!("details.more", count => hidden).into_owned()
        ));
    }
    sections.push((title, std::mem::take(rows)));
}

/// Fetches the deep details of a database cluster that the list endpoint does
/// not carry: connection admins (host and login), the databases inside the
/// cluster, and the decoded tariff preset. Passwords are never included.
#[expect(clippy::cast_possible_truncation)]
async fn fetch_database_extra(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32,
    preset_id: i32
) -> tui::app::DetailSections {
    use rust_i18n::t;
    use timeweb_rs::apis::databases_api;

    let (users, instances, presets) = tokio::join!(
        databases_api::get_database_users(config, id),
        databases_api::get_database_instances(config, id),
        databases_api::get_databases_presets(config, None)
    );

    let mut sections = Vec::new();

    if let Ok(resp) = users {
        let mut rows: Vec<(String, String)> = Vec::new();
        for admin in &resp.admins {
            if let Some(host) = admin.host.as_deref().filter(|h| !h.is_empty() && *h != "%") {
                rows.push((t!("details.host").into_owned(), host.to_string()));
            }
            rows.push((t!("details.login").into_owned(), admin.login.clone()));
        }
        if !rows.is_empty() {
            rows.dedup();
            sections.push((t!("details.connection").into_owned(), rows));
        }
    }

    if let Ok(resp) = instances {
        let rows: Vec<(String, String)> = resp
            .instances
            .iter()
            .map(|db| (db.name.clone(), db.description.clone()))
            .collect();
        if !rows.is_empty() {
            sections.push((t!("details.databases_in").into_owned(), rows));
        }
    }

    if let Ok(resp) = presets
        && let Some(preset) = resp
            .databases_presets
            .iter()
            .find(|p| p.id == Some(i64::from(preset_id)))
    {
        let mut rows: Vec<(String, String)> = Vec::new();
        if let Some(cpu) = preset.cpu {
            rows.push((t!("details.cpu").into_owned(), format!("{cpu}")));
        }
        if let Some(ram) = preset.ram {
            rows.push((
                t!("details.ram").into_owned(),
                crate::tui::humanize::megabytes(ram as i64)
            ));
        }
        if let Some(disk) = preset.disk {
            rows.push((
                t!("details.disk").into_owned(),
                crate::tui::humanize::megabytes(disk as i64)
            ));
        }
        if let Some(price) = preset.price {
            rows.push((
                t!("details.price").into_owned(),
                t!("details.per_month", price => price).into_owned()
            ));
        }
        if let Some(desc) = preset
            .description_short
            .as_deref()
            .filter(|d| !d.is_empty())
        {
            rows.push((t!("details.plan").into_owned(), desc.to_string()));
        }
        if !rows.is_empty() {
            sections.push((t!("details.tariff").into_owned(), rows));
        }
    }

    sections
}

/// Fetches the deep details of an application: its recent deploys and the
/// decoded tariff preset.
async fn fetch_app_extra(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32,
    preset_id: i64
) -> tui::app::DetailSections {
    use rust_i18n::t;
    use timeweb_rs::apis::apps_api;

    let id_text = id.to_string();
    let (deploys, presets) = tokio::join!(
        apps_api::get_app_deploys(config, &id_text, None, None),
        apps_api::get_apps_presets(config, &id_text)
    );

    let mut sections = Vec::new();

    if let Ok(resp) = deploys {
        let rows: Vec<(String, String)> = resp
            .deploys
            .unwrap_or_default()
            .iter()
            .take(5)
            .map(|d| {
                let sha: String = d.commit_sha.chars().take(7).collect();
                (
                    crate::tui::humanize::date(&d.started_at),
                    format!("{:?} · {sha} · {}", d.status, d.commit_msg)
                )
            })
            .collect();
        if !rows.is_empty() {
            sections.push((t!("details.deploys").into_owned(), rows));
        }
    }

    if let Ok(resp) = presets {
        let mut rows: Vec<(String, String)> = Vec::new();
        if let Some(p) = resp
            .backend_presets
            .iter()
            .flatten()
            .find(|p| i64::from(p.id) == preset_id)
        {
            rows.push((t!("details.cpu").into_owned(), p.cpu.to_string()));
            rows.push((
                t!("details.ram").into_owned(),
                crate::tui::humanize::megabytes(i64::from(p.ram))
            ));
            rows.push((
                t!("details.disk").into_owned(),
                crate::tui::humanize::megabytes(i64::from(p.disk))
            ));
            rows.push((
                t!("details.price").into_owned(),
                t!("details.per_month", price => p.price).into_owned()
            ));
            if !p.description_short.is_empty() {
                rows.push((t!("details.plan").into_owned(), p.description_short.clone()));
            }
        } else if let Some(p) = resp
            .frontend_presets
            .iter()
            .flatten()
            .find(|p| i64::from(p.id) == preset_id)
        {
            rows.push((
                t!("details.disk").into_owned(),
                crate::tui::humanize::megabytes(i64::from(p.disk))
            ));
            if let Some(requests) = p.requests {
                rows.push((t!("details.requests").into_owned(), requests.to_string()));
            }
            rows.push((
                t!("details.price").into_owned(),
                t!("details.per_month", price => p.price).into_owned()
            ));
            if !p.description_short.is_empty() {
                rows.push((t!("details.plan").into_owned(), p.description_short.clone()));
            }
        }
        if !rows.is_empty() {
            sections.push((t!("details.tariff").into_owned(), rows));
        }
    }

    sections
}

/// Fetches the deep details of a cloud server: its attached IP addresses
/// (with PTR records) and its disks with usage and status.
#[expect(clippy::cast_possible_truncation)]
async fn fetch_server_extra(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32
) -> tui::app::DetailSections {
    use rust_i18n::t;
    use timeweb_rs::apis::servers_api;

    let (ips, disks) = tokio::join!(
        servers_api::get_server_ips(config, id),
        servers_api::get_server_disks(config, id)
    );

    let mut sections = Vec::new();

    if let Ok(resp) = ips {
        let mut rows: Vec<(String, String)> = resp
            .server_ips
            .iter()
            .map(|ip| {
                let value = if ip.ptr.is_empty() {
                    ip.ip.clone()
                } else {
                    format!("{} \u{b7} {}", ip.ip, ip.ptr)
                };
                (ip.r#type.clone(), value)
            })
            .collect();
        push_section(&mut sections, t!("details.ips").into_owned(), &mut rows);
    }

    if let Ok(resp) = disks {
        let mut rows: Vec<(String, String)> = resp
            .server_disks
            .iter()
            .map(|disk| {
                (
                    disk.system_name.clone(),
                    format!(
                        "{} / {} \u{b7} {}",
                        crate::tui::humanize::megabytes(disk.used as i64),
                        crate::tui::humanize::megabytes(disk.size as i64),
                        disk.status
                    )
                )
            })
            .collect();
        push_section(&mut sections, t!("details.disks").into_owned(), &mut rows);
    }

    sections
}

/// Formats a Kubernetes cluster resource as `used / capacity`, falling back
/// to whichever side the API returned.
fn usage_value(resource: &timeweb_rs::models::Resource) -> Option<String> {
    match (resource.used, resource.capacity) {
        (Some(used), Some(capacity)) => Some(format!("{used} / {capacity}")),
        (Some(used), None) => Some(used.to_string()),
        (None, Some(capacity)) => Some(capacity.to_string()),
        (None, None) => None
    }
}

/// Fetches the deep details of a Kubernetes cluster: its node groups and the
/// aggregate node/CPU/RAM/pod usage.
///
/// The SDK marks `get_cluster_resources` deprecated (it answers only for
/// older clusters) but offers no replacement; newer clusters simply skip the
/// resources section when the call fails.
async fn fetch_kubernetes_extra(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32
) -> tui::app::DetailSections {
    use rust_i18n::t;
    use timeweb_rs::apis::kubernetes_api;

    #[expect(deprecated)]
    let (groups, resources) = tokio::join!(
        kubernetes_api::get_cluster_node_groups(config, id),
        kubernetes_api::get_cluster_resources(config, id)
    );

    let mut sections = Vec::new();

    if let Ok(resp) = groups {
        let mut rows: Vec<(String, String)> = resp
            .node_groups
            .iter()
            .map(|group| {
                (
                    group.name.clone(),
                    t!(
                        "details.nodes_preset",
                        count => group.node_count,
                        preset => group.preset_id
                    )
                    .into_owned()
                )
            })
            .collect();
        push_section(
            &mut sections,
            t!("details.node_groups").into_owned(),
            &mut rows
        );
    }

    if let Ok(resp) = resources {
        let mut rows: Vec<(String, String)> = Vec::new();
        if let Some(nodes) = resp.resources.nodes {
            rows.push((t!("details.nodes").into_owned(), nodes.to_string()));
        }
        if let Some(cores) = resp.resources.cores.as_deref().and_then(usage_value) {
            rows.push((t!("details.cpu").into_owned(), cores));
        }
        if let Some(memory) = resp.resources.memory.as_deref().and_then(usage_value) {
            rows.push((t!("details.ram").into_owned(), memory));
        }
        if let Some(pods) = resp.resources.pods.as_deref().and_then(usage_value) {
            rows.push((t!("details.pods").into_owned(), pods));
        }
        push_section(
            &mut sections,
            t!("details.resources").into_owned(),
            &mut rows
        );
    }

    sections
}

/// Fetches the deep details of a load balancer: its forwarding rules and the
/// IP addresses it answers on.
async fn fetch_balancer_extra(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32
) -> tui::app::DetailSections {
    use rust_i18n::t;
    use timeweb_rs::apis::balancers_api;

    let (rules, ips) = tokio::join!(
        balancers_api::get_balancer_rules(config, id),
        balancers_api::get_balancer_ips(config, id)
    );

    let mut sections = Vec::new();

    if let Ok(resp) = rules {
        let mut rows: Vec<(String, String)> = resp
            .rules
            .iter()
            .map(|rule| {
                (
                    format!("{:?} {}", rule.balancer_proto, rule.balancer_port).to_lowercase(),
                    format!(
                        "\u{2192} {} {}",
                        format!("{:?}", rule.server_proto).to_lowercase(),
                        rule.server_port
                    )
                )
            })
            .collect();
        push_section(&mut sections, t!("details.rules").into_owned(), &mut rows);
    }

    if let Ok(resp) = ips {
        let mut rows: Vec<(String, String)> = resp
            .ips
            .iter()
            .map(|ip| (t!("details.ip").into_owned(), ip.clone()))
            .collect();
        push_section(&mut sections, t!("details.ips").into_owned(), &mut rows);
    }

    sections
}

/// Fetches the deep details of a container registry: its repositories with
/// their stored size.
async fn fetch_registry_extra(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32
) -> tui::app::DetailSections {
    use rust_i18n::t;
    use timeweb_rs::apis::container_registry_api;

    let mut sections = Vec::new();

    if let Ok(resp) = container_registry_api::get_registry_repositories(config, id).await {
        let mut rows: Vec<(String, String)> = resp
            .container_registries_repositories
            .iter()
            .map(|repo| {
                (
                    repo.name.clone(),
                    crate::tui::humanize::megabytes(i64::from(repo.tags.size))
                )
            })
            .collect();
        push_section(
            &mut sections,
            t!("details.repositories").into_owned(),
            &mut rows
        );
    }

    sections
}

/// Fetches the deep details of a domain by its fqdn: the DNS records and the
/// delegated nameservers.
async fn fetch_domain_extra(
    config: &timeweb_rs::apis::configuration::Configuration,
    fqdn: &str
) -> tui::app::DetailSections {
    use rust_i18n::t;
    use timeweb_rs::apis::domains_api;

    let (records, nameservers) = tokio::join!(
        domains_api::get_domain_dns_records(config, fqdn, None, None),
        domains_api::get_domain_name_servers(config, fqdn)
    );

    let mut sections = Vec::new();

    if let Ok(resp) = records {
        let mut rows: Vec<(String, String)> = resp
            .dns_records
            .iter()
            .map(|record| {
                let mut key = format!("{:?}", record.r#type).to_uppercase();
                if let Some(sub) = record
                    .data
                    .subdomain
                    .clone()
                    .flatten()
                    .filter(|s| !s.is_empty())
                {
                    key = format!("{key} {sub}");
                }
                (key, record.data.value.clone())
            })
            .collect();
        push_section(
            &mut sections,
            t!("details.dns_records").into_owned(),
            &mut rows
        );
    }

    if let Ok(resp) = nameservers {
        let mut rows: Vec<(String, String)> = resp
            .name_servers
            .iter()
            .flat_map(|ns| ns.items.iter())
            .map(|item| {
                let ips = if item.ips.is_empty() {
                    "\u{2014}".to_string()
                } else {
                    item.ips.join(", ")
                };
                (item.host.clone(), ips)
            })
            .collect();
        push_section(
            &mut sections,
            t!("details.nameservers").into_owned(),
            &mut rows
        );
    }

    sections
}

/// Fetches the deep details of a firewall group: its rules and the resources
/// the group is attached to. The SDK models the rule CIDR as an empty struct,
/// so rules show direction, protocol, port and description instead.
async fn fetch_firewall_extra(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: &str
) -> tui::app::DetailSections {
    use rust_i18n::t;
    use timeweb_rs::apis::firewall_api;

    let (rules, resources) = tokio::join!(
        firewall_api::get_group_rules(config, id, None, None),
        firewall_api::get_group_resources(config, id, None, None)
    );

    let mut sections = Vec::new();

    if let Ok(resp) = rules {
        let mut rows: Vec<(String, String)> = resp
            .rules
            .iter()
            .map(|rule| {
                let port = rule
                    .port
                    .as_deref()
                    .filter(|p| !p.is_empty())
                    .unwrap_or("*");
                let value = if rule.description.is_empty() {
                    port.to_string()
                } else {
                    format!("{port} \u{b7} {}", rule.description)
                };
                (format!("{} {}", rule.direction, rule.protocol), value)
            })
            .collect();
        push_section(&mut sections, t!("details.rules").into_owned(), &mut rows);
    }

    if let Ok(resp) = resources {
        let mut rows: Vec<(String, String)> = resp
            .resources
            .iter()
            .map(|resource| (resource.r#type.to_string(), resource.id.to_string()))
            .collect();
        push_section(
            &mut sections,
            t!("details.attached").into_owned(),
            &mut rows
        );
    }

    sections
}

/// Human label for a VPC port NAT mode.
const fn nat_label(mode: timeweb_rs::models::vpc_port::NatMode) -> &'static str {
    use timeweb_rs::models::vpc_port::NatMode;

    match mode {
        NatMode::DnatAndSnat => "dnat+snat",
        NatMode::Snat => "snat",
        NatMode::NoNat => "no nat"
    }
}

/// Fetches the deep details of a VPC: its ports with their IP, NAT mode and
/// the service behind each port.
async fn fetch_vpc_extra(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: &str
) -> tui::app::DetailSections {
    use rust_i18n::t;
    use timeweb_rs::apis::vpc_api;

    let mut sections = Vec::new();

    if let Ok(resp) = vpc_api::get_vpc_ports(config, id).await {
        let mut rows: Vec<(String, String)> = resp
            .vpc_ports
            .iter()
            .map(|port| {
                (
                    port.ipv4.clone(),
                    format!("{} \u{b7} {}", nat_label(port.nat_mode), port.service.name)
                )
            })
            .collect();
        push_section(&mut sections, t!("details.ports").into_owned(), &mut rows);
    }

    sections
}

/// Fetches the deep details of an S3 storage: its bound subdomains with their
/// SSL status.
async fn fetch_s3_extra(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32
) -> tui::app::DetailSections {
    use rust_i18n::t;
    use timeweb_rs::apis::s3_api;

    let mut sections = Vec::new();

    if let Ok(resp) = s3_api::get_storage_subdomains(config, id).await {
        let mut rows: Vec<(String, String)> = resp
            .subdomains
            .iter()
            .map(|sub| (sub.subdomain.clone(), format!("{:?}", sub.status)))
            .collect();
        push_section(
            &mut sections,
            t!("details.subdomains").into_owned(),
            &mut rows
        );
    }

    sections
}
