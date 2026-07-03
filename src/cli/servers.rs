// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Cloud server subcommands.

use clap::Subcommand;

/// Server-related subcommands.
#[derive(Subcommand, Debug)]
pub enum ServerCommands {
    /// List all cloud servers.
    List {
        /// Maximum number of servers to return.
        #[arg(long)]
        limit: Option<i32>,

        /// Number of servers to skip.
        #[arg(long)]
        offset: Option<i32>
    },
    /// Show detailed info for a server.
    Info {
        /// Server ID.
        #[arg(long)]
        id: i32
    },
    /// Delete a server by ID.
    Delete {
        /// Server ID.
        #[arg(long)]
        id: i32
    },
    /// Reboot a server by ID.
    Reboot {
        /// Server ID.
        #[arg(long)]
        id: i32
    },
    /// Power a server on.
    Start {
        /// Server ID.
        #[arg(long)]
        id: i32
    },
    /// Gracefully shut a server down.
    Shutdown {
        /// Server ID.
        #[arg(long)]
        id: i32
    },
    /// Clone a server by ID.
    Clone {
        /// Server ID.
        #[arg(long)]
        id: i32
    },
    /// Reset a server's root password.
    ResetPassword {
        /// Server ID.
        #[arg(long)]
        id: i32
    },
    /// List available server presets.
    ListPresets,
    /// List installable OS images.
    ListOs,
    /// List available pre-installable software.
    ListSoftware,
    /// List server configurators (custom builds).
    ListConfigurators,
    /// List the disks attached to a server.
    Disk {
        /// Server ID.
        #[arg(long)]
        id: i32
    },
    /// List the IP addresses of a server.
    Ip {
        /// Server ID.
        #[arg(long)]
        id: i32
    },
    /// Show the recent action history (logs) of a server.
    History {
        /// Server ID.
        #[arg(long)]
        id: i32
    },
    /// Set the NAT mode of a server's local network.
    SetNatMode {
        /// Server ID.
        #[arg(long)]
        id:       i32,
        /// One of: `dnat_and_snat`, `snat`, `no_nat`.
        #[arg(long)]
        nat_mode: String
    },
    /// Set the OS boot mode of a server (restarts the server).
    SetBootMode {
        /// Server ID.
        #[arg(long)]
        id:        i32,
        /// One of: `default`, `single`, `recovery_disk`.
        #[arg(long)]
        boot_mode: String
    },
    /// Resize a server to a different preset.
    Resize {
        /// Server ID.
        #[arg(long)]
        id:        i32,
        /// Target preset ID.
        #[arg(long)]
        preset_id: i32
    },
    /// Reinstall the OS of a server (wipes data).
    Reinstall {
        /// Server ID.
        #[arg(long)]
        id:    i32,
        /// OS image ID to install.
        #[arg(long)]
        os_id: i32
    },
    /// Create a new cloud server from a preset and OS image.
    Create {
        /// Server name (max 255 chars).
        #[arg(long)]
        name:              String,
        /// Preset (tariff) ID. Use `server list-presets` to list.
        #[arg(long)]
        preset_id:         i32,
        /// OS image ID. Use `server list-os` to list.
        #[arg(long)]
        os_id:             i32,
        /// Optional comment (max 255 chars).
        #[arg(long)]
        comment:           Option<String>,
        /// SSH key IDs to attach (repeatable).
        #[arg(long = "ssh-key")]
        ssh_key:           Vec<i32>,
        /// Project ID to place the server in.
        #[arg(long)]
        project_id:        Option<i32>,
        /// Availability zone (e.g. spb-1, msk-1, ams-1).
        #[arg(long)]
        availability_zone: Option<String>
    },
    /// Update a server's name and/or comment.
    Set {
        /// Server ID.
        #[arg(long)]
        id:      i32,
        /// New name.
        #[arg(long)]
        name:    Option<String>,
        /// New comment.
        #[arg(long)]
        comment: Option<String>
    },
    /// List disk backups of a server.
    BackupList {
        /// Server ID.
        #[arg(long)]
        id: i32
    },
    /// Create a disk backup of a server's system disk.
    BackupCreate {
        /// Server ID.
        #[arg(long)]
        id:      i32,
        /// Optional backup comment.
        #[arg(long)]
        comment: Option<String>
    }
}
