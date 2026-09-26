// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Table row types for the servers command output.

use std::fmt;

use tabled::Tabled;

/// Compact row for the server list table.
#[derive(Tabled)]
pub(super) struct ServerRow {
    #[tabled(rename = "ID")]
    pub(super) id:       String,
    #[tabled(rename = "Name")]
    pub(super) name:     String,
    #[tabled(rename = "Status")]
    pub(super) status:   String,
    #[tabled(rename = "CPU")]
    pub(super) cpu:      String,
    #[tabled(rename = "RAM (MB)")]
    pub(super) ram:      String,
    #[tabled(rename = "OS")]
    pub(super) os:       String,
    #[tabled(rename = "Location")]
    pub(super) location: String
}

impl fmt::Display for ServerRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {} {}",
            self.id, self.name, self.status, self.cpu, self.ram, self.os, self.location
        )
    }
}

/// Compact row for the server preset list table.
#[derive(Tabled)]
pub(super) struct PresetRow {
    #[tabled(rename = "ID")]
    pub(super) id:          String,
    #[tabled(rename = "Location")]
    pub(super) location:    String,
    #[tabled(rename = "CPU")]
    pub(super) cpu:         String,
    #[tabled(rename = "RAM (MB)")]
    pub(super) ram:         String,
    #[tabled(rename = "Disk (GB)")]
    pub(super) disk:        String,
    #[tabled(rename = "Price")]
    pub(super) price:       String,
    #[tabled(rename = "Description")]
    pub(super) description: String
}

impl fmt::Display for PresetRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {} {}",
            self.id, self.location, self.cpu, self.ram, self.disk, self.price, self.description
        )
    }
}

/// Compact row for the OS image list table.
#[derive(Tabled)]
pub(super) struct OsRow {
    #[tabled(rename = "ID")]
    pub(super) id:      String,
    #[tabled(rename = "Family")]
    pub(super) family:  String,
    #[tabled(rename = "Name")]
    pub(super) name:    String,
    #[tabled(rename = "Version")]
    pub(super) version: String
}

impl fmt::Display for OsRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.id, self.family, self.name, self.version
        )
    }
}

/// Compact row for the software list table.
#[derive(Tabled)]
pub(super) struct SoftwareRow {
    #[tabled(rename = "ID")]
    pub(super) id:            String,
    #[tabled(rename = "Name")]
    pub(super) name:          String,
    #[tabled(rename = "Installations")]
    pub(super) installations: String
}

impl fmt::Display for SoftwareRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.id, self.name, self.installations)
    }
}

/// Compact row for the configurator list table.
#[derive(Tabled)]
pub(super) struct ConfiguratorRow {
    #[tabled(rename = "ID")]
    pub(super) id:            String,
    #[tabled(rename = "Location")]
    pub(super) location:      String,
    #[tabled(rename = "Disk Type")]
    pub(super) disk_type:     String,
    #[tabled(rename = "CPU Frequency")]
    pub(super) cpu_frequency: String
}

impl fmt::Display for ConfiguratorRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.id, self.location, self.disk_type, self.cpu_frequency
        )
    }
}

/// Compact row for the server disk list table.
#[derive(Tabled)]
pub(super) struct DiskRow {
    #[tabled(rename = "ID")]
    pub(super) id:     String,
    #[tabled(rename = "Size (MB)")]
    pub(super) size:   String,
    #[tabled(rename = "Used (MB)")]
    pub(super) used:   String,
    #[tabled(rename = "Type")]
    pub(super) r#type: String,
    #[tabled(rename = "Status")]
    pub(super) status: String
}

impl fmt::Display for DiskRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.id, self.size, self.used, self.r#type, self.status
        )
    }
}

/// Compact row for the server IP list table.
#[derive(Tabled)]
pub(super) struct IpRow {
    #[tabled(rename = "IP")]
    pub(super) ip:      String,
    #[tabled(rename = "Type")]
    pub(super) r#type:  String,
    #[tabled(rename = "PTR")]
    pub(super) ptr:     String,
    #[tabled(rename = "Main")]
    pub(super) is_main: String
}

impl fmt::Display for IpRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.ip, self.r#type, self.ptr, self.is_main
        )
    }
}

/// Compact row for the server log/history table.
#[derive(Tabled)]
pub(super) struct LogRow {
    #[tabled(rename = "ID")]
    pub(super) id:        String,
    #[tabled(rename = "Logged At")]
    pub(super) logged_at: String,
    #[tabled(rename = "Event")]
    pub(super) event:     String
}

