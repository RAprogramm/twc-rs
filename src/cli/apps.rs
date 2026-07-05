// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! App platform subcommands and creation arguments.

use clap::Subcommand;

/// Cloud apps subcommands.
#[derive(Subcommand, Debug)]
pub enum AppsCommands {
    /// List all cloud apps.
    List,
    /// Show detailed info for a single app.
    Info {
        /// App ID.
        #[arg(long)]
        id: String
    },
    /// Delete an app by ID.
    Delete {
        /// App ID.
        #[arg(long)]
        id: String
    },
    /// List available app presets (tariffs).
    ListPresets,
    /// List configured VCS providers.
    ListVcsProviders,
    /// List repositories of a VCS provider.
    ListRepositories {
        /// VCS provider ID.
        #[arg(long)]
        provider_id: String
    },
    /// Create a new app from a connected VCS repository.
    Create(Box<AppCreateArgs>),
    /// Show runtime logs of an app.
    Logs {
        /// App ID.
        #[arg(long)]
        id:    String,
        /// Show only the last N lines (applied after date filters).
        #[arg(long)]
        tail:  Option<usize>,
        /// Show only lines logged at or after this moment; accepts
        /// `YYYY-MM-DD` (local midnight) or an RFC 3339 timestamp.
        #[arg(long, conflicts_with = "today")]
        since: Option<String>,
        /// Show only lines logged today (local time).
        #[arg(long)]
        today: bool
    },
    /// List deploys of an app, newest first.
    ListDeploys {
        /// App ID.
        #[arg(long)]
        id: String
    },
    /// Show build/deploy logs of a deploy (the latest one by default).
    DeployLogs {
        /// App ID.
        #[arg(long)]
        id:        String,
        /// Deploy ID (UUID); defaults to the most recent deploy.
        #[arg(long)]
        deploy_id: Option<String>,
        /// Include debug output.
        #[arg(long)]
        debug:     bool
    }
}

/// Arguments for `apps create` (boxed in the enum to keep variant sizes even).
#[derive(clap::Args, Debug)]
pub struct AppCreateArgs {
    /// App name.
    #[arg(long)]
    pub name:          String,
    /// Optional comment.
    #[arg(long)]
    pub comment:       Option<String>,
    /// VCS provider ID (UUID).
    #[arg(long)]
    pub provider_id:   String,
    /// Repository ID (UUID).
    #[arg(long)]
    pub repository_id: String,
    /// Preset (tariff) ID.
    #[arg(long)]
    pub preset_id:     i64,
    /// App type: backend or frontend.
    #[arg(long = "type")]
    pub app_type:      String,
    /// Framework (e.g. docker, react, next.js, django).
    #[arg(long)]
    pub framework:     String,
    /// Repository branch to build from.
    #[arg(long, default_value = "main")]
    pub branch:        String,
    /// Specific commit SHA (defaults to latest on the branch).
    #[arg(long)]
    pub commit_sha:    Option<String>,
    /// Build command.
    #[arg(long)]
    pub build_cmd:     Option<String>,
    /// Run command (required for backend apps).
    #[arg(long)]
    pub run_cmd:       Option<String>,
    /// Index directory starting with '/' (required for frontend apps).
    #[arg(long)]
    pub index_dir:     Option<String>,
    /// Enable automatic deploy on push.
    #[arg(long)]
    pub auto_deploy:   bool,
    /// Optional project ID to place the app in.
    #[arg(long)]
    pub project_id:    Option<i64>
}
