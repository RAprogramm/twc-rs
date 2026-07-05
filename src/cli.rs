// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use clap::{Parser, Subcommand, ValueEnum};

mod apps;
mod balancers;
mod databases;
mod domains;
mod firewall;
mod images;
mod kubernetes;
mod network;
mod projects;
mod registry;
mod s3;
mod servers;
mod settings;
mod ssh;

pub use apps::AppsCommands;
pub use balancers::BalancerCommands;
pub use databases::DatabaseCommands;
pub use domains::DomainCommands;
pub use firewall::FirewallCommands;
pub use images::ImageCommands;
pub use kubernetes::KubernetesCommands;
pub use network::{IpCommands, VpcCommands};
pub use projects::ProjectCommands;
pub use registry::RegistryCommands;
pub use s3::S3Commands;
pub use servers::ServerCommands;
pub use settings::{AccountCommands, AuthCommands, ConfigCommands};
pub use ssh::SshCommands;

/// UI language selectable on the command line.
#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum LangArg {
    /// English.
    En,
    /// Russian.
    Ru
}

/// Shell to generate a completion script for.
#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum ShellArg {
    /// Bash.
    Bash,
    /// Zsh.
    Zsh,
    /// Fish.
    Fish,
    /// PowerShell.
    Powershell,
    /// Elvish.
    Elvish,
    /// Nushell.
    Nushell
}

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

    /// Use a named profile's token from the config file.
    #[arg(long, global = true, env = "TWC_PROFILE")]
    pub profile: Option<String>,

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

    /// Manage Kubernetes clusters.
    #[command(subcommand)]
    Kubernetes(KubernetesCommands),

    /// Manage container registries.
    #[command(subcommand)]
    Registry(RegistryCommands),

    /// Manage load balancers.
    #[command(subcommand)]
    Balancer(BalancerCommands),

    /// Manage domains.
    #[command(subcommand)]
    Domain(DomainCommands),

    /// Manage firewall groups.
    #[command(subcommand)]
    Firewall(FirewallCommands),

    /// Manage cloud apps.
    #[command(subcommand)]
    Apps(AppsCommands),

    /// Manage disk images.
    #[command(subcommand)]
    Image(ImageCommands),

    /// Manage floating IPs.
    #[command(subcommand)]
    Ip(IpCommands),

    /// Manage virtual networks (VPC).
    #[command(subcommand)]
    Vpc(VpcCommands),

    /// Show account information.
    #[command(subcommand)]
    Account(AccountCommands),

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
    },

    /// Generate a shell completion script (print to stdout).
    Completions {
        /// Target shell.
        #[arg(value_enum)]
        shell: ShellArg
    },

    /// Check the local installation for conflicting copies in PATH.
    Doctor
}

#[cfg(test)]
mod tests;
