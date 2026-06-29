// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use clap::{Parser, Subcommand};

/// Professional CLI tool for managing Timeweb Cloud infrastructure.
#[derive(Parser, Debug)]
#[command(
    name = "twc-rs",
    version,
    about = "Timeweb Cloud CLI — manage servers, SSH keys, and projects"
)]
pub struct Cli {
    /// Output format: table (default), json, or quiet.
    #[arg(
        short,
        long,
        global = true,
        default_value = "table",
        env = "TWC_OUTPUT"
    )]
    pub format: String,

    /// API token override (overrides config file and `TWC_TOKEN` env).
    #[arg(short, long, global = true, env = "TWC_TOKEN")]
    pub token: Option<String>,

    #[command(subcommand)]
    pub command: Commands
}

/// Available top-level commands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage cloud servers.
    #[command(subcommand)]
    Server(ServerCommands),

    /// Manage SSH keys.
    #[command(subcommand)]
    Ssh(SshCommands),

    /// Manage projects.
    #[command(subcommand)]
    Project(ProjectCommands),

    /// Manage databases.
    #[command(subcommand)]
    Database(DatabaseCommands),

    /// Manage S3 storages.
    #[command(subcommand)]
    S3(S3Commands),

    /// Configure twc-rs settings.
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Authenticate with Timeweb Cloud (guided browser flow).
    #[command(subcommand)]
    Auth(AuthCommands),

    /// Open the interactive dashboard.
    Dashboard {
        /// Refresh interval in seconds.
        #[arg(short, long, default_value_t = 5)]
        interval: u64
    }
}

/// Server-related subcommands.
#[derive(Subcommand, Debug)]
pub enum ServerCommands {
    /// List all cloud servers.
    List {
        /// Maximum number of servers to return.
        #[arg(short, long)]
        limit: Option<i32>,

        /// Number of servers to skip.
        #[arg(short, long)]
        offset: Option<i32>
    },
    /// Show detailed info for a server.
    Info {
        /// Server ID.
        #[arg(short, long)]
        id: i32
    },
    /// Delete a server by ID.
    Delete {
        /// Server ID.
        #[arg(short, long)]
        id: i32
    },
    /// Reboot a server by ID.
    Reboot {
        /// Server ID.
        #[arg(short, long)]
        id: i32
    }
}

/// SSH key subcommands.
#[derive(Subcommand, Debug)]
pub enum SshCommands {
    /// List all SSH keys.
    List,
    /// Add an SSH key from a file or stdin.
    Add {
        /// Human-readable name for the key.
        #[arg(short, long)]
        name: String,

        /// Path to the public key file. Reads from stdin if omitted.
        #[arg(short, long)]
        file: Option<String>,

        /// Mark this key as default for new servers.
        #[arg(long)]
        default: bool
    },
    /// Delete an SSH key by ID.
    Delete {
        /// SSH key ID.
        #[arg(short, long)]
        id: i32
    }
}

/// Project subcommands.
#[derive(Subcommand, Debug)]
pub enum ProjectCommands {
    /// List all projects.
    List,
    /// Create a new project.
    Create {
        /// Project name (max 255 chars).
        #[arg(short, long)]
        name: String,

        /// Project description (max 255 chars).
        #[arg(short, long)]
        description: Option<String>
    },
    /// Delete a project by ID.
    Delete {
        /// Project ID.
        #[arg(short, long)]
        id: i32
    }
}

