// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

mod rows;

use rows::{AddonRow, ClusterRow, NodeGroupRow, NodeRow, PresetRow};
use rust_i18n::t;
use tabled::Tabled;
use timeweb_rs::{
    apis::{configuration::Configuration, kubernetes_api},
    models as k8s_models
};

use crate::{error::TwcError, output::OutputFormat};

/// Formats an i32 value as a string.
fn fmt_i32(v: i32) -> String {
    v.to_string()
}

/// Formats an f64 value as a string.
fn fmt_f64(v: f64) -> String {
    v.to_string()
}

/// Lists all Kubernetes clusters.
///
/// # Overview
///
/// Fetches clusters from the Timeweb Cloud API and displays them
/// in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list(
    config: &Configuration,
    limit: Option<i32>,
    offset: Option<i32>,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = kubernetes_api::get_clusters(config, limit, offset).await?;

    let rows: Vec<ClusterRow> = resp
        .clusters
        .iter()
        .map(|c| ClusterRow {
            id:             c.id,
            name:           c.name.clone(),
            status:         c.status.clone(),
            k8s_version:    c.k8s_version.clone(),
            network_driver: format!("{:?}", c.network_driver),
            created_at:     c.created_at.to_string()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_clusters_found"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.clusters)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for c in &resp.clusters {
                println!("{}", c.id);
            }
        }
    }
    Ok(())
}

/// Shows detailed info for a single Kubernetes cluster.
///
/// # Overview
///
/// Fetches cluster details by ID and displays them.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn info(config: &Configuration, id: i32, format: OutputFormat) -> Result<(), TwcError> {
    let resp = kubernetes_api::get_cluster(config, id).await?;
    let cluster = &resp.cluster;

    match format {
        OutputFormat::Table => {
            println!("ID:                 {}", cluster.id);
            println!("Name:               {}", cluster.name);
            println!("Status:             {}", cluster.status);
            println!("K8s Version:        {}", cluster.k8s_version);
            println!("Network Driver:     {:?}", cluster.network_driver);
            println!("Ingress:            {}", cluster.ingress);
            println!("Preset ID:          {}", cluster.preset_id);
            println!(
                "CPU:                {:?}",
                cluster.cpu.map(|c| c.to_string())
            );
            println!(
                "RAM:                {:?}",
                cluster.ram.map(|r| format!("{r} MB"))
            );
            println!(
                "Disk:               {:?}",
                cluster.disk.map(|d| format!("{d} GB"))
            );
            println!("Availability Zone:  {:?}", cluster.availability_zone);
            println!("Project ID:         {:?}", cluster.project_id);
            println!("Description:        {}", cluster.description);
            println!("Created at:         {}", cluster.created_at);
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.cluster)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            println!(
                "{}\t{}\t{}\t{}",
                cluster.id, cluster.name, cluster.status, cluster.k8s_version
            );
        }
    }
    Ok(())
}

/// Creates a new Kubernetes cluster.
///
/// # Overview
///
/// Creates a cluster with the given name, Kubernetes version, and network
/// driver. Uses default preset if not specified.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn create(
    config: &Configuration,
    name: &str,
    k8s_version: &str,
    format: OutputFormat
) -> Result<(), TwcError> {
    let mut req = k8s_models::ClusterIn::new(
        name.to_string(),
        k8s_version.to_string(),
        k8s_models::cluster_in::NetworkDriver::default()
    );
    req.is_ingress = Some(true);
    req.is_k8s_dashboard = Some(true);

    let resp = kubernetes_api::create_cluster(config, req).await?;
    let cluster = &resp.cluster;

    match format {
        OutputFormat::Table => {
            println!(
                "{}",
                t!("cli.cluster_created", name => cluster.name, id => cluster.id)
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.cluster)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            println!("{}", cluster.id);
        }
    }
    Ok(())
}

/// Deletes a Kubernetes cluster by ID.
///
/// # Overview
///
/// Sends a delete request for the specified cluster.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn delete(config: &Configuration, id: i32) -> Result<(), TwcError> {
    kubernetes_api::delete_cluster(config, id, None, None).await?;
    println!("{}", t!("cli.cluster_deleted", id => id));
    Ok(())
}

/// Updates a Kubernetes cluster by ID.
///
/// # Overview
///
/// Updates the cluster name via the Timeweb Cloud API.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn update(
    config: &Configuration,
    id: i32,
    name: Option<&str>,
    format: OutputFormat
) -> Result<(), TwcError> {
    let mut edit = k8s_models::ClusterEdit::new();
    if let Some(n) = name {
        edit.name = Some(n.to_string());
    }
    let resp = kubernetes_api::update_cluster(config, id, edit).await?;
    let cluster = &resp.cluster;

    match format {
        OutputFormat::Table => {
            println!(
                "{}",
                t!("cli.cluster_updated", name => cluster.name, id => cluster.id)
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.cluster)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", cluster.id, cluster.name);
        }
    }
    Ok(())
}

