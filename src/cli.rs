// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use clap::{Parser, Subcommand, ValueEnum};

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
    }
}

/// Server-related subcommands.
#[derive(Subcommand, Debug)]
pub enum ServerCommands {
    /// List all cloud servers.
    List {
        /// Maximum number of servers to return.
        #[arg(long)]
        limit: Option<i32>,

        /// Number of servers to skip.
        #[arg(long)]
        offset: Option<i32>
    },
    /// Show detailed info for a server.
    Info {
        /// Server ID.
        #[arg(long)]
        id: i32
    },
    /// Delete a server by ID.
    Delete {
        /// Server ID.
        #[arg(long)]
        id: i32
    },
    /// Reboot a server by ID.
    Reboot {
        /// Server ID.
        #[arg(long)]
        id: i32
    },
    /// Power a server on.
    Start {
        /// Server ID.
        #[arg(long)]
        id: i32
    },
    /// Gracefully shut a server down.
    Shutdown {
        /// Server ID.
        #[arg(long)]
        id: i32
    },
    /// Clone a server by ID.
    Clone {
        /// Server ID.
        #[arg(long)]
        id: i32
    },
    /// Reset a server's root password.
    ResetPassword {
        /// Server ID.
        #[arg(long)]
        id: i32
    },
    /// List available server presets.
    ListPresets,
    /// List installable OS images.
    ListOs,
    /// List available pre-installable software.
    ListSoftware,
    /// List server configurators (custom builds).
    ListConfigurators,
    /// List the disks attached to a server.
    Disk {
        /// Server ID.
        #[arg(long)]
        id: i32
    },
    /// List the IP addresses of a server.
    Ip {
        /// Server ID.
        #[arg(long)]
        id: i32
    },
    /// Show the recent action history (logs) of a server.
    History {
        /// Server ID.
        #[arg(long)]
        id: i32
    },
    /// Set the NAT mode of a server's local network.
    SetNatMode {
        /// Server ID.
        #[arg(long)]
        id:       i32,
        /// One of: `dnat_and_snat`, `snat`, `no_nat`.
        #[arg(long)]
        nat_mode: String
    },
    /// Set the OS boot mode of a server (restarts the server).
    SetBootMode {
        /// Server ID.
        #[arg(long)]
        id:        i32,
        /// One of: `default`, `single`, `recovery_disk`.
        #[arg(long)]
        boot_mode: String
    },
    /// Resize a server to a different preset.
    Resize {
        /// Server ID.
        #[arg(long)]
        id:        i32,
        /// Target preset ID.
        #[arg(long)]
        preset_id: i32
    },
    /// Reinstall the OS of a server (wipes data).
    Reinstall {
        /// Server ID.
        #[arg(long)]
        id:    i32,
        /// OS image ID to install.
        #[arg(long)]
        os_id: i32
    },
    /// Create a new cloud server from a preset and OS image.
    Create {
        /// Server name (max 255 chars).
        #[arg(long)]
        name:              String,
        /// Preset (tariff) ID. Use `server list-presets` to list.
        #[arg(long)]
        preset_id:         i32,
        /// OS image ID. Use `server list-os` to list.
        #[arg(long)]
        os_id:             i32,
        /// Optional comment (max 255 chars).
        #[arg(long)]
        comment:           Option<String>,
        /// SSH key IDs to attach (repeatable).
        #[arg(long = "ssh-key")]
        ssh_key:           Vec<i32>,
        /// Project ID to place the server in.
        #[arg(long)]
        project_id:        Option<i32>,
        /// Availability zone (e.g. spb-1, msk-1, ams-1).
        #[arg(long)]
        availability_zone: Option<String>
    },
    /// Update a server's name and/or comment.
    Set {
        /// Server ID.
        #[arg(long)]
        id:      i32,
        /// New name.
        #[arg(long)]
        name:    Option<String>,
        /// New comment.
        #[arg(long)]
        comment: Option<String>
    },
    /// List disk backups of a server.
    BackupList {
        /// Server ID.
        #[arg(long)]
        id: i32
    },
    /// Create a disk backup of a server's system disk.
    BackupCreate {
        /// Server ID.
        #[arg(long)]
        id:      i32,
        /// Optional backup comment.
        #[arg(long)]
        comment: Option<String>
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
    /// Restart a database by ID.
    Restart {
        /// Database ID.
        #[arg(long)]
        id: i32
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

/// Configuration subcommands.
#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show the current configuration.
    Show,
    /// Set the API token (for the default profile, or a named one).
    SetToken {
        /// The Timeweb Cloud API token.
        #[arg(long)]
        token: String,

        /// Store the token under this profile name instead of the default.
        #[arg(long)]
        profile: Option<String>
    },
    /// List configured profile names.
    Profiles,
    /// Set the UI language (en or ru).
    SetLanguage {
        /// Language code.
        #[arg(value_enum)]
        language: LangArg
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
        #[arg(long)]
        token: String
    }
}

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

/// Kubernetes subcommands.
#[derive(Subcommand, Debug)]
pub enum KubernetesCommands {
    /// List all Kubernetes clusters.
    List {
        /// Maximum number of clusters to return.
        #[arg(long)]
        limit: Option<i32>,

        /// Number of clusters to skip.
        #[arg(long)]
        offset: Option<i32>
    },
    /// Show detailed info for a cluster.
    Info {
        /// Cluster ID.
        #[arg(long)]
        id: i32
    },
    /// Create a new Kubernetes cluster.
    Create {
        /// Cluster name.
        #[arg(long)]
        name: String,

        /// Kubernetes version (e.g., 1.30).
        #[arg(long)]
        type_: String
    },
    /// Delete a cluster by ID.
    Delete {
        /// Cluster ID.
        #[arg(long)]
        id: i32
    },
    /// Update cluster settings.
    Update {
        /// Cluster ID.
        #[arg(long)]
        id: i32,

        /// New cluster name.
        #[arg(long)]
        name: Option<String>
    },
    /// List node groups for a cluster.
    NodegroupList {
        /// Cluster ID.
        #[arg(long)]
        id: i32
    },
    /// Create a node group for a cluster.
    NodegroupCreate {
        /// Cluster ID.
        #[arg(long)]
        id: i32,

        /// Node group name.
        #[arg(long)]
        name: String
    },
    /// Delete a node group from a cluster.
    NodegroupDelete {
        /// Cluster ID.
        #[arg(long)]
        id: i32,

        /// Node group ID.
        #[arg(long)]
        group_id: i32
    },
    /// List nodes for a cluster.
    NodeList {
        /// Cluster ID.
        #[arg(long)]
        id: i32
    },
    /// List installed addons for a cluster.
    AddonList {
        /// Cluster ID.
        #[arg(long)]
        id: i32
    },
    /// Install an addon on a cluster.
    AddonInstall {
        /// Cluster ID.
        #[arg(long)]
        id: i32,

        /// Addon name (e.g., calico, metrics-server).
        #[arg(long)]
        addon_name: String
    },
    /// Delete an addon from a cluster.
    AddonDelete {
        /// Cluster ID.
        #[arg(long)]
        id: i32,

        /// Addon name to delete.
        #[arg(long)]
        addon_name: String
    },
    /// List available Kubernetes presets.
    PresetList,

    /// List available Kubernetes versions.
    VersionList,

    /// List available Kubernetes network drivers.
    NetworkDrivers,

    /// Get kubeconfig for a cluster.
    Kubeconfig {
        /// Cluster ID.
        #[arg(long)]
        id: i32
    },

    /// Show cluster resources (deprecated).
    #[command(hide = true)]
    Resources {
        /// Cluster ID.
        #[arg(long)]
        id: i32
    }
}

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

/// Balancer subcommands.
#[derive(Subcommand, Debug)]
pub enum BalancerCommands {
    /// List all balancers.
    List {
        /// Maximum number of balancers to return.
        #[arg(long)]
        limit: Option<i32>,

        /// Number of balancers to skip.
        #[arg(long)]
        offset: Option<i32>
    },
    /// Show detailed info for a balancer.
    Info {
        /// Balancer ID.
        #[arg(long)]
        id: i32
    },
    /// Create a new balancer.
    Create {
        /// Balancer name.
        #[arg(long)]
        name: String
    },
    /// Delete a balancer by ID.
    Delete {
        /// Balancer ID.
        #[arg(long)]
        id: i32
    },
    /// Update balancer settings.
    Update {
        /// Balancer ID.
        #[arg(long)]
        id: i32,

        /// New balancer name.
        #[arg(long)]
        name: Option<String>
    },
    /// List rules for a balancer.
    RuleList {
        /// Balancer ID.
        #[arg(long)]
        id: i32
    },
    /// Create a rule for a balancer.
    RuleCreate {
        /// Balancer ID.
        #[arg(long)]
        id: i32
    },
    /// Delete a rule from a balancer.
    RuleDelete {
        /// Balancer ID.
        #[arg(long)]
        id: i32,

        /// Rule ID to delete.
        #[arg(long)]
        rule_id: i32
    },
    /// List IPs for a balancer.
    IpList {
        /// Balancer ID.
        #[arg(long)]
        id: i32
    },
    /// Add an IP to a balancer.
    IpAdd {
        /// Balancer ID.
        #[arg(long)]
        id: i32,

        /// IP address to add.
        #[arg(long)]
        ip: String
    },
    /// Remove an IP from a balancer.
    IpRemove {
        /// Balancer ID.
        #[arg(long)]
        id: i32,

        /// IP address to remove.
        #[arg(long)]
        ip: String
    },
    /// List available balancer presets.
    PresetList
}

/// Domain management subcommands.
#[derive(Subcommand, Debug)]
pub enum DomainCommands {
    /// List all domains on the account.
    List {
        /// Maximum number of domains to return.
        #[arg(long)]
        limit: Option<i32>,

        /// Number of domains to skip.
        #[arg(long)]
        offset: Option<i32>
    },
    /// Show detailed info for a domain.
    Info {
        /// Domain FQDN (e.g., example.com).
        #[arg(long)]
        id: String
    },
    /// Check if a domain is available for registration.
    Check {
        /// Domain name to check (e.g., example.com).
        #[arg(long)]
        domain: String
    },
    /// Add a domain to the account.
    Add {
        /// Domain FQDN to add (e.g., example.com).
        #[arg(long)]
        domain: String
    },
    /// Delete a domain from the account.
    Delete {
        /// Domain FQDN to delete (e.g., example.com).
        #[arg(long)]
        id: String
    },
    /// List DNS records for a domain.
    DnsList {
        /// Domain FQDN (e.g., example.com).
        #[arg(long)]
        id: String
    },
    /// Add a DNS record to a domain.
    DnsAdd {
        /// Domain FQDN (e.g., example.com).
        #[arg(long)]
        id: String,

        /// DNS record type (A, AAAA, CNAME, MX, TXT, SRV).
        #[arg(long)]
        record_type: String,

        /// DNS record value (e.g., IP address for A record).
        #[arg(long)]
        value: String
    },
    /// Delete a DNS record from a domain.
    DnsDelete {
        /// Domain FQDN (e.g., example.com).
        #[arg(long)]
        id: String,

        /// DNS record ID to delete.
        #[arg(long)]
        record_id: i32
    },
    /// Update a DNS record on a domain.
    DnsUpdate {
        /// Domain FQDN (e.g., example.com).
        #[arg(long)]
        id: String,

        /// DNS record ID to update.
        #[arg(long)]
        record_id: i32,

        /// New DNS record type (A, AAAA, CNAME, MX, TXT, SRV).
        #[arg(long)]
        record_type: String,

        /// New DNS record value.
        #[arg(long)]
        value: String
    },
    /// List name servers for a domain.
    NsList {
        /// Domain FQDN (e.g., example.com).
        #[arg(long)]
        id: String
    },
    /// Update name servers for a domain.
    NsUpdate {
        /// Domain FQDN (e.g., example.com).
        #[arg(long)]
        id: String,

        /// First name server (e.g., ns1.example.com).
        #[arg(short = '1', long)]
        ns1: String,

        /// Second name server (e.g., ns2.example.com).
        #[arg(short = '2', long)]
        ns2: String
    },
    /// List subdomains for a domain.
    SubdomainList {
        /// Domain FQDN (e.g., example.com).
        #[arg(long)]
        id: String
    },
    /// Add a subdomain to a domain.
    SubdomainAdd {
        /// Domain FQDN (e.g., example.com).
        #[arg(long)]
        id: String,

        /// Subdomain name (e.g., www).
        #[arg(long)]
        name: String
    },
    /// Delete a subdomain from a domain.
    SubdomainDelete {
        /// Domain FQDN (e.g., example.com).
        #[arg(long)]
        id: String,

        /// Subdomain name to delete (e.g., www).
        #[arg(long)]
        name: String
    },
    /// List domain registration/transfer/prolongation requests.
    RequestList,
    /// List available TLDs (top-level domains).
    TldList,
    /// Toggle auto-prolongation for a domain.
    AutoProlong {
        /// Domain FQDN (e.g., example.com).
        #[arg(long)]
        id: String,

        /// Enable (true) or disable (false) auto-prolongation.
        #[arg(long)]
        enabled: bool
    }
}

/// Firewall management subcommands.
#[derive(Subcommand, Debug)]
pub enum FirewallCommands {
    /// List all firewall groups.
    List {
        /// Maximum number of groups to return.
        #[arg(long)]
        limit: Option<i32>,

        /// Number of groups to skip.
        #[arg(long)]
        offset: Option<i32>
    },
    /// Show detailed info for a firewall group.
    Info {
        /// Firewall group ID.
        #[arg(long)]
        id: String
    },
    /// Create a new firewall group.
    Create {
        /// Group name.
        #[arg(long)]
        name: String
    },
    /// Delete a firewall group by ID.
    Delete {
        /// Firewall group ID.
        #[arg(long)]
        id: String
    },
    /// Update firewall group settings.
    Update {
        /// Firewall group ID.
        #[arg(long)]
        id: String,

        /// New group name.
        #[arg(long)]
        name: Option<String>
    },
    /// List rules for a firewall group.
    RuleList {
        /// Firewall group ID.
        #[arg(long)]
        id: String
    },
    /// Create a rule for a firewall group.
    RuleCreate {
        /// Firewall group ID.
        #[arg(long)]
        id: String
    },
    /// Delete a rule from a firewall group.
    RuleDelete {
        /// Firewall group ID.
        #[arg(long)]
        id: String,

        /// Rule ID to delete.
        #[arg(long)]
        rule_id: String
    },
    /// List resources for a firewall group.
    ResourceList {
        /// Firewall group ID.
        #[arg(long)]
        id: String
    },
    /// Add a resource to a firewall group.
    ResourceAdd {
        /// Firewall group ID.
        #[arg(long)]
        id: String,

        /// Resource ID to add.
        #[arg(long)]
        resource_id: String
    },
    /// Remove a resource from a firewall group.
    ResourceRemove {
        /// Firewall group ID.
        #[arg(long)]
        id: String,

        /// Resource ID to remove.
        #[arg(long)]
        resource_id: String
    }
}

#[cfg(test)]
mod tests;

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
    Create(Box<AppCreateArgs>)
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

/// Floating IP subcommands.
#[derive(Subcommand, Debug)]
pub enum IpCommands {
    /// List all floating IPs.
    List,
    /// Show detailed info about a floating IP.
    Info {
        /// Floating IP ID.
        #[arg(long)]
        id: String
    },
    /// Create a new floating IP in an availability zone.
    Create {
        /// Availability zone (e.g. spb-1, msk-1, ams-1).
        #[arg(long)]
        availability_zone: String
    },
    /// Attach a floating IP to a resource.
    Attach {
        /// Floating IP ID.
        #[arg(long)]
        id:          String,
        /// Resource ID to bind to.
        #[arg(long)]
        resource_id: i32
    },
    /// Detach a floating IP from its resource.
    Detach {
        /// Floating IP ID.
        #[arg(long)]
        id: String
    },
    /// Update a floating IP's comment.
    Set {
        /// Floating IP ID.
        #[arg(long)]
        id:      String,
        /// New comment.
        #[arg(long)]
        comment: Option<String>
    },
    /// Delete a floating IP by ID.
    Delete {
        /// Floating IP ID.
        #[arg(long)]
        id: String
    }
}

/// VPC subcommands.
#[derive(Subcommand, Debug)]
pub enum VpcCommands {
    /// List all virtual networks.
    List,
    /// Show detailed information about a VPC.
    Info {
        /// VPC ID.
        #[arg(long)]
        id: String
    },
    /// Create a new VPC.
    Create {
        /// VPC name.
        #[arg(long)]
        name:      String,
        /// IPv4 subnet mask (e.g. 192.168.0.0/24).
        #[arg(long)]
        subnet_v4: String,
        /// Location (e.g. ru-1).
        #[arg(long)]
        location:  String
    },
    /// Update a VPC's name and/or description.
    Set {
        /// VPC ID.
        #[arg(long)]
        id:          String,
        /// New name.
        #[arg(long)]
        name:        Option<String>,
        /// New description.
        #[arg(long)]
        description: Option<String>
    },
    /// List network ports of a VPC.
    Ports {
        /// VPC ID.
        #[arg(long)]
        id: String
    },
    /// Delete a VPC by ID.
    Delete {
        /// VPC ID.
        #[arg(long)]
        id: String
    }
}

/// Account subcommands.
#[derive(Subcommand, Debug)]
pub enum AccountCommands {
    /// Show account login, company and balance.
    Show,
    /// Show account auth access restrictions (IP/country allow lists).
    Access
}