/// Database subcommands.
#[derive(Subcommand, Debug)]
pub enum DatabaseCommands {
    /// List all databases.
    List {
        /// Maximum number of databases to return.
        #[arg(short, long)]
        limit: Option<i32>,

        /// Number of databases to skip.
        #[arg(short, long)]
        offset: Option<i32>
    },
    /// Show detailed info for a database.
    Info {
        /// Database ID.
        #[arg(short, long)]
        id: i32
    },
    /// Create a new database.
    Create {
        /// Database name.
        #[arg(short, long)]
        name: String,

        /// Database engine type (mysql, postgres, redis, mongodb, opensearch,
        /// clickhouse, kafka, rabbitmq).
        #[arg(short, long)]
        type_: String,

        /// Preset ID for the database.
        #[arg(short = 'p', long)]
        preset_id: i32
    },
    /// Delete a database by ID.
    Delete {
        /// Database ID.
        #[arg(short, long)]
        id: i32
    },
    /// Update database settings.
    Update {
        /// Database ID.
        #[arg(short, long)]
        id: i32,

        /// New database name.
        #[arg(short, long)]
        name: Option<String>
    },
    /// Restart a database by ID.
    Restart {
        /// Database ID.
        #[arg(short, long)]
        id: i32
    },
    /// List backups for a database.
    BackupList {
        /// Database ID.
        #[arg(short, long)]
        id: i32
    },
    /// Create a backup for a database.
    BackupCreate {
        /// Database ID.
        #[arg(short, long)]
        id: i32
    },
    /// List users for a database.
    UserList {
        /// Database ID.
        #[arg(short, long)]
        id: i32
    },
    /// Create a user for a database.
    UserCreate {
        /// Database ID.
        #[arg(short, long)]
        db_id: i32,

        /// Database user login name.
        #[arg(short, long)]
        login: String,

        /// Database user password.
        #[arg(short, long)]
        password: String
    },
    /// Delete a user from a database.
    UserDelete {
        /// Database ID.
        #[arg(short, long)]
        db_id: i32,

        /// Database user login name.
        #[arg(short, long)]
        user_name: String
    },
    /// List available database presets.
    PresetList
}

/// Configuration subcommands.
#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show the current configuration.
    Show,
    /// Set the API token.
    SetToken {
        /// The Timeweb Cloud API token.
        #[arg(short, long)]
        token: String
    }
}

/// Authentication subcommands.
#[derive(Subcommand, Debug)]
pub enum AuthCommands {
    /// Run the guided browser authentication flow.
    Flow,
    /// Show current authentication status.
    Status,
    /// Remove stored token from keyring and config.
    Logout,
    /// Accept a token directly (for CI/CD).
    Token {
        /// The API token to store.
        #[arg(short, long)]
        token: String
    }
}

/// S3 storage subcommands.
#[derive(Subcommand, Debug)]
pub enum S3Commands {
    /// List all S3 storages.
    List {
        /// Maximum number of storages to return.
        #[arg(short, long)]
        limit: Option<i32>,

        /// Number of storages to skip.
        #[arg(short, long)]
        offset: Option<i32>
    },
    /// Show detailed info for a storage.
    Info {
        /// Storage ID.
        #[arg(short, long)]
        id: i32
    },
    /// Create a new S3 storage.
    Create {
        /// Storage name.
        #[arg(short, long)]
        name: String,

        /// Preset ID for the storage.
        #[arg(short = 'p', long)]
        preset_id: Option<f64>
    },
    /// Delete a storage by ID.
    Delete {
        /// Storage ID.
        #[arg(short, long)]
        id: i32
    },
    /// Update storage settings.
    Update {
        /// Storage ID.
        #[arg(short, long)]
        id: i32,

        /// New storage description.
        #[arg(short, long)]
        description: Option<String>
    },
    /// List users for a storage.
    UserList {
        /// Storage ID.
        #[arg(short, long)]
        id: i32
    },
    /// Update a storage user.
    UserUpdate {
        /// Storage user ID.
        #[arg(short, long)]
        user_id: i32
    },
    /// Transfer a storage.
    Transfer {
        /// Target storage ID (reserved for future use).
        #[arg(short, long)]
        target_id: Option<i32>
    },
    /// List subdomains for a storage.
    SubdomainList {
        /// Storage ID.
        #[arg(short, long)]
        id: i32
    },
    /// Add a subdomain to a storage.
    SubdomainAdd {
        /// Storage ID.
        #[arg(short, long)]
        id: i32,

        /// Subdomain name.
        #[arg(short, long)]
        subdomain: String
    },
    /// Delete a subdomain from a storage.
    SubdomainDelete {
        /// Storage ID.
        #[arg(short, long)]
        id: i32,

        /// Subdomain name.
        #[arg(short, long)]
        subdomain: String
    },
    /// List available storage presets.
    PresetList
}

#[cfg(test)]
mod tests;
