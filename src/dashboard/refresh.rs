// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Dashboard data loading: paginated fetches and summary mapping.

use timeweb_rs::authenticated;

use crate::tui;

#[expect(clippy::large_futures)]
pub(crate) async fn fetch_data(
    token: String,
    interval: u64,
    theme: crate::tui::themes::Theme
) -> tui::app::DashboardData {
    let config = authenticated(token);
    let mut tmp = tui::app::App::new_with_theme(interval, theme, None);
    refresh_all(&config, &mut tmp).await;
    tui::app::DashboardData::from_app(&tmp)
}

pub(crate) fn spawn_refresh_loop(
    tx: tokio::sync::mpsc::UnboundedSender<tui::event::AppEvent>,
    token: String,
    theme: crate::tui::themes::Theme,
    interval: u64
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let period = tokio::time::Duration::from_secs(interval.max(2));
        loop {
            let data = Box::pin(fetch_data(token.clone(), interval, theme)).await;
            if tx.send(tui::event::AppEvent::Data(Box::new(data))).is_err() {
                break;
            }
            tokio::time::sleep(period).await;
        }
    })
}

pub(crate) fn spawn_one_shot_refresh(
    tx: tokio::sync::mpsc::UnboundedSender<tui::event::AppEvent>,
    token: String,
    theme: crate::tui::themes::Theme,
    interval: u64
) {
    tokio::spawn(async move {
        let data = Box::pin(fetch_data(token, interval, theme)).await;
        let _ = tx.send(tui::event::AppEvent::Data(Box::new(data)));
    });
}

/// Fetches every page of a paginated list endpoint, advancing the offset by
/// the number of collected items until `meta.total` is reached or a page
/// comes back empty.
#[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
async fn fetch_all_pages<'a, T, E, F>(mut fetch_page: F) -> Result<Vec<T>, E>
where
    F: FnMut(
        i32,
        i32
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(Vec<T>, i32), E>> + Send + 'a>
    >
{
    const PAGE_LIMIT: i32 = 100;

    let mut items: Vec<T> = Vec::new();
    loop {
        let (page, total) = fetch_page(PAGE_LIMIT, items.len() as i32).await?;
        if page.is_empty() {
            return Ok(items);
        }
        items.extend(page);
        if items.len() as i32 >= total {
            return Ok(items);
        }
    }
}

/// Extracts the primary public IPv4 address of a server, preferring the
/// address marked as main and falling back to the first public one.
fn server_public_ip(server: &timeweb_rs::models::Vds) -> String {
    use timeweb_rs::models::vds_networks_inner::Type;

    let mut fallback = None;
    for network in &server.networks {
        if !matches!(network.r#type, Type::Public) {
            continue;
        }
        for ip in network.ips.iter().flatten() {
            if ip.is_main {
                return ip.ip.clone();
            }
            if fallback.is_none() {
                fallback = Some(ip.ip.clone());
            }
        }
    }
    fallback.unwrap_or_default()
}

/// Sums the sizes of all disks attached to a server, converting the API's
/// megabyte values to whole gigabytes.
#[expect(clippy::cast_possible_truncation)]
fn server_disk_gb(server: &timeweb_rs::models::Vds) -> i32 {
    let total_mb: f64 = server.disks.iter().map(|d| d.size).sum();
    (total_mb / 1024.0).round() as i32
}

/// Describes what a floating IP is bound to, resolving server names from the
/// already-fetched server list and falling back to `type #id` for other
/// resource kinds.
fn floating_ip_binding(
    ip: &timeweb_rs::models::FloatingIp,
    server_names: &std::collections::HashMap<i64, String>
) -> String {
    use timeweb_rs::models::FloatingIpResourceId;

    let Some(resource_id) = ip.resource_id.as_deref() else {
        return String::new();
    };
    let id_text = match resource_id {
        FloatingIpResourceId::Number(n) => format!("{n}"),
        FloatingIpResourceId::String(s) => s.clone()
    };
    let resource_type = ip.resource_type.clone().unwrap_or_default();
    if resource_type == "server"
        && let Ok(id) = id_text.parse::<i64>()
        && let Some(name) = server_names.get(&id)
    {
        return name.clone();
    }
    if resource_type.is_empty() {
        format!("#{id_text}")
    } else {
        format!("{resource_type} #{id_text}")
    }
}

/// Renders a serde-tagged enum as its wire string (e.g. `Frameworks::NextJs`
/// becomes `next.js`), falling back to an empty string.
fn enum_label<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default()
}

/// Shortens a git commit SHA to its first seven characters.
fn short_commit(sha: &str) -> String {
    sha.chars().take(7).collect()
}

