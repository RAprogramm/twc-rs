// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Firewall group and rule subcommands.

use clap::Subcommand;

/// Firewall management subcommands.
#[derive(Subcommand, Debug)]
pub enum FirewallCommands {
    /// List all firewall groups.
    List {
        /// Maximum number of groups to return.
        #[arg(long)]
        limit: Option<i32>,

        /// Number of groups to skip.
        #[arg(long)]
        offset: Option<i32>
    },
    /// Show detailed info for a firewall group.
    Info {
        /// Firewall group ID.
        #[arg(long)]
        id: String
    },
    /// Create a new firewall group.
    Create {
        /// Group name.
        #[arg(long)]
        name: String
    },
    /// Delete a firewall group by ID.
    Delete {
        /// Firewall group ID.
        #[arg(long)]
        id: String
    },
    /// Update firewall group settings.
    Update {
        /// Firewall group ID.
        #[arg(long)]
        id: String,

        /// New group name.
        #[arg(long)]
        name: Option<String>
    },
    /// List rules for a firewall group.
    RuleList {
        /// Firewall group ID.
        #[arg(long)]
        id: String
    },
    /// Create a rule for a firewall group.
    RuleCreate {
        /// Firewall group ID.
        #[arg(long)]
        id: String
    },
    /// Delete a rule from a firewall group.
    RuleDelete {
        /// Firewall group ID.
        #[arg(long)]
        id: String,

        /// Rule ID to delete.
        #[arg(long)]
        rule_id: String
    },
    /// List resources for a firewall group.
    ResourceList {
        /// Firewall group ID.
        #[arg(long)]
        id: String
    },
    /// Add a resource to a firewall group.
    ResourceAdd {
        /// Firewall group ID.
        #[arg(long)]
        id: String,

        /// Resource ID to add.
        #[arg(long)]
        resource_id: String
    },
    /// Remove a resource from a firewall group.
    ResourceRemove {
        /// Firewall group ID.
        #[arg(long)]
        id: String,

        /// Resource ID to remove.
        #[arg(long)]
        resource_id: String
    }
}