/// Lists node groups for a Kubernetes cluster.
///
/// # Overview
///
/// Fetches node groups for the specified cluster and displays them
/// in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn nodegroup_list(
    config: &Configuration,
    id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = kubernetes_api::get_cluster_node_groups(config, id).await?;

    let rows: Vec<NodeGroupRow> = resp
        .node_groups
        .iter()
        .map(|g| NodeGroupRow {
            id:         g.id,
            name:       g.name.clone(),
            node_count: g.node_count,
            preset_id:  g.preset_id,
            created_at: g.created_at.to_string()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_node_groups_found"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.node_groups)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for g in &resp.node_groups {
                println!("{}\t{}", g.id, g.name);
            }
        }
    }
    Ok(())
}

/// Creates a new node group for a Kubernetes cluster.
///
/// # Overview
///
/// Creates a node group with the given name and default node count (1).
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn nodegroup_create(
    config: &Configuration,
    cluster_id: i32,
    name: &str,
    format: OutputFormat
) -> Result<(), TwcError> {
    let mut req = k8s_models::NodeGroupIn::new(name.to_string(), 1);
    req.is_autohealing = Some(true);

    let resp = kubernetes_api::create_cluster_node_group(config, cluster_id, req).await?;
    let group = &resp.node_group;

    match format {
        OutputFormat::Table => {
            println!(
                "{}",
                t!("cli.node_group_created", name => group.name, cluster_id => cluster_id, id => group.id)
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.node_group)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", group.id, group.name);
        }
    }
    Ok(())
}

/// Deletes a node group from a Kubernetes cluster.
///
/// # Overview
///
/// Sends a delete request for the specified node group.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn nodegroup_delete(
    config: &Configuration,
    cluster_id: i32,
    group_id: i32
) -> Result<(), TwcError> {
    kubernetes_api::delete_cluster_node_group(config, cluster_id, group_id).await?;
    println!(
        "{}",
        t!("cli.node_group_deleted", group_id => group_id, cluster_id => cluster_id)
    );
    Ok(())
}

/// Lists nodes for a Kubernetes cluster.
///
/// # Overview
///
/// Fetches all nodes for the specified cluster and displays them
/// in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn node_list(
    config: &Configuration,
    id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = kubernetes_api::get_cluster_nodes(config, id).await?;

    let rows: Vec<NodeRow> = resp
        .nodes
        .iter()
        .map(|n| NodeRow {
            id:      n.id,
            type_:   n.r#type.clone(),
            status:  n.status.clone(),
            cpu:     n.cpu,
            ram:     n.ram,
            disk:    n.disk,
            node_ip: n.node_ip.clone()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_nodes_found"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.nodes)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for n in &resp.nodes {
                println!("{}\t{}\t{}", n.id, n.node_ip, n.status);
            }
        }
    }
    Ok(())
}

/// Lists installed addons for a Kubernetes cluster.
///
/// # Overview
///
/// Fetches installed addons for the specified cluster and displays them
/// in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn addon_list(
    config: &Configuration,
    id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = kubernetes_api::get_kubernetes_addons(config, id).await?;

    let rows: Vec<AddonRow> = resp
        .addons
        .iter()
        .map(|a| AddonRow {
            id:          a.id,
            type_:       a.r#type.clone(),
            status:      a.status.clone(),
            version:     a.version.clone(),
            config_type: a.config_type.clone()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_addons_installed"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.addons)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for a in &resp.addons {
                println!("{}\t{}\t{}", a.id, a.r#type, a.status);
            }
        }
    }
    Ok(())
}

/// Installs an addon on a Kubernetes cluster.
///
/// # Overview
///
/// Installs an addon by name using default configuration (basic config type,
/// empty yaml config, latest version).
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn addon_install(
    config: &Configuration,
    cluster_id: i32,
    addon_name: &str
) -> Result<(), TwcError> {
    let req = k8s_models::ClusterIn1::new(
        addon_name.to_string(),
        k8s_models::cluster_in_1::ConfigType::Basic,
        String::new(),
        "latest".to_string()
    );
    kubernetes_api::post_kubernetes_addons(config, cluster_id, req).await?;
    println!(
        "{}",
        t!("cli.addon_install_started", name => addon_name, cluster_id => cluster_id)
    );
    Ok(())
}

