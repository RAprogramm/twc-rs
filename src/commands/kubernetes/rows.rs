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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cluster_row_display() {
        let row = ClusterRow {
            id: 1,
            name: "prod".to_string(),
            status: "ready".to_string(),
            k8s_version: "1.28".to_string(),
            network_driver: "calico".to_string(),
            created_at: "2026-01-01".to_string()
        };
        assert_eq!(row.to_string(), "1 prod ready 1.28 calico 2026-01-01");
    }

    #[test]
    fn node_group_row_display() {
        let row = NodeGroupRow {
            id: 1,
            name: "workers".to_string(),
            node_count: 3,
            preset_id: 10,
            created_at: "2026-01-01".to_string()
        };
        assert_eq!(row.to_string(), "1 workers 3 10 2026-01-01");
    }

    #[test]
    fn node_row_display() {
        let row = NodeRow {
            id: 1,
            type_: "worker".to_string(),
            status: "running".to_string(),
            cpu: 2,
            ram: 4096,
            disk: 40,
            node_ip: "10.0.0.1".to_string()
        };
        assert_eq!(row.to_string(), "1 worker running 2 4096 40 10.0.0.1");
    }

    #[test]
    fn addon_row_display() {
        let row = AddonRow {
            id: 1,
            type_: "ingress".to_string(),
            status: "installed".to_string(),
            version: "1.0".to_string(),
            config_type: "helm".to_string()
        };
        assert_eq!(row.to_string(), "1 ingress installed 1.0 helm");
    }

    #[test]
    fn preset_row_display() {
        let row = PresetRow {
            preset_type: "standard".to_string(),
            cpu: "2".to_string(),
            ram: "4096".to_string(),
            disk: "40".to_string(),
            price: "1000".to_string()
        };
        assert_eq!(row.to_string(), "standard 2 4096 40 1000");
    }
}