#[expect(clippy::too_many_lines)]
#[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
async fn refresh_all(
    config: &timeweb_rs::apis::configuration::Configuration,
    app: &mut tui::app::App
) {
    use tui::app::{
        AccountInfo, AiAgentSummary, AppSummary, BalancerSummary, DatabaseSummary,
        DedicatedServerSummary, DomainSummary, FirewallSummary, FloatingIpSummary, ImageSummary,
        K8sSummary, KnowledgeBaseSummary, MailSummary, NetworkDriveSummary, ProjectSummary,
        RegistrySummary, S3Summary, ServerSummary, VpcSummary
    };

    let c = config.clone();
    let cfg = &c;

    let (
        account_res,
        servers_res,
        dbs_res,
        s3_res,
        k8s_res,
        projects_res,
        balancers_res,
        registries_res,
        domains_res,
        firewalls_res,
        floating_ips_res,
        images_res,
        network_drives_res,
        vpcs_res,
        dedicated_servers_res,
        mails_res,
        apps_res,
        ai_agents_res,
        knowledge_bases_res,
        ssh_keys_res,
        finances_res
    ) = tokio::join!(
        timeweb_rs::apis::account_api::get_account_status(&c),
        fetch_all_pages(move |limit, offset| {
            Box::pin(async move {
                timeweb_rs::apis::servers_api::get_servers(cfg, Some(limit), Some(offset))
                    .await
                    .map(|r| (r.servers, r.meta.total))
            })
        }),
        fetch_all_pages(move |limit, offset| {
            Box::pin(async move {
                timeweb_rs::apis::databases_api::get_database_clusters(
                    cfg,
                    Some(limit),
                    Some(offset)
                )
                .await
                .map(|r| (r.dbs, r.meta.total))
            })
        }),
        timeweb_rs::apis::s3_api::get_storages(&c),
        fetch_all_pages(move |limit, offset| {
            Box::pin(async move {
                timeweb_rs::apis::kubernetes_api::get_clusters(cfg, Some(limit), Some(offset))
                    .await
                    .map(|r| (r.clusters, r.meta.total))
            })
        }),
        timeweb_rs::apis::projects_api::get_projects(&c),
        fetch_all_pages(move |limit, offset| {
            Box::pin(async move {
                timeweb_rs::apis::balancers_api::get_balancers(cfg, Some(limit), Some(offset))
                    .await
                    .map(|r| (r.balancers, r.meta.total))
            })
        }),
        timeweb_rs::apis::container_registry_api::get_registries(&c),
        fetch_all_pages(move |limit, offset| {
            Box::pin(async move {
                timeweb_rs::apis::domains_api::get_domains(
                    cfg,
                    Some(limit),
                    Some(offset),
                    None,
                    None,
                    None,
                    None
                )
                .await
                .map(|r| (r.domains, r.meta.total))
            })
        }),
        fetch_all_pages(move |limit, offset| {
            Box::pin(async move {
                timeweb_rs::apis::firewall_api::get_groups(cfg, Some(limit), Some(offset))
                    .await
                    .map(|r| (r.groups, r.meta.total))
            })
        }),
        timeweb_rs::apis::floating_ip_api::get_floating_ips(&c),
        fetch_all_pages(move |limit, offset| {
            Box::pin(async move {
                timeweb_rs::apis::images_api::get_images(cfg, Some(limit), Some(offset))
                    .await
                    .map(|r| (r.images, r.meta.total))
            })
        }),
        timeweb_rs::apis::network_drives_api::get_network_drives(&c),
        timeweb_rs::apis::vpc_api::get_vpcs(&c),
        timeweb_rs::apis::dedicated_servers_api::get_dedicated_servers(&c),
        fetch_all_pages(move |limit, offset| {
            Box::pin(async move {
                timeweb_rs::apis::mail_api::get_all_mailboxes_v2(
                    cfg,
                    Some(limit),
                    Some(offset),
                    None
                )
                .await
                .map(|r| (r.mailboxes, r.meta.total))
            })
        }),
        timeweb_rs::apis::apps_api::get_apps(&c),
        timeweb_rs::apis::ai_agents_api::get_agents(&c),
        timeweb_rs::apis::knowledge_bases_api::get_knowledgebases_v2(&c),
        timeweb_rs::apis::ssh_api::get_keys(&c),
        timeweb_rs::apis::payments_api::get_finances(&c)
    );

    let err_of = |e: Option<String>, name: &str| e.map(|msg| format!("{name}: {msg}"));
    app.last_load_errors = [
        err_of(
            account_res.as_ref().err().map(ToString::to_string),
            "account"
        ),
        err_of(
            servers_res.as_ref().err().map(ToString::to_string),
            "servers"
        ),
        err_of(dbs_res.as_ref().err().map(ToString::to_string), "databases"),
        err_of(s3_res.as_ref().err().map(ToString::to_string), "s3"),
        err_of(
            k8s_res.as_ref().err().map(ToString::to_string),
            "kubernetes"
        ),
        err_of(
            projects_res.as_ref().err().map(ToString::to_string),
            "projects"
        ),
        err_of(
            balancers_res.as_ref().err().map(ToString::to_string),
            "balancers"
        ),
        err_of(
            registries_res.as_ref().err().map(ToString::to_string),
            "registries"
        ),
        err_of(
            domains_res.as_ref().err().map(ToString::to_string),
            "domains"
        ),
        err_of(
            firewalls_res.as_ref().err().map(ToString::to_string),
            "firewall"
        ),
        err_of(
            floating_ips_res.as_ref().err().map(ToString::to_string),
            "floating IPs"
        ),
        err_of(images_res.as_ref().err().map(ToString::to_string), "images"),
        err_of(
            network_drives_res.as_ref().err().map(ToString::to_string),
            "network drives"
        ),
        err_of(vpcs_res.as_ref().err().map(ToString::to_string), "VPCs"),
        err_of(
            dedicated_servers_res
                .as_ref()
                .err()
                .map(ToString::to_string),
            "dedicated servers"
        ),
        err_of(mails_res.as_ref().err().map(ToString::to_string), "mail"),
        err_of(apps_res.as_ref().err().map(ToString::to_string), "apps"),
        err_of(
            ai_agents_res.as_ref().err().map(ToString::to_string),
            "AI agents"
        ),
        err_of(
            knowledge_bases_res.as_ref().err().map(ToString::to_string),
            "knowledge bases"
        ),
        err_of(
            ssh_keys_res.as_ref().err().map(ToString::to_string),
            "SSH keys"
        ),
        err_of(
            finances_res.as_ref().err().map(ToString::to_string),
            "finances"
        )
    ]
    .into_iter()
    .flatten()
    .collect();

    let mut account_id: i64 = 0;
    let mut login = String::new();
    let mut balance = String::new();
    let mut account_status = String::from("active");

    if let Ok(resp) = account_res {
        account_id = resp.status.company_info.id;
        login = resp.status.login.clone().unwrap_or_default();
        if resp.status.is_blocked || resp.status.is_permanent_blocked {
            account_status = String::from("blocked");
        }
    }
    if let Ok(ref resp) = finances_res {
        let f = &resp.finances;
        balance = format!("{:.2} {}", f.balance, f.currency);
    }
    app.update_account(AccountInfo {
        login,
        account_id,
        balance,
        status: account_status
    });

    let mut server_names: std::collections::HashMap<i64, String> =
        std::collections::HashMap::new();
    if let Ok(servers) = servers_res {
        for s in &servers {
            server_names.insert(s.id, s.name.clone());
        }
        let summaries: Vec<ServerSummary> = servers
            .iter()
            .map(|s| ServerSummary {
                id:       s.id as i32,
                name:     s.name.clone(),
                status:   format!("{:?}", s.status),
                cpu:      s.cpu as i32,
                ram_mb:   s.ram as i32,
                disk_gb:  server_disk_gb(s),
                ip:       server_public_ip(s),
                location: s.location.clone()
            })
            .collect();
        app.update_servers(summaries);
    }

    if let Ok(dbs) = dbs_res {
        let summaries: Vec<DatabaseSummary> = dbs
            .iter()
            .map(|d| DatabaseSummary {
                id:      d.id as i32,
                name:    d.name.clone(),
                status:  format!("{:?}", d.status),
                engine:  d.r#type.clone(),
                size_mb: d
                    .disk
                    .as_ref()
                    .and_then(|disk| disk.as_deref())
                    .map_or(0, |disk| (disk.size / 1024.0) as i64)
            })
            .collect();
        app.update_databases(summaries);
    }

    if let Ok(resp) = s3_res {
        let summaries: Vec<S3Summary> = resp
            .buckets
            .iter()
            .map(|b| S3Summary {
                id:           b.id as i32,
                name:         b.name.clone(),
                region:       b.location.clone(),
                size_kb:      b.disk_stats.size as i64,
                object_count: b.object_amount as i64
            })
            .collect();
        app.update_s3(summaries);
    }

    if let Ok(clusters) = k8s_res {
        let summaries: Vec<K8sSummary> = clusters
            .iter()
            .map(|c| K8sSummary {
                id:      c.id,
                name:    c.name.clone(),
                status:  c.status.clone(),
                version: c.k8s_version.clone(),
                cpu:     c.cpu.unwrap_or(0),
                ram_mb:  c.ram.unwrap_or(0),
                disk_gb: c.disk.unwrap_or(0)
            })
            .collect();
        app.update_k8s(summaries);
    }

    if let Ok(resp) = projects_res {
        let mut count_handles = Vec::with_capacity(resp.projects.len());
        for p in &resp.projects {
            let task_cfg = c.clone();
            let project_id = p.id as i32;
            count_handles.push(tokio::spawn(async move {
                timeweb_rs::apis::projects_api::get_all_project_resources(&task_cfg, project_id)
                    .await
                    .map(|r| {
                        (
                            r.servers.len() as i32,
                            r.databases.len() as i32,
                            r.buckets.len() as i32,
                            r.clusters.len() as i32,
                            r.balancers.len() as i32,
                            r.dedicated_servers.len() as i32,
                            r.apps
                                .into_iter()
                                .flatten()
                                .next()
                                .map_or(0, |v| v.len() as i32)
                        )
                    })
                    .map_err(|e| e.to_string())
            }));
        }
        let mut summaries = Vec::with_capacity(resp.projects.len());
        for (p, handle) in resp.projects.iter().zip(count_handles) {
            let mut summary = ProjectSummary {
                id: p.id as i32,
                name: p.name.clone(),
                ..Default::default()
            };
            match handle.await {
                Ok(Ok((servers, databases, buckets, clusters, balancers, dedicated, apps))) => {
                    summary.server_count = servers;
                    summary.database_count = databases;
                    summary.bucket_count = buckets;
                    summary.cluster_count = clusters;
                    summary.balancer_count = balancers;
                    summary.dedicated_count = dedicated;
                    summary.app_count = apps;
                }
                Ok(Err(e)) => {
                    app.last_load_errors
                        .push(format!("project '{}' resources: {e}", p.name));
                }
                Err(e) => {
                    app.last_load_errors
                        .push(format!("project '{}' resources: {e}", p.name));
                }
            }
            summaries.push(summary);
        }
        app.update_projects(summaries);
    }

    if let Ok(balancers) = balancers_res {
        let summaries: Vec<BalancerSummary> = balancers
            .iter()
            .map(|b| BalancerSummary {
                id:       b.id as i32,
                name:     b.name.clone(),
                status:   format!("{:?}", b.status),
                ip:       b.ips.first().cloned().unwrap_or_default(),
                location: b.location.clone()
            })
            .collect();
        app.update_balancers(summaries);
    }

    if let Ok(resp) = registries_res
        && let Some(registries) = resp.container_registry_list
    {
        let summaries: Vec<RegistrySummary> = registries
            .iter()
            .map(|r| RegistrySummary {
                id:        r.id,
                name:      r.name.clone(),
                disk_used: i64::from(r.disk_stats.used),
                disk_size: i64::from(r.disk_stats.size)
            })
            .collect();
        app.update_registries(summaries);
    }

    if let Ok(domains) = domains_res {
        let summaries: Vec<DomainSummary> = domains
            .iter()
            .map(|d| DomainSummary {
                id:           d.id as i32,
                name:         d.fqdn.clone(),
                status:       format!("{:?}", d.domain_status),
                auto_prolong: d.is_autoprolong_enabled.unwrap_or(false)
            })
            .collect();
        app.update_domains(summaries);
    }

    if let Ok(groups) = firewalls_res {
        let summaries: Vec<FirewallSummary> = groups
            .iter()
            .map(|g| FirewallSummary {
                id:     g.id.clone(),
                name:   g.name.clone(),
                policy: g.policy.to_string()
            })
            .collect();
        app.update_firewalls(summaries);
    }

    if let Ok(resp) = floating_ips_res {
        let summaries: Vec<FloatingIpSummary> = resp
            .ips
            .iter()
            .map(|ip| FloatingIpSummary {
                id:          ip.id.clone(),
                ip:          ip.ip.clone().unwrap_or_default(),
                status:      if ip.resource_id.is_some() {
                    String::from("attached")
                } else {
                    String::from("available")
                },
                server_name: floating_ip_binding(ip, &server_names)
            })
            .collect();
        app.update_floating_ips(summaries);
    }

    if let Ok(images) = images_res {
        let summaries: Vec<ImageSummary> = images
            .iter()
            .map(|img| ImageSummary {
                id:      img.id.clone(),
                name:    img.name.clone(),
                size_mb: i64::from(img.size),
                status:  format!("{:?}", img.status)
            })
            .collect();
        app.update_images(summaries);
    }

    if let Ok(resp) = network_drives_res {
        let summaries: Vec<NetworkDriveSummary> = resp
            .network_drives
            .iter()
            .map(|nd| NetworkDriveSummary {
                id:      nd.id.clone(),
                name:    nd.name.clone(),
                size_gb: nd.size as i64,
                status:  format!("{:?}", nd.status)
            })
            .collect();
        app.update_network_drives(summaries);
    }

    if let Ok(resp) = vpcs_res {
        let summaries: Vec<VpcSummary> = resp
            .vpcs
            .iter()
            .map(|v| VpcSummary {
                id:       v.id.clone(),
                name:     v.name.clone(),
                subnet:   v.subnet_v4.clone(),
                location: v.location.clone()
            })
            .collect();
        app.update_vpcs(summaries);
    }

    if let Ok(resp) = dedicated_servers_res {
        let summaries: Vec<DedicatedServerSummary> = resp
            .dedicated_servers
            .iter()
            .map(|ds| DedicatedServerSummary {
                id:     ds.id as i32,
                name:   ds.name.clone(),
                status: format!("{:?}", ds.status),
                cpu:    ds.cpu_description.clone(),
                ram:    ds.ram_description.clone(),
                disk:   ds.hdd_description.clone(),
                ip:     ds.ip.clone().unwrap_or_default()
            })
            .collect();
        app.update_dedicated_servers(summaries);
    }

    if let Ok(mailboxes) = mails_res {
        let summaries: Vec<MailSummary> = mailboxes
            .iter()
            .map(|m| MailSummary {
                name:    format!("{}@{}", m.mailbox, m.fqdn),
                owner:   m.owner_full_name.clone(),
                comment: m.comment.clone()
            })
            .collect();
        app.update_mails(summaries);
    }

    if let Ok(resp) = apps_res {
        let summaries: Vec<AppSummary> = resp
            .apps
            .iter()
            .map(|a| AppSummary {
                id:          a.id as i32,
                name:        a.name.clone(),
                status:      enum_label(&a.status),
                ip:          a.ip.clone().unwrap_or_default(),
                location:    a.location.clone().unwrap_or_default(),
                app_type:    a.r#type.as_ref().map(enum_label).unwrap_or_default(),
                framework:   a.framework.as_deref().map(enum_label).unwrap_or_default(),
                language:    a.language.clone().unwrap_or_default(),
                branch:      a.branch_name.clone().unwrap_or_default(),
                commit:      a
                    .commit_sha
                    .as_deref()
                    .map(short_commit)
                    .unwrap_or_default(),
                auto_deploy: a.is_auto_deploy.unwrap_or(false),
                comment:     a.comment.clone().unwrap_or_default(),
                domains:     a
                    .domains
                    .as_ref()
                    .map(|d| d.iter().filter_map(|x| x.fqdn.clone()).collect())
                    .unwrap_or_default()
            })
            .collect();
        app.update_apps(summaries);
    }

    if let Ok(resp) = ai_agents_res {
        let summaries: Vec<AiAgentSummary> = resp
            .agents
            .iter()
            .map(|a| AiAgentSummary {
                id:           a.id as i32,
                name:         a.name.clone(),
                status:       format!("{:?}", a.status),
                tokens_used:  a.used_tokens as i64,
                tokens_total: a.total_tokens as i64
            })
            .collect();
        app.update_ai_agents(summaries);
    }

    if let Ok(resp) = knowledge_bases_res {
        let summaries: Vec<KnowledgeBaseSummary> = resp
            .knowledge_bases
            .iter()
            .map(|kb| KnowledgeBaseSummary {
                id:             kb.id as i32,
                name:           kb.name.clone(),
                document_count: kb.documents_count as i32,
                status:         format!("{:?}", kb.status)
            })
            .collect();
        app.update_knowledge_bases(summaries);
    }

    if let Ok(resp) = ssh_keys_res {
        let keys: Vec<String> = resp.ssh_keys.iter().map(|k| k.name.clone()).collect();
        app.update_ssh_keys(keys);
    }

    if let Ok(resp) = finances_res {
        let f = resp.finances;
        let data = vec![format!("Balance: {:.2} {}", f.balance, f.currency)];
        app.update_finances(data);
    }

    if app.last_load_errors.is_empty() {
        app.status_message = Some("Resources loaded successfully".to_string());
    } else {
        app.error_message = Some(format!(
            "{} resource loads failed \u{2014} see events log",
            app.last_load_errors.len()
        ));
    }
}
