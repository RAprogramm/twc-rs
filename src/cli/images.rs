// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Server image subcommands.

use clap::Subcommand;

/// Disk image subcommands.
#[derive(Subcommand, Debug)]
pub enum ImageCommands {
    /// List all disk images.
    List,
    /// Show detailed info for an image.
    Info {
        /// Image ID.
        #[arg(long)]
        id: String
    },
    /// Create a new image.
    Create {
        /// Image name.
        #[arg(long)]
        name:     String,
        /// Location where the image is created.
        #[arg(long)]
        location: String
    },
    /// Update an image's name.
    Set {
        /// Image ID.
        #[arg(long)]
        id:   String,
        /// New image name.
        #[arg(long)]
        name: Option<String>
    },
    /// Delete an image by ID.
    Delete {
        /// Image ID.
        #[arg(long)]
        id: String
    },
    /// Upload a local image file to an image.
    Upload {
        /// Image ID.
        #[arg(long)]
        id:   String,
        /// Path to the local image file.
        #[arg(long)]
        file: String
    }
}
