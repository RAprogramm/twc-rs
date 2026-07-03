// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! SSH key subcommands.

use clap::Subcommand;

/// SSH key subcommands.
#[derive(Subcommand, Debug)]
pub enum SshCommands {
    /// List all SSH keys.
    List,
    /// Add an SSH key from a file or stdin.
    Add {
        /// Human-readable name for the key.
        #[arg(long)]
        name: String,

        /// Path to the public key file. Reads from stdin if omitted.
        #[arg(long)]
        file: Option<String>,

        /// Mark this key as default for new servers.
        #[arg(long)]
        default: bool
    },
    /// Delete an SSH key by ID.
    Delete {
        /// SSH key ID.
        #[arg(long)]
        id: i32
    },
    /// Show detailed information about an SSH key.
    Info {
        /// SSH key ID.
        #[arg(long)]
        id: i32
    },
    /// Edit an SSH key's name and/or default flag.
    Edit {
        /// SSH key ID.
        #[arg(long)]
        id:      i32,
        /// New name for the key.
        #[arg(long)]
        name:    Option<String>,
        /// Mark this key as default for new servers.
        #[arg(long)]
        default: Option<bool>
    },
    /// Attach existing SSH key(s) to a cloud server.
    Attach {
        /// Target server ID.
        #[arg(long)]
        server: i32,
        /// SSH key ID to attach. Repeat to attach several at once.
        #[arg(long = "key")]
        key:    Vec<i32>
    },
    /// Detach an SSH key from a cloud server.
    Detach {
        /// Target server ID.
        #[arg(long)]
        server: i32,
        /// SSH key ID to detach.
        #[arg(long = "key")]
        key:    i32
    }
}
