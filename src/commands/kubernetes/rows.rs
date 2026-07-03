// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Table row types for the kubernetes command output.

use std::fmt;

use tabled::Tabled;

/// Compact row for the cluster list table.
#[derive(Tabled)]
pub(super) struct ClusterRow {
    #[tabled(rename = "ID")]
    pub(super) id:             i32,
    #[tabled(rename = "Name")]
    pub(super) name:           String,
    #[tabled(rename = "Status")]
    pub(super) status:         String,
    #[tabled(rename = "Version")]
    pub(super) k8s_version:    String,
    #[tabled(rename = "Driver")]
    pub(super) network_driver: String,
    #[tabled(rename = "Created")]
    pub(super) created_at:     String
}

impl fmt::Display for ClusterRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {}",
            self.id,
            self.name,
            self.status,
            self.k8s_version,
            self.network_driver,
            self.created_at
        )
    }
}

/// Compact row for the node group table.
#[derive(Tabled)]
pub(super) struct NodeGroupRow {
    #[tabled(rename = "ID")]
    pub(super) id:         i32,
    #[tabled(rename = "Name")]
    pub(super) name:       String,
    #[tabled(rename = "Node Count")]
    pub(super) node_count: i32,
    #[tabled(rename = "Preset")]
    pub(super) preset_id:  i32,
    #[tabled(rename = "Created")]
    pub(super) created_at: String
}

impl fmt::Display for NodeGroupRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.id, self.name, self.node_count, self.preset_id, self.created_at
        )
    }
}

/// Compact row for the node table.
#[derive(Tabled)]
pub(super) struct NodeRow {
    #[tabled(rename = "ID")]
    pub(super) id:      i32,
    #[tabled(rename = "Type")]
    pub(super) type_:   String,
    #[tabled(rename = "Status")]
    pub(super) status:  String,
    #[tabled(rename = "CPU")]
    pub(super) cpu:     i32,
    #[tabled(rename = "RAM (MB)")]
    pub(super) ram:     i32,
    #[tabled(rename = "Disk (GB)")]
    pub(super) disk:    i32,
    #[tabled(rename = "IP")]
    pub(super) node_ip: String
}

impl fmt::Display for NodeRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {} {}",
            self.id, self.type_, self.status, self.cpu, self.ram, self.disk, self.node_ip
        )
    }
}

/// Compact row for the addon table.
#[derive(Tabled)]
pub(super) struct AddonRow {
    #[tabled(rename = "ID")]
    pub(super) id:          i32,
    #[tabled(rename = "Type")]
    pub(super) type_:       String,
    #[tabled(rename = "Status")]
    pub(super) status:      String,
    #[tabled(rename = "Version")]
    pub(super) version:     String,
    #[tabled(rename = "Config Type")]
    pub(super) config_type: String
}

impl fmt::Display for AddonRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.id, self.type_, self.status, self.version, self.config_type
        )
    }
}

/// Compact row for the preset table.
#[derive(Tabled)]
pub(super) struct PresetRow {
    #[tabled(rename = "Type")]
    pub(super) preset_type: String,
    #[tabled(rename = "CPU")]
    pub(super) cpu:         String,
    #[tabled(rename = "RAM (MB)")]
    pub(super) ram:         String,
    #[tabled(rename = "Disk (GB)")]
    pub(super) disk:        String,
    #[tabled(rename = "Price")]
    pub(super) price:       String
}

impl fmt::Display for PresetRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.preset_type, self.cpu, self.ram, self.disk, self.price
        )
    }
}
