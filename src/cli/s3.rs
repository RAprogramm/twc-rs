// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! S3 storage subcommands.

use clap::Subcommand;

/// S3 storage subcommands.
#[derive(Subcommand, Debug)]
pub enum S3Commands {
    /// List all S3 storages.
    List {
        /// Maximum number of storages to return.
        #[arg(long)]
        limit: Option<i32>,

        /// Number of storages to skip.
        #[arg(long)]
        offset: Option<i32>
    },
    /// Show detailed info for a storage.
    Info {
        /// Storage ID.
        #[arg(long)]
        id: i32
    },
    /// Create a new S3 storage.
    Create {
        /// Storage name.
        #[arg(long)]
        name: String,

        /// Preset ID for the storage.
        #[arg(short = 'p', long)]
        preset_id: Option<f64>
    },
    /// Delete a storage by ID.
    Delete {
        /// Storage ID.
        #[arg(long)]
        id: i32
    },
    /// Update storage settings.
    Update {
        /// Storage ID.
        #[arg(long)]
        id: i32,

        /// New storage description.
        #[arg(long)]
        description: Option<String>
    },
    /// List users for a storage.
    UserList {
        /// Storage ID.
        #[arg(long)]
        id: i32
    },
    /// Update a storage user.
    UserUpdate {
        /// Storage user ID.
        #[arg(long)]
        user_id: i32
    },
    /// Transfer a storage.
    Transfer {
        /// Target storage ID (reserved for future use).
        #[arg(long)]
        target_id: Option<i32>
    },
    /// List subdomains for a storage.
    SubdomainList {
        /// Storage ID.
        #[arg(long)]
        id: i32
    },
    /// Add a subdomain to a storage.
    SubdomainAdd {
        /// Storage ID.
        #[arg(long)]
        id: i32,

        /// Subdomain name.
        #[arg(long)]
        subdomain: String
    },
    /// Delete a subdomain from a storage.
    SubdomainDelete {
        /// Storage ID.
        #[arg(long)]
        id: i32,

        /// Subdomain name.
        #[arg(long)]
        subdomain: String
    },
    /// List available storage presets.
    PresetList,
    /// Print an s3cmd config file for a storage.
    Genconfig {
        /// Storage ID.
        #[arg(long)]
        id: i32
    }
}
