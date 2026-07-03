// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Load balancer subcommands.

use clap::Subcommand;

/// Balancer subcommands.
#[derive(Subcommand, Debug)]
pub enum BalancerCommands {
    /// List all balancers.
    List {
        /// Maximum number of balancers to return.
        #[arg(long)]
        limit: Option<i32>,

        /// Number of balancers to skip.
        #[arg(long)]
        offset: Option<i32>
    },
    /// Show detailed info for a balancer.
    Info {
        /// Balancer ID.
        #[arg(long)]
        id: i32
    },
    /// Create a new balancer.
    Create {
        /// Balancer name.
        #[arg(long)]
        name: String
    },
    /// Delete a balancer by ID.
    Delete {
        /// Balancer ID.
        #[arg(long)]
        id: i32
    },
    /// Update balancer settings.
    Update {
        /// Balancer ID.
        #[arg(long)]
        id: i32,

        /// New balancer name.
        #[arg(long)]
        name: Option<String>
    },
    /// List rules for a balancer.
    RuleList {
        /// Balancer ID.
        #[arg(long)]
        id: i32
    },
    /// Create a rule for a balancer.
    RuleCreate {
        /// Balancer ID.
        #[arg(long)]
        id: i32
    },
    /// Delete a rule from a balancer.
    RuleDelete {
        /// Balancer ID.
        #[arg(long)]
        id: i32,

        /// Rule ID to delete.
        #[arg(long)]
        rule_id: i32
    },
    /// List IPs for a balancer.
    IpList {
        /// Balancer ID.
        #[arg(long)]
        id: i32
    },
    /// Add an IP to a balancer.
    IpAdd {
        /// Balancer ID.
        #[arg(long)]
        id: i32,

        /// IP address to add.
        #[arg(long)]
        ip: String
    },
    /// Remove an IP from a balancer.
    IpRemove {
        /// Balancer ID.
        #[arg(long)]
        id: i32,

        /// IP address to remove.
        #[arg(long)]
        ip: String
    },
    /// List available balancer presets.
    PresetList
}
