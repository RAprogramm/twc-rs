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

    /// Configure twc-rs settings.
    #[command(subcommand)]
    Config(ConfigCommands)
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
