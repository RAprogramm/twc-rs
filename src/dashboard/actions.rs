// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Dashboard resource actions: dispatch, creation and drill-in views.

use crate::tui;

pub(crate) async fn fetch_drill(
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
                    kind:   "Server".to_string(),
                    name:   s.name.clone(),
                    detail: format!("{:?}", s.status)
                });
            }
            for d in &resp.databases {
                items.push(DrillItem {
                    kind:   "Database".to_string(),
                    name:   d.name.clone(),
                    detail: d.r#type.clone()
                });
            }
            for b in &resp.buckets {
                items.push(DrillItem {
                    kind:   "S3 bucket".to_string(),
                    name:   b.name.clone(),
                    detail: String::new()
                });
            }
            for c in &resp.clusters {
                items.push(DrillItem {
                    kind:   "Kubernetes".to_string(),
                    name:   c.name.clone(),
                    detail: format!("{:?}", c.status)
                });
            }
            for b in &resp.balancers {
                items.push(DrillItem {
                    kind:   "Balancer".to_string(),
                    name:   b.name.clone(),
                    detail: format!("{:?}", b.status)
                });
            }
            for d in &resp.dedicated_servers {
                items.push(DrillItem {
                    kind:   "Dedicated".to_string(),
                    name:   d.name.clone(),
                    detail: String::new()
                });
            }
            for a in resp.apps.iter().flatten().flatten() {
                items.push(DrillItem {
                    kind:   "App".to_string(),
                    name:   a.name.clone(),
                    detail: format!("{:?}", a.status)
                });
            }
            Ok(DrillView {
                title: format!("Project '{name}'  ({} resources)", items.len()),
                items,
                selected: 0
            })
        }
        _ => Err("this resource cannot be entered".to_string())
    }
}

/// Performs an in-dashboard resource creation submitted from a create form,
/// logging the outcome. Only resources with a simple create form are handled.
pub(crate) async fn perform_create(
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

pub(crate) async fn perform_action(
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