impl fmt::Display for LogRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.id, self.logged_at, self.event)
    }
}

/// Compact row for the server disk-backup list table.
#[derive(Tabled, serde::Serialize)]
pub(super) struct BackupRow {
    #[tabled(rename = "ID")]
    pub(super) id:         String,
    #[tabled(rename = "Disk ID")]
    pub(super) disk_id:    String,
    #[tabled(rename = "Status")]
    pub(super) status:     String,
    #[tabled(rename = "Created At")]
    pub(super) created_at: String,
    #[tabled(rename = "Size (MB)")]
    pub(super) size:       String
}

impl fmt::Display for BackupRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.id, self.disk_id, self.status, self.created_at, self.size
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_row_display() {
        let row = ServerRow {
            id: "1".to_string(),
            name: "web-1".to_string(),
            status: "On".to_string(),
            cpu: "2".to_string(),
            ram: "4096".to_string(),
            os: "Ubuntu 24.04".to_string(),
            location: "ru-1".to_string()
        };
        assert_eq!(
            row.to_string(),
            "1 web-1 On 2 4096 Ubuntu 24.04 ru-1"
        );
    }

    #[test]
    fn preset_row_display() {
        let row = PresetRow {
            id: "1".to_string(),
            location: "ru-1".to_string(),
            cpu: "2".to_string(),
            ram: "4096".to_string(),
            disk: "40".to_string(),
            price: "1000".to_string(),
            description: "Basic".to_string()
        };
        assert_eq!(
            row.to_string(),
            "1 ru-1 2 4096 40 1000 Basic"
        );
    }

    #[test]
    fn os_row_display() {
        let row = OsRow {
            id: "1".to_string(),
            family: "ubuntu".to_string(),
            name: "Ubuntu".to_string(),
            version: "24.04".to_string()
        };
        assert_eq!(row.to_string(), "1 ubuntu Ubuntu 24.04");
    }

    #[test]
    fn software_row_display() {
        let row = SoftwareRow {
            id: "1".to_string(),
            name: "Docker".to_string(),
            installations: "5".to_string()
        };
        assert_eq!(row.to_string(), "1 Docker 5");
    }

    #[test]
    fn configurator_row_display() {
        let row = ConfiguratorRow {
            id: "1".to_string(),
            location: "ru-1".to_string(),
            disk_type: "SSD".to_string(),
            cpu_frequency: "3.3".to_string()
        };
        assert_eq!(row.to_string(), "1 ru-1 SSD 3.3");
    }

    #[test]
    fn disk_row_display() {
        let row = DiskRow {
            id: "1".to_string(),
            size: "40960".to_string(),
            used: "10240".to_string(),
            r#type: "SSD".to_string(),
            status: "active".to_string()
        };
        assert_eq!(row.to_string(), "1 40960 10240 SSD active");
    }

    #[test]
    fn ip_row_display() {
        let row = IpRow {
            ip: "192.168.1.1".to_string(),
            r#type: "public".to_string(),
            ptr: "host.example.com".to_string(),
            is_main: "true".to_string()
        };
        assert_eq!(row.to_string(), "192.168.1.1 public host.example.com true");
    }

    #[test]
    fn log_row_display() {
        let row = LogRow {
            id: "1".to_string(),
            logged_at: "2026-01-01".to_string(),
            event: "created".to_string()
        };
        assert_eq!(row.to_string(), "1 2026-01-01 created");
    }

    #[test]
    fn backup_row_display() {
        let row = BackupRow {
            id: "1".to_string(),
            disk_id: "10".to_string(),
            status: "ready".to_string(),
            created_at: "2026-01-01".to_string(),
            size: "40960".to_string()
        };
        assert_eq!(row.to_string(), "1 10 ready 2026-01-01 40960");
    }

    #[test]
    fn backup_row_serializes_to_json() {
        let row = BackupRow {
            id: "1".to_string(),
            disk_id: "10".to_string(),
            status: "ready".to_string(),
            created_at: "2026-01-01".to_string(),
            size: "40960".to_string()
        };
        let json = serde_json::to_string(&row).unwrap();
        assert!(json.contains("\"id\":\"1\""));
        assert!(json.contains("\"disk_id\":\"10\""));
        assert!(json.contains("\"status\":\"ready\""));
    }
}
