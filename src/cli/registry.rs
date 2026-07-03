// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Container registry subcommands.

use clap::Subcommand;

/// Container registry subcommands.
#[derive(Subcommand, Debug)]
pub enum RegistryCommands {
    /// List all container registries.
    List {
        /// Maximum number of registries to return (not supported by API).
        #[arg(long)]
        limit: Option<i32>,

        /// Number of registries to skip (not supported by API).
        #[arg(long)]
        offset: Option<i32>
    },
    /// Show detailed info for a registry.
    Info {
        /// Registry ID.
        #[arg(long)]
        id: i32
    },
    /// Create a new container registry.
    Create {
        /// Registry name (3-48 chars, lowercase alphanumeric and hyphens).
        #[arg(long)]
        name: String
    },
    /// Delete a registry by ID.
    Delete {
        /// Registry ID.
        #[arg(long)]
        id: i32
    },
    /// Update registry settings.
    Update {
        /// Registry ID.
        #[arg(long)]
        id: i32,

        /// New registry description.
        #[arg(long)]
        description: Option<String>
    },
    /// List repositories for a registry.
    RepoList {
        /// Registry ID.
        #[arg(long)]
        id: i32
    },
    /// List available registry presets.
    PresetList
}
