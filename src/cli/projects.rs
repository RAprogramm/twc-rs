// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Project subcommands.

use clap::Subcommand;

/// Project subcommands.
#[derive(Subcommand, Debug)]
pub enum ProjectCommands {
    /// List all projects.
    List,
    /// Create a new project.
    Create {
        /// Project name (max 255 chars).
        #[arg(long)]
        name: String,

        /// Project description (max 255 chars).
        #[arg(long)]
        description: Option<String>
    },
    /// Delete a project by ID.
    Delete {
        /// Project ID.
        #[arg(long)]
        id: i32
    },
    /// Update a project's name and/or description.
    Set {
        /// Project ID.
        #[arg(long)]
        id:          i32,
        /// New project name (max 255 chars).
        #[arg(long)]
        name:        Option<String>,
        /// New project description (max 255 chars).
        #[arg(long)]
        description: Option<String>
    },
    /// List all resources in a project.
    Resources {
        /// Project ID.
        #[arg(long)]
        id: i32
    }
}
