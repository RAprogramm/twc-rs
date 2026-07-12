// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Streamed dashboard loading: every endpoint runs as its own task and sends
//! its slice to the UI the moment it responds, so fast resources paint
//! immediately instead of waiting for the slowest request.

use timeweb_rs::{apis::configuration::Configuration, authenticated};
use tokio::sync::mpsc::UnboundedSender;

use crate::tui::{self, app::DataSlice, event::AppEvent};

type Tx = UnboundedSender<AppEvent>;

fn send(tx: &Tx, slice: DataSlice) {
    let _ = tx.send(AppEvent::Slice(Box::new(slice)));
}

fn send_result<T>(
    tx: &Tx,
    name: &str,
    result: Result<T, impl ToString>,
    into: impl FnOnce(T) -> DataSlice
) {
    match result {
        Ok(value) => send(tx, into(value)),
        Err(e) => send(tx, DataSlice::Error(format!("{name}: {}", e.to_string())))
    }
}

pub(crate) fn spawn_refresh_loop(
    tx: Tx,
    token: String,
    interval: u64
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let period = tokio::time::Duration::from_secs(interval.max(2));
        loop {
            run_cycle(&tx, &token).await;
            tokio::time::sleep(period).await;
        }
    })
}

pub(crate) fn spawn_one_shot_refresh(tx: Tx, token: String) {
    tokio::spawn(async move {
        run_cycle(&tx, &token).await;
    });
}

