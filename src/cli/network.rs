// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Floating IP and VPC subcommands.

use clap::Subcommand;

/// Floating IP subcommands.
#[derive(Subcommand, Debug)]
pub enum IpCommands {
    /// List all floating IPs.
    List,
    /// Show detailed info about a floating IP.
    Info {
        /// Floating IP ID.
        #[arg(long)]
        id: String
    },
    /// Create a new floating IP in an availability zone.
    Create {
        /// Availability zone (e.g. spb-1, msk-1, ams-1).
        #[arg(long)]
        availability_zone: String
    },
    /// Attach a floating IP to a resource.
    Attach {
        /// Floating IP ID.
        #[arg(long)]
        id:          String,
        /// Resource ID to bind to.
        #[arg(long)]
        resource_id: i32
    },
    /// Detach a floating IP from its resource.
    Detach {
        /// Floating IP ID.
        #[arg(long)]
        id: String
    },
    /// Update a floating IP's comment.
    Set {
        /// Floating IP ID.
        #[arg(long)]
        id:      String,
        /// New comment.
        #[arg(long)]
        comment: Option<String>
    },
    /// Delete a floating IP by ID.
    Delete {
        /// Floating IP ID.
        #[arg(long)]
        id: String
    }
}

/// VPC subcommands.
#[derive(Subcommand, Debug)]
pub enum VpcCommands {
    /// List all virtual networks.
    List,
    /// Show detailed information about a VPC.
    Info {
        /// VPC ID.
        #[arg(long)]
        id: String
    },
    /// Create a new VPC.
    Create {
        /// VPC name.
        #[arg(long)]
        name:      String,
        /// IPv4 subnet mask (e.g. 192.168.0.0/24).
        #[arg(long)]
        subnet_v4: String,
        /// Location (e.g. ru-1).
        #[arg(long)]
        location:  String
    },
    /// Update a VPC's name and/or description.
    Set {
        /// VPC ID.
        #[arg(long)]
        id:          String,
        /// New name.
        #[arg(long)]
        name:        Option<String>,
        /// New description.
        #[arg(long)]
        description: Option<String>
    },
    /// List network ports of a VPC.
    Ports {
        /// VPC ID.
        #[arg(long)]
        id: String
    },
    /// Delete a VPC by ID.
    Delete {
        /// VPC ID.
        #[arg(long)]
        id: String
    }
}
