// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Managed database subcommands.

use clap::Subcommand;

/// Database subcommands.
#[derive(Subcommand, Debug)]
pub enum DatabaseCommands {
    /// List all databases.
    List {
        /// Maximum number of databases to return.
        #[arg(long)]
        limit: Option<i32>,

        /// Number of databases to skip.
        #[arg(long)]
        offset: Option<i32>
    },
    /// Show detailed info for a database.
    Info {
        /// Database ID.
        #[arg(long)]
        id: i32
    },
    /// Create a new database.
    Create {
        /// Database name.
        #[arg(long)]
        name: String,

        /// Database engine type (mysql, postgres, redis, mongodb, opensearch,
        /// clickhouse, kafka, rabbitmq).
        #[arg(long)]
        type_: String,

        /// Preset ID for the database.
        #[arg(short = 'p', long)]
        preset_id: i32
    },
    /// Delete a database by ID.
    Delete {
        /// Database ID.
        #[arg(long)]
        id: i32
    },
    /// Update database settings.
    Update {
        /// Database ID.
        #[arg(long)]
        id: i32,

        /// New database name.
        #[arg(long)]
        name: Option<String>
    },
    /// List backups for a database.
    BackupList {
        /// Database ID.
        #[arg(long)]
        id: i32
    },
    /// Create a backup for a database.
    BackupCreate {
        /// Database ID.
        #[arg(long)]
        id: i32
    },
    /// List users for a database.
    UserList {
        /// Database ID.
        #[arg(long)]
        id: i32
    },
    /// Create a user for a database.
    UserCreate {
        /// Database ID.
        #[arg(long)]
        db_id: i32,

        /// Database user login name.
        #[arg(long)]
        login: String,

        /// Database user password.
        #[arg(long)]
        password: String
    },
    /// Delete a user from a database.
    UserDelete {
        /// Database ID.
        #[arg(long)]
        db_id: i32,

        /// Database user login name.
        #[arg(long)]
        user_name: String
    },
    /// List available database presets.
    PresetList,
    /// List available database cluster types (engines and versions).
    ListTypes,
    /// List individual database instances within a cluster.
    ListInstances {
        /// Database cluster ID.
        #[arg(long)]
        id: i32
    }
}