/// Deletes an addon from a Kubernetes cluster.
///
/// # Overview
///
/// Looks up the addon by name and deletes it from the specified cluster.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn addon_delete(
    config: &Configuration,
    cluster_id: i32,
    addon_name: &str
) -> Result<(), TwcError> {
    let addons = kubernetes_api::get_kubernetes_addons(config, cluster_id).await?;

    let target = addons.addons.iter().find(|a| a.r#type == addon_name);

    let Some(addon) = target else {
        return Err(TwcError::Api(format!(
            "addon '{addon_name}' not found in cluster {cluster_id}"
        )));
    };

    kubernetes_api::delete_kubernetes_addons(config, cluster_id, addon.id).await?;
    println!(
        "{}",
        t!("cli.addon_deleted", name => addon_name, cluster_id => cluster_id)
    );
    Ok(())
}

/// Lists available Kubernetes presets.
///
/// # Overview
///
/// Fetches Kubernetes presets (worker and master node configurations)
/// from the Timeweb Cloud API and displays them in the requested format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn preset_list(config: &Configuration, format: OutputFormat) -> Result<(), TwcError> {
    let resp = kubernetes_api::get_kubernetes_presets(config).await?;

    let rows: Vec<PresetRow> = resp
        .k8s_presets
        .iter()
        .map(|p| match p {
            k8s_models::K8SPresetsInner::Worker(w) => PresetRow {
                preset_type: "worker".to_string(),
                cpu:         fmt_i32(w.cpu),
                ram:         fmt_i32(w.ram),
                disk:        fmt_i32(w.disk),
                price:       fmt_f64(w.price)
            },
            k8s_models::K8SPresetsInner::Master(m) => PresetRow {
                preset_type: "master".to_string(),
                cpu:         fmt_i32(m.cpu),
                ram:         fmt_i32(m.ram),
                disk:        fmt_i32(m.disk),
                price:       fmt_f64(m.price)
            }
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_presets_found"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.k8s_presets)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for p in &resp.k8s_presets {
                match p {
                    k8s_models::K8SPresetsInner::Worker(w) => {
                        println!("worker\t{}\t{}", fmt_i32(w.cpu), fmt_i32(w.ram));
                    }
                    k8s_models::K8SPresetsInner::Master(m) => {
                        println!("master\t{}\t{}", fmt_i32(m.cpu), fmt_i32(m.ram));
                    }
                }
            }
        }
    }
    Ok(())
}

/// Lists available Kubernetes versions.
///
/// # Overview
///
/// Fetches available Kubernetes versions from the Timeweb Cloud API
/// and displays them in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn version_list(config: &Configuration, format: OutputFormat) -> Result<(), TwcError> {
    let resp = kubernetes_api::get_k8_s_versions(config).await?;

    match format {
        OutputFormat::Table => {
            if resp.k8s_versions.is_empty() {
                println!("{}", t!("cli.no_versions_found"));
            } else {
                #[derive(Tabled)]
                struct VersionRow {
                    #[tabled(rename = "Version")]
                    version: String
                }
                let rows: Vec<VersionRow> = resp
                    .k8s_versions
                    .iter()
                    .map(|v| VersionRow {
                        version: v.clone()
                    })
                    .collect();
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.k8s_versions)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for v in &resp.k8s_versions {
                println!("{v}");
            }
        }
    }
    Ok(())
}

/// Lists available Kubernetes network drivers.
///
/// # Overview
///
/// Fetches the network drivers supported for Kubernetes clusters from the
/// Timeweb Cloud API and displays them in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list_network_drivers(
    config: &Configuration,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = kubernetes_api::get_k8_s_network_drivers(config).await?;

    match format {
        OutputFormat::Table => {
            if resp.network_drivers.is_empty() {
                println!("{}", t!("cli.no_network_drivers_found"));
            } else {
                #[derive(Tabled)]
                struct NetworkDriverRow {
                    #[tabled(rename = "Network Driver")]
                    network_driver: String
                }
                let rows: Vec<NetworkDriverRow> = resp
                    .network_drivers
                    .iter()
                    .map(|d| NetworkDriverRow {
                        network_driver: d.clone()
                    })
                    .collect();
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            if let Some(out) = crate::output::serialized(format, &resp.network_drivers) {
                println!("{}", out?);
            }
        }
        OutputFormat::Quiet => {
            for d in &resp.network_drivers {
                println!("{d}");
            }
        }
    }
    Ok(())
}

/// Gets the kubeconfig for a Kubernetes cluster.
///
/// # Overview
///
/// Fetches the kubeconfig YAML file for the specified cluster.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn kubeconfig(config: &Configuration, id: i32) -> Result<(), TwcError> {
    let kubeconfig = kubernetes_api::get_cluster_kubeconfig(config, id).await?;
    println!("{kubeconfig}");
    Ok(())
}

/// Shows cluster resources (deprecated).
///
/// # Overview
///
/// Fetches cluster resources via the deprecated resources endpoint.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
#[allow(deprecated)]
pub async fn resources(
    config: &Configuration,
    id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = kubernetes_api::get_cluster_resources(config, id).await?;

    match format {
        OutputFormat::Table => {
            println!("{}", t!("cli.cluster_resources_header", id => id));
            let json = serde_json::to_string_pretty(&resp.resources)
                .map_err(|e| TwcError::Api(e.to_string()))?;
            println!("{json}");
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.resources)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            let json = serde_json::to_string(&resp.resources)
                .map_err(|e| TwcError::Api(e.to_string()))?;
            println!("{json}");
        }
    }
    Ok(())
}