/// Runs one full streamed load cycle: spawns every endpoint loader
/// concurrently, and reports completion once all of them settled.
async fn run_cycle(tx: &Tx, token: &str) {
    if tx.send(AppEvent::LoadStarted).is_err() {
        return;
    }
    let config = authenticated(token.to_string());
    let mut set = tokio::task::JoinSet::new();

    macro_rules! task {
        ($f:ident) => {
            let c = config.clone();
            let t = tx.clone();
            set.spawn(async move { $f(&c, &t).await });
        };
    }

    task!(load_account);
    task!(load_servers_and_floating_ips);
    task!(load_databases);
    task!(load_s3);
    task!(load_k8s);
    task!(load_projects);
    task!(load_balancers);
    task!(load_registries);
    task!(load_domains);
    task!(load_firewalls);
    task!(load_images);
    task!(load_network_drives);
    task!(load_vpcs);
    task!(load_dedicated_servers);
    task!(load_mails);
    task!(load_apps);
    task!(load_ai_agents);
    task!(load_knowledge_bases);
    task!(load_ssh_keys);
    task!(load_finances);

    while set.join_next().await.is_some() {}
    let _ = tx.send(AppEvent::LoadFinished);
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

async fn load_account(c: &Configuration, tx: &Tx) {
    use tui::app::AccountInfo;

    send_result(
        tx,
        "account",
        timeweb_rs::apis::account_api::get_account_status(c).await,
        |resp| {
            let status = if resp.status.is_blocked || resp.status.is_permanent_blocked {
                String::from("blocked")
            } else {
                String::from("active")
            };
            DataSlice::Account(AccountInfo {
                login: resp.status.login.clone().unwrap_or_default(),
                account_id: resp.status.company_info.id,
                balance: String::new(),
                status
            })
        }
    );
}

#[expect(clippy::cast_possible_truncation)]
async fn load_servers_and_floating_ips(c: &Configuration, tx: &Tx) {
    use tui::app::{FloatingIpSummary, ServerSummary};

    let mut server_names: std::collections::HashMap<i64, String> =
        std::collections::HashMap::new();

    let servers_res = fetch_all_pages(move |limit, offset| {
        Box::pin(async move {
            timeweb_rs::apis::servers_api::get_servers(c, Some(limit), Some(offset))
                .await
                .map(|r| (r.servers, r.meta.total))
        })
    })
    .await;

    match servers_res {
        Ok(servers) => {
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
            send(tx, DataSlice::Servers(summaries));
        }
        Err(e) => send(tx, DataSlice::Error(format!("servers: {e}")))
    }

    send_result(
        tx,
        "floating IPs",
        timeweb_rs::apis::floating_ip_api::get_floating_ips(c).await,
        |resp| {
            DataSlice::FloatingIps(
                resp.ips
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
                    .collect()
            )
        }
    );
}

#[expect(clippy::cast_possible_truncation)]
async fn load_databases(c: &Configuration, tx: &Tx) {
    use tui::app::DatabaseSummary;

    let res = fetch_all_pages(move |limit, offset| {
        Box::pin(async move {
            timeweb_rs::apis::databases_api::get_database_clusters(c, Some(limit), Some(offset))
                .await
                .map(|r| (r.dbs, r.meta.total))
        })
    })
    .await;
    send_result(tx, "databases", res, |dbs| {
        DataSlice::Databases(
            dbs.iter()
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
                .collect()
        )
    });
}

#[expect(clippy::cast_possible_truncation)]
async fn load_s3(c: &Configuration, tx: &Tx) {
    use tui::app::S3Summary;

    send_result(
        tx,
        "s3",
        timeweb_rs::apis::s3_api::get_storages(c).await,
        |resp| {
            DataSlice::S3(
                resp.buckets
                    .iter()
                    .map(|b| S3Summary {
                        id:           b.id as i32,
                        name:         b.name.clone(),
                        region:       b.location.clone(),
                        size_kb:      b.disk_stats.size as i64,
                        object_count: b.object_amount as i64
                    })
                    .collect()
            )
        }
    );
}

async fn load_k8s(c: &Configuration, tx: &Tx) {
    use tui::app::K8sSummary;

    let res = fetch_all_pages(move |limit, offset| {
        Box::pin(async move {
            timeweb_rs::apis::kubernetes_api::get_clusters(c, Some(limit), Some(offset))
                .await
                .map(|r| (r.clusters, r.meta.total))
        })
    })
    .await;
    send_result(tx, "kubernetes", res, |clusters| {
        DataSlice::K8s(
            clusters
                .iter()
                .map(|k| K8sSummary {
                    id:      k.id,
                    name:    k.name.clone(),
                    status:  k.status.clone(),
                    version: k.k8s_version.clone(),
                    cpu:     k.cpu.unwrap_or(0),
                    ram_mb:  k.ram.unwrap_or(0),
                    disk_gb: k.disk.unwrap_or(0)
                })
                .collect()
        )
    });
}

/// Loads the project list, streams it immediately with zero counts so the
/// sidebar fills at once, then streams it again with per-project resource
/// counts once those (parallel) fetches finish.
#[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
async fn load_projects(c: &Configuration, tx: &Tx) {
    use tui::app::ProjectSummary;

    let resp = match timeweb_rs::apis::projects_api::get_projects(c).await {
        Ok(resp) => resp,
        Err(e) => {
            send(tx, DataSlice::Error(format!("projects: {e}")));
            return;
        }
    };

    let mut summaries: Vec<ProjectSummary> = resp
        .projects
        .iter()
        .map(|p| ProjectSummary {
            id: p.id as i32,
            name: p.name.clone(),
            ..Default::default()
        })
        .collect();
    send(tx, DataSlice::Projects(summaries.clone()));

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
    for (summary, handle) in summaries.iter_mut().zip(count_handles) {
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
            Ok(Err(e)) => send(
                tx,
                DataSlice::Error(format!("project '{}' resources: {e}", summary.name))
            ),
            Err(e) => send(
                tx,
                DataSlice::Error(format!("project '{}' resources: {e}", summary.name))
            )
        }
    }
    send(tx, DataSlice::Projects(summaries));
}

#[expect(clippy::cast_possible_truncation)]
async fn load_balancers(c: &Configuration, tx: &Tx) {
    use tui::app::BalancerSummary;

    let res = fetch_all_pages(move |limit, offset| {
        Box::pin(async move {
            timeweb_rs::apis::balancers_api::get_balancers(c, Some(limit), Some(offset))
                .await
                .map(|r| (r.balancers, r.meta.total))
        })
    })
    .await;
    send_result(tx, "balancers", res, |balancers| {
        DataSlice::Balancers(
            balancers
                .iter()
                .map(|b| BalancerSummary {
                    id:       b.id as i32,
                    name:     b.name.clone(),
                    status:   format!("{:?}", b.status),
                    ip:       b.ips.first().cloned().unwrap_or_default(),
                    location: b.location.clone()
                })
                .collect()
        )
    });
}

