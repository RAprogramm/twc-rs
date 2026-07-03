// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Kubernetes cluster subcommands.

use clap::Subcommand;

/// Kubernetes subcommands.
#[derive(Subcommand, Debug)]
pub enum KubernetesCommands {
    /// List all Kubernetes clusters.
    List {
        /// Maximum number of clusters to return.
        #[arg(long)]
        limit: Option<i32>,

        /// Number of clusters to skip.
        #[arg(long)]
        offset: Option<i32>
    },
    /// Show detailed info for a cluster.
    Info {
        /// Cluster ID.
        #[arg(long)]
        id: i32
    },
    /// Create a new Kubernetes cluster.
    Create {
        /// Cluster name.
        #[arg(long)]
        name: String,

        /// Kubernetes version (e.g., 1.30).
        #[arg(long)]
        type_: String
    },
    /// Delete a cluster by ID.
    Delete {
        /// Cluster ID.
        #[arg(long)]
        id: i32
    },
    /// Update cluster settings.
    Update {
        /// Cluster ID.
        #[arg(long)]
        id: i32,

        /// New cluster name.
        #[arg(long)]
        name: Option<String>
    },
    /// List node groups for a cluster.
    NodegroupList {
        /// Cluster ID.
        #[arg(long)]
        id: i32
    },
    /// Create a node group for a cluster.
    NodegroupCreate {
        /// Cluster ID.
        #[arg(long)]
        id: i32,

        /// Node group name.
        #[arg(long)]
        name: String
    },
    /// Delete a node group from a cluster.
    NodegroupDelete {
        /// Cluster ID.
        #[arg(long)]
        id: i32,

        /// Node group ID.
        #[arg(long)]
        group_id: i32
    },
    /// List nodes for a cluster.
    NodeList {
        /// Cluster ID.
        #[arg(long)]
        id: i32
    },
    /// List installed addons for a cluster.
    AddonList {
        /// Cluster ID.
        #[arg(long)]
        id: i32
    },
    /// Install an addon on a cluster.
    AddonInstall {
        /// Cluster ID.
        #[arg(long)]
        id: i32,

        /// Addon name (e.g., calico, metrics-server).
        #[arg(long)]
        addon_name: String
    },
    /// Delete an addon from a cluster.
    AddonDelete {
        /// Cluster ID.
        #[arg(long)]
        id: i32,

        /// Addon name to delete.
        #[arg(long)]
        addon_name: String
    },
    /// List available Kubernetes presets.
    PresetList,

    /// List available Kubernetes versions.
    VersionList,

    /// List available Kubernetes network drivers.
    NetworkDrivers,

    /// Get kubeconfig for a cluster.
    Kubeconfig {
        /// Cluster ID.
        #[arg(long)]
        id: i32
    },

    /// Show cluster resources (deprecated).
    #[command(hide = true)]
    Resources {
        /// Cluster ID.
        #[arg(long)]
        id: i32
    }
}