async fn load_registries(c: &Configuration, tx: &Tx) {
    use tui::app::RegistrySummary;

    send_result(
        tx,
        "registries",
        timeweb_rs::apis::container_registry_api::get_registries(c).await,
        |resp| {
            DataSlice::Registries(
                resp.container_registry_list
                    .unwrap_or_default()
                    .iter()
                    .map(|r| RegistrySummary {
                        id:        r.id,
                        name:      r.name.clone(),
                        disk_used: i64::from(r.disk_stats.used),
                        disk_size: i64::from(r.disk_stats.size)
                    })
                    .collect()
            )
        }
    );
}

#[expect(clippy::cast_possible_truncation)]
async fn load_domains(c: &Configuration, tx: &Tx) {
    use tui::app::DomainSummary;

    let res = fetch_all_pages(move |limit, offset| {
        Box::pin(async move {
            timeweb_rs::apis::domains_api::get_domains(
                c,
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
    })
    .await;
    send_result(tx, "domains", res, |domains| {
        DataSlice::Domains(
            domains
                .iter()
                .map(|d| DomainSummary {
                    id:           d.id as i32,
                    name:         d.fqdn.clone(),
                    status:       format!("{:?}", d.domain_status),
                    auto_prolong: d.is_autoprolong_enabled.unwrap_or(false)
                })
                .collect()
        )
    });
}

async fn load_firewalls(c: &Configuration, tx: &Tx) {
    use tui::app::FirewallSummary;

    let res = fetch_all_pages(move |limit, offset| {
        Box::pin(async move {
            timeweb_rs::apis::firewall_api::get_groups(c, Some(limit), Some(offset))
                .await
                .map(|r| (r.groups, r.meta.total))
        })
    })
    .await;
    send_result(tx, "firewall", res, |groups| {
        DataSlice::Firewalls(
            groups
                .iter()
                .map(|g| FirewallSummary {
                    id:     g.id.clone(),
                    name:   g.name.clone(),
                    policy: g.policy.to_string()
                })
                .collect()
        )
    });
}

async fn load_images(c: &Configuration, tx: &Tx) {
    use tui::app::ImageSummary;

    let res = fetch_all_pages(move |limit, offset| {
        Box::pin(async move {
            timeweb_rs::apis::images_api::get_images(c, Some(limit), Some(offset))
                .await
                .map(|r| (r.images, r.meta.total))
        })
    })
    .await;
    send_result(tx, "images", res, |images| {
        DataSlice::Images(
            images
                .iter()
                .map(|img| ImageSummary {
                    id:      img.id.clone(),
                    name:    img.name.clone(),
                    size_mb: i64::from(img.size),
                    status:  format!("{:?}", img.status)
                })
                .collect()
        )
    });
}

async fn load_network_drives(c: &Configuration, tx: &Tx) {
    use tui::app::NetworkDriveSummary;

    send_result(
        tx,
        "network drives",
        timeweb_rs::apis::network_drives_api::get_network_drives(c).await,
        |resp| {
            DataSlice::NetworkDrives(
                resp.network_drives
                    .iter()
                    .map(|nd| NetworkDriveSummary {
                        id:      nd.id.clone(),
                        name:    nd.name.clone(),
                        size_gb: nd.size as i64,
                        status:  format!("{:?}", nd.status)
                    })
                    .collect()
            )
        }
    );
}

async fn load_vpcs(c: &Configuration, tx: &Tx) {
    use tui::app::VpcSummary;

    send_result(
        tx,
        "VPCs",
        timeweb_rs::apis::vpc_api::get_vpcs(c).await,
        |resp| {
            DataSlice::Vpcs(
                resp.vpcs
                    .iter()
                    .map(|v| VpcSummary {
                        id:       v.id.clone(),
                        name:     v.name.clone(),
                        subnet:   v.subnet_v4.clone(),
                        location: v.location.clone()
                    })
                    .collect()
            )
        }
    );
}

#[expect(clippy::cast_possible_truncation)]
async fn load_dedicated_servers(c: &Configuration, tx: &Tx) {
    use tui::app::DedicatedServerSummary;

    send_result(
        tx,
        "dedicated servers",
        timeweb_rs::apis::dedicated_servers_api::get_dedicated_servers(c).await,
        |resp| {
            DataSlice::DedicatedServers(
                resp.dedicated_servers
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
                    .collect()
            )
        }
    );
}

async fn load_mails(c: &Configuration, tx: &Tx) {
    use tui::app::MailSummary;

    let res = fetch_all_pages(move |limit, offset| {
        Box::pin(async move {
            timeweb_rs::apis::mail_api::get_all_mailboxes_v2(c, Some(limit), Some(offset), None)
                .await
                .map(|r| (r.mailboxes, r.meta.total))
        })
    })
    .await;
    send_result(tx, "mail", res, |mailboxes| {
        DataSlice::Mails(
            mailboxes
                .iter()
                .map(|m| MailSummary {
                    name:    format!("{}@{}", m.mailbox, m.fqdn),
                    owner:   m.owner_full_name.clone(),
                    comment: m.comment.clone()
                })
                .collect()
        )
    });
}

#[expect(clippy::cast_possible_truncation)]
async fn load_apps(c: &Configuration, tx: &Tx) {
    use tui::app::AppSummary;

    send_result(
        tx,
        "apps",
        timeweb_rs::apis::apps_api::get_apps(c).await,
        |resp| {
            DataSlice::Apps(
                resp.apps
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
                    .collect()
            )
        }
    );
}

#[expect(clippy::cast_possible_truncation)]
async fn load_ai_agents(c: &Configuration, tx: &Tx) {
    use tui::app::AiAgentSummary;

    send_result(
        tx,
        "AI agents",
        timeweb_rs::apis::ai_agents_api::get_agents(c).await,
        |resp| {
            DataSlice::AiAgents(
                resp.agents
                    .iter()
                    .map(|a| AiAgentSummary {
                        id:           a.id as i32,
                        name:         a.name.clone(),
                        status:       format!("{:?}", a.status),
                        tokens_used:  a.used_tokens as i64,
                        tokens_total: a.total_tokens as i64
                    })
                    .collect()
            )
        }
    );
}

#[expect(clippy::cast_possible_truncation)]
async fn load_knowledge_bases(c: &Configuration, tx: &Tx) {
    use tui::app::KnowledgeBaseSummary;

    send_result(
        tx,
        "knowledge bases",
        timeweb_rs::apis::knowledge_bases_api::get_knowledgebases_v2(c).await,
        |resp| {
            DataSlice::KnowledgeBases(
                resp.knowledge_bases
                    .iter()
                    .map(|kb| KnowledgeBaseSummary {
                        id:             kb.id as i32,
                        name:           kb.name.clone(),
                        document_count: kb.documents_count as i32,
                        status:         format!("{:?}", kb.status)
                    })
                    .collect()
            )
        }
    );
}

async fn load_ssh_keys(c: &Configuration, tx: &Tx) {
    send_result(
        tx,
        "SSH keys",
        timeweb_rs::apis::ssh_api::get_keys(c).await,
        |resp| DataSlice::SshKeys(resp.ssh_keys.iter().map(|k| k.name.clone()).collect())
    );
}

async fn load_finances(c: &Configuration, tx: &Tx) {
    send_result(
        tx,
        "finances",
        timeweb_rs::apis::payments_api::get_finances(c).await,
        |resp| {
            let f = resp.finances;
            let balance = format!("{:.2} {}", f.balance, f.currency);
            DataSlice::Finances {
                lines: vec![format!("Balance: {balance}")],
                balance
            }
        }
    );
}
