// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

#[cfg(feature = "auth")]
mod auth;
mod cli;
mod commands;
mod config;
mod error;
mod jwt;
mod output;
#[cfg(feature = "tui")]
mod tui;

rust_i18n::i18n!("locales", fallback = "en");

use clap::Parser;
use cli::{
    AccountCommands, AppsCommands, AuthCommands, BalancerCommands, Cli, Commands, ConfigCommands,
    DatabaseCommands, DomainCommands, FirewallCommands, ImageCommands, IpCommands,
    KubernetesCommands, ProjectCommands, RegistryCommands, S3Commands, ServerCommands,
    SshCommands, VpcCommands
};
use config::AppConfig;
use error::TwcError;
use output::OutputFormat;
use rust_i18n::t;
use timeweb_rs::authenticated;

/// Resolves the API token from CLI flag, environment, or config file.
///
/// # Overview
///
/// Priority order: `--token` flag > `TWC_TOKEN` env var >
/// config file (`~/.config/twc-rs/config.toml`).
///
/// # Errors
///
/// Returns [`TwcError::TokenMissing`] when no token is found.
fn resolve_token(cli_token: Option<&str>, profile: Option<&str>) -> Result<String, TwcError> {
    if let Some(token) = cli_token {
        return Ok(token.to_string());
    }

    let app_config = AppConfig::load()?;
    if let Some(token) = app_config.token_for(profile)? {
        return Ok(token);
    }

    Err(TwcError::TokenMissing)
}

/// Resolves the API token, falling back to interactive prompt.
///
/// # Overview
///
/// Calls [`resolve_token`] first. If no token is found, shows an
/// interactive menu to get one. Saves the token to config so the
/// user is never prompted again.
///
/// # Errors
///
/// Only returns error if config file operations fail catastrophically.
fn ensure_token(cli_token: Option<&str>, profile: Option<&str>) -> Result<String, TwcError> {
    if let Ok(token) = resolve_token(cli_token, profile) {
        return Ok(token);
    }

    #[cfg(feature = "auth")]
    if profile.is_none()
        && let Ok(config_path) = AppConfig::path()
        && let Ok(token) = auth::load_token(&config_path)
    {
        return Ok(token);
    }

    prompt_and_save_token()
}

/// Shows an interactive prompt to get a token, then saves it.
fn prompt_and_save_token() -> Result<String, TwcError> {
    use colored::Colorize as _;
    use dialoguer::Select;

    println!("\n  {}\n", t!("app.no_token_configured").yellow().bold());

    #[cfg(feature = "auth")]
    let options = vec![
        "Create new API key (opens browser)",
        "I have a key — paste it",
    ];
    #[cfg(not(feature = "auth"))]
    let options = vec!["Paste token from clipboard"];

    let selection = Select::new()
        .with_prompt("How to authenticate?")
        .items(&options)
        .default(0)
        .interact()
        .map_err(|e| TwcError::Io(e.to_string()))?;

    let token = match selection {
        0 => {
            #[cfg(feature = "auth")]
            {
                prompt_browser_flow()?
            }
            #[cfg(not(feature = "auth"))]
            prompt_paste_token()?
        }
        #[cfg(feature = "auth")]
        1 => prompt_paste_token()?,
        _ => std::process::exit(0)
    };

    save_token_to_config(&token)?;
    Ok(token)
}

/// Prompts user to paste a token (reads full stdin, shows masked preview).
fn prompt_paste_token() -> Result<String, TwcError> {
    let token: String = dialoguer::Password::new()
        .with_prompt("Paste your API token")
        .allow_empty_password(false)
        .interact()
        .map_err(|e| TwcError::Io(e.to_string()))?;

    let token = token.trim().to_string();
    if token.is_empty() {
        return Err(TwcError::Api("empty token".to_string()));
    }
    let masked = mask_token(&token);
    println!("  \u{2713} Token received: {masked}");
    Ok(token)
}

/// Masks a token for safe display: first 4 + *** + last 4.
fn mask_token(token: &str) -> String {
    let len = token.len();
    if len <= 8 {
        return "*".repeat(len);
    }
    let first = &token[..4];
    let last = &token[len - 4..];
    format!("{first}***{last}")
}

/// Opens browser and runs full local HTTP server auth flow.
#[cfg(feature = "auth")]
fn prompt_browser_flow() -> Result<String, TwcError> {
    let config_path =
        AppConfig::path().unwrap_or_else(|_| std::path::PathBuf::from("config.toml"));
    auth::run_auth_flow(&config_path).map_err(|e| TwcError::Api(e.to_string()))?;
    auth::load_token(&config_path).map_err(|e| TwcError::Api(e.to_string()))
}

/// Saves the token to config file (and keyring if auth feature is on).
fn save_token_to_config(token: &str) -> Result<(), TwcError> {
    use colored::Colorize as _;

    #[cfg(feature = "auth")]
    {
        let config_path =
            AppConfig::path().unwrap_or_else(|_| std::path::PathBuf::from("config.toml"));
        if let Err(e) = auth::store::save_token(token, &config_path) {
            eprintln!("  Warning: could not save to keyring: {e}");
        }
    }

    let mut cfg = AppConfig::load()?;
    cfg.token = Some(token.to_string());
    cfg.save()?;

    let masked = mask_token(token);
    println!("\n  {} ({masked})\n", t!("app.token_saved").green().bold());
    Ok(())
}

/// Dispatches a server subcommand. Kept separate so its future is boxed at the
/// call site, keeping the main `run` stack frame small.
async fn handle_server(
    cmd: ServerCommands,
    config: &timeweb_rs::apis::configuration::Configuration,
    format: OutputFormat
) -> Result<(), TwcError> {
    match cmd {
        ServerCommands::List {
            limit,
            offset
        } => commands::servers::list(config, limit, offset, format).await,
        ServerCommands::Info {
            id
        } => commands::servers::info(config, id, format).await,
        ServerCommands::Delete {
            id
        } => commands::servers::delete(config, id).await,
        ServerCommands::Reboot {
            id
        } => commands::servers::reboot(config, id).await,
        ServerCommands::Start {
            id
        } => commands::servers::start(config, id).await,
        ServerCommands::Shutdown {
            id
        } => commands::servers::shutdown(config, id).await,
        ServerCommands::Clone {
            id
        } => commands::servers::clone(config, id).await,
        ServerCommands::ResetPassword {
            id
        } => commands::servers::reset_password(config, id).await,
        ServerCommands::ListPresets => commands::servers::list_presets(config, format).await,
        ServerCommands::ListOs => commands::servers::list_os(config, format).await,
        ServerCommands::ListSoftware => commands::servers::list_software(config, format).await,
        ServerCommands::ListConfigurators => {
            commands::servers::list_configurators(config, format).await
        }
        ServerCommands::Disk {
            id
        } => commands::servers::list_disks(config, id, format).await,
        ServerCommands::Ip {
            id
        } => commands::servers::list_ips(config, id, format).await,
        ServerCommands::History {
            id
        } => commands::servers::history(config, id, format).await,
        ServerCommands::SetNatMode {
            id,
            nat_mode
        } => commands::servers::set_nat_mode(config, id, &nat_mode).await,
        ServerCommands::SetBootMode {
            id,
            boot_mode
        } => commands::servers::set_boot_mode(config, id, &boot_mode).await,
        ServerCommands::Resize {
            id,
            preset_id
        } => commands::servers::resize(config, id, preset_id).await,
        ServerCommands::Reinstall {
            id,
            os_id
        } => commands::servers::reinstall(config, id, os_id).await,
        ServerCommands::Create {
            name,
            preset_id,
            os_id,
            comment,
            ssh_key,
            project_id,
            availability_zone
        } => {
            commands::servers::create(
                config,
                &name,
                preset_id,
                os_id,
                comment.as_deref(),
                &ssh_key,
                project_id,
                availability_zone.as_deref()
            )
            .await
        }
        ServerCommands::Set {
            id,
            name,
            comment
        } => commands::servers::set(config, id, name.as_deref(), comment.as_deref()).await,
        ServerCommands::BackupList {
            id
        } => commands::servers::backup_list(config, id, format).await,
        ServerCommands::BackupCreate {
            id,
            comment
        } => commands::servers::backup_create(config, id, comment.as_deref()).await
    }
}

/// Dispatches an apps subcommand (boxed at the call site to keep `run` small).
async fn handle_apps(
    cmd: AppsCommands,
    config: &timeweb_rs::apis::configuration::Configuration,
    format: OutputFormat
) -> Result<(), TwcError> {
    match cmd {
        AppsCommands::List => commands::apps::list(config, format).await,
        AppsCommands::Info {
            id
        } => commands::apps::info(config, &id, format).await,
        AppsCommands::Delete {
            id
        } => commands::apps::delete(config, &id).await,
        AppsCommands::ListPresets => commands::apps::list_presets(config, format).await,
        AppsCommands::ListVcsProviders => commands::apps::list_vcs_providers(config, format).await,
        AppsCommands::ListRepositories {
            provider_id
        } => commands::apps::list_repositories(config, &provider_id, format).await,
        AppsCommands::Create(args) => {
            commands::apps::create(
                config,
                &args.name,
                args.comment.as_deref(),
                &args.provider_id,
                &args.repository_id,
                args.preset_id,
                &args.app_type,
                &args.framework,
                &args.branch,
                args.commit_sha.as_deref(),
                args.build_cmd.as_deref(),
                args.run_cmd.as_deref(),
                args.index_dir.as_deref(),
                args.auto_deploy,
                args.project_id,
                format
            )
            .await
        }
    }
}

/// Dispatches an image subcommand (boxed at the call site to keep `run` small).
async fn handle_image(
    cmd: ImageCommands,
    config: &timeweb_rs::apis::configuration::Configuration,
    format: OutputFormat
) -> Result<(), TwcError> {
    match cmd {
        ImageCommands::List => commands::images::list(config, format).await,
        ImageCommands::Info {
            id
        } => commands::images::info(config, &id, format).await,
        ImageCommands::Create {
            name,
            location
        } => commands::images::create(config, &name, &location, format).await,
        ImageCommands::Set {
            id,
            name
        } => commands::images::set(config, &id, name.as_deref()).await,
        ImageCommands::Delete {
            id
        } => commands::images::delete(config, &id).await,
        ImageCommands::Upload {
            id,
            file
        } => commands::images::upload(config, &id, &file).await
    }
}

/// Dispatches a floating IP subcommand (boxed to keep `run` small).
async fn handle_ip(
    cmd: IpCommands,
    config: &timeweb_rs::apis::configuration::Configuration,
    format: OutputFormat
) -> Result<(), TwcError> {
    match cmd {
        IpCommands::List => commands::floating_ips::list(config, format).await,
        IpCommands::Info {
            id
        } => commands::floating_ips::info(config, &id, format).await,
        IpCommands::Create {
            availability_zone
        } => commands::floating_ips::create(config, &availability_zone, format).await,
        IpCommands::Attach {
            id,
            resource_id
        } => commands::floating_ips::attach(config, &id, resource_id).await,
        IpCommands::Detach {
            id
        } => commands::floating_ips::detach(config, &id).await,
        IpCommands::Set {
            id,
            comment
        } => commands::floating_ips::set(config, &id, comment.as_deref()).await,
        IpCommands::Delete {
            id
        } => commands::floating_ips::delete(config, &id).await
    }
}

/// Dispatches a VPC subcommand (boxed to keep `run` small).
async fn handle_vpc(
    cmd: VpcCommands,
    config: &timeweb_rs::apis::configuration::Configuration,
    format: OutputFormat
) -> Result<(), TwcError> {
    match cmd {
        VpcCommands::List => commands::vpc::list(config, format).await,
        VpcCommands::Info {
            id
        } => commands::vpc::info(config, &id, format).await,
        VpcCommands::Create {
            name,
            subnet_v4,
            location
        } => commands::vpc::create(config, &name, &subnet_v4, &location, format).await,
        VpcCommands::Set {
            id,
            name,
            description
        } => commands::vpc::set(config, &id, name.as_deref(), description.as_deref()).await,
        VpcCommands::Ports {
            id
        } => commands::vpc::list_ports(config, &id, format).await,
        VpcCommands::Delete {
            id
        } => commands::vpc::delete(config, &id).await
    }
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

#[expect(clippy::too_many_lines)]
async fn run() -> Result<(), TwcError> {
    let cli = Cli::parse();
    let language = AppConfig::load().map(|c| c.language).unwrap_or_default();
    rust_i18n::set_locale(language.locale());
    let format = OutputFormat::parse(&cli.format).map_err(TwcError::Api)?;

    match cli.command {
        Commands::Completions {
            shell
        } => {
            use clap::CommandFactory;
            use clap_complete::{Shell, generate};

            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            let out = &mut std::io::stdout();
            match shell {
                cli::ShellArg::Bash => generate(Shell::Bash, &mut cmd, name, out),
                cli::ShellArg::Zsh => generate(Shell::Zsh, &mut cmd, name, out),
                cli::ShellArg::Fish => generate(Shell::Fish, &mut cmd, name, out),
                cli::ShellArg::Powershell => generate(Shell::PowerShell, &mut cmd, name, out),
                cli::ShellArg::Elvish => generate(Shell::Elvish, &mut cmd, name, out),
                cli::ShellArg::Nushell => {
                    generate(clap_complete_nushell::Nushell, &mut cmd, name, out);
                }
            }
            Ok(())
        }
        Commands::Config(cmd) => match cmd {
            ConfigCommands::Show => {
                let cfg = AppConfig::load()?;
                let has_token = cfg.token.is_some();
                println!(
                    "Config path: {}",
                    AppConfig::path()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default()
                );
                println!("Token set:   {has_token}");
                Ok(())
            }
            ConfigCommands::SetToken {
                token,
                profile
            } => {
                let mut cfg = AppConfig::load()?;
                if let Some(name) = profile {
                    cfg.profiles.insert(name.clone(), token);
                    cfg.save()?;
                    println!("Token saved for profile '{name}'.");
                } else {
                    cfg.token = Some(token);
                    cfg.save()?;
                    println!("Token saved.");
                }
                Ok(())
            }
            ConfigCommands::Profiles => {
                let cfg = AppConfig::load()?;
                if cfg.profiles.is_empty() {
                    println!("No profiles configured.");
                } else {
                    let mut names: Vec<&String> = cfg.profiles.keys().collect();
                    names.sort();
                    for name in names {
                        println!("{name}");
                    }
                }
                Ok(())
            }
            ConfigCommands::SetLanguage {
                language
            } => {
                let language = match language {
                    cli::LangArg::En => config::Language::En,
                    cli::LangArg::Ru => config::Language::Ru
                };
                let mut cfg = AppConfig::load()?;
                cfg.language = language;
                cfg.save()?;
                rust_i18n::set_locale(language.locale());
                println!("{}", t!("app.language_saved"));
                Ok(())
            }
        },
        Commands::Server(cmd) => {
            let token = ensure_token(cli.token.as_deref(), cli.profile.as_deref())?;
            let config = authenticated(token);
            Box::pin(handle_server(cmd, &config, format)).await
        }
        Commands::Ssh(cmd) => {
            let token = ensure_token(cli.token.as_deref(), cli.profile.as_deref())?;
            let config = authenticated(token);
            match cmd {
                SshCommands::List => commands::ssh_keys::list(&config, format).await,
                SshCommands::Add {
                    name,
                    file,
                    default
                } => commands::ssh_keys::add(&config, &name, file.as_deref(), default).await,
                SshCommands::Delete {
                    id
                } => commands::ssh_keys::delete(&config, id).await,
                SshCommands::Info {
                    id
                } => commands::ssh_keys::info(&config, id, format).await,
                SshCommands::Edit {
                    id,
                    name,
                    default
                } => commands::ssh_keys::edit(&config, id, name.as_deref(), default).await,
                SshCommands::Attach {
                    server,
                    key
                } => commands::ssh_keys::attach(&config, server, &key).await,
                SshCommands::Detach {
                    server,
                    key
                } => commands::ssh_keys::detach(&config, server, key).await
            }
        }
        Commands::Project(cmd) => {
            let token = ensure_token(cli.token.as_deref(), cli.profile.as_deref())?;
            let config = authenticated(token);
            match cmd {
                ProjectCommands::List => commands::projects::list(&config, format).await,
                ProjectCommands::Create {
                    name,
                    description
                } => commands::projects::create(&config, &name, description.as_deref()).await,
                ProjectCommands::Delete {
                    id
                } => commands::projects::delete(&config, id).await,
                ProjectCommands::Set {
                    id,
                    name,
                    description
                } => {
                    commands::projects::set(
                        &config,
                        id,
                        name.as_deref(),
                        description.as_deref(),
                        format
                    )
                    .await
                }
                ProjectCommands::Resources {
                    id
                } => commands::projects::resources(&config, id, format).await
            }
        }
        Commands::Database(cmd) => {
            let token = ensure_token(cli.token.as_deref(), cli.profile.as_deref())?;
            let config = authenticated(token);
            match cmd {
                DatabaseCommands::List {
                    limit,
                    offset
                } => commands::databases::list(&config, limit, offset, format).await,
                DatabaseCommands::Info {
                    id
                } => commands::databases::info(&config, id, format).await,
                DatabaseCommands::Create {
                    name,
                    type_,
                    preset_id
                } => commands::databases::create(&config, &name, &type_, preset_id, format).await,
                DatabaseCommands::Delete {
                    id
                } => commands::databases::delete(&config, id).await,
                DatabaseCommands::Update {
                    id,
                    name
                } => commands::databases::update(&config, id, name.as_deref(), format).await,
                DatabaseCommands::BackupList {
                    id
                } => commands::databases::backup_list(&config, id, format).await,
                DatabaseCommands::BackupCreate {
                    id
                } => commands::databases::backup_create(&config, id).await,
                DatabaseCommands::UserList {
                    id
                } => commands::databases::user_list(&config, id, format).await,
                DatabaseCommands::UserCreate {
                    db_id,
                    login,
                    password
                } => {
                    commands::databases::user_create(&config, db_id, &login, &password, format)
                        .await
                }
                DatabaseCommands::UserDelete {
                    db_id,
                    user_name
                } => commands::databases::user_delete(&config, db_id, &user_name).await,
                DatabaseCommands::PresetList => {
                    commands::databases::preset_list(&config, format).await
                }
                DatabaseCommands::ListTypes => {
                    commands::databases::list_types(&config, format).await
                }
                DatabaseCommands::ListInstances {
                    id
                } => commands::databases::list_instances(&config, id, format).await
            }
        }
        Commands::S3(cmd) => {
            let token = ensure_token(cli.token.as_deref(), cli.profile.as_deref())?;
            let config = authenticated(token);
            match cmd {
                S3Commands::List {
                    limit,
                    offset
                } => commands::s3::list(&config, limit, offset, format).await,
                S3Commands::Info {
                    id
                } => commands::s3::info(&config, id, format).await,
                S3Commands::Create {
                    name,
                    preset_id
                } => commands::s3::create(&config, &name, preset_id, format).await,
                S3Commands::Delete {
                    id
                } => commands::s3::delete(&config, id).await,
                S3Commands::Update {
                    id,
                    description
                } => commands::s3::update(&config, id, description.as_deref(), format).await,
                S3Commands::UserList {
                    id
                } => commands::s3::user_list(&config, id, format).await,
                S3Commands::UserUpdate {
                    user_id
                } => commands::s3::user_update(&config, user_id, format).await,
                S3Commands::Transfer {
                    target_id
                } => commands::s3::transfer(&config, target_id).await,
                S3Commands::SubdomainList {
                    id
                } => commands::s3::subdomain_list(&config, id, format).await,
                S3Commands::SubdomainAdd {
                    id,
                    subdomain
                } => commands::s3::subdomain_add(&config, id, &subdomain).await,
                S3Commands::SubdomainDelete {
                    id,
                    subdomain
                } => commands::s3::subdomain_delete(&config, id, &subdomain).await,
                S3Commands::PresetList => commands::s3::preset_list(&config, format).await,
                S3Commands::Genconfig {
                    id
                } => commands::s3::genconfig(&config, id).await
            }
        }
        Commands::Kubernetes(cmd) => {
            let token = ensure_token(cli.token.as_deref(), cli.profile.as_deref())?;
            let config = authenticated(token);
            match cmd {
                KubernetesCommands::List {
                    limit,
                    offset
                } => commands::kubernetes::list(&config, limit, offset, format).await,
                KubernetesCommands::Info {
                    id
                } => commands::kubernetes::info(&config, id, format).await,
                KubernetesCommands::Create {
                    name,
                    type_
                } => commands::kubernetes::create(&config, &name, &type_, format).await,
                KubernetesCommands::Delete {
                    id
                } => commands::kubernetes::delete(&config, id).await,
                KubernetesCommands::Update {
                    id,
                    name
                } => commands::kubernetes::update(&config, id, name.as_deref(), format).await,
                KubernetesCommands::NodegroupList {
                    id
                } => commands::kubernetes::nodegroup_list(&config, id, format).await,
                KubernetesCommands::NodegroupCreate {
                    id,
                    name
                } => commands::kubernetes::nodegroup_create(&config, id, &name, format).await,
                KubernetesCommands::NodegroupDelete {
                    id,
                    group_id
                } => commands::kubernetes::nodegroup_delete(&config, id, group_id).await,
                KubernetesCommands::NodeList {
                    id
                } => commands::kubernetes::node_list(&config, id, format).await,
                KubernetesCommands::AddonList {
                    id
                } => commands::kubernetes::addon_list(&config, id, format).await,
                KubernetesCommands::AddonInstall {
                    id,
                    addon_name
                } => commands::kubernetes::addon_install(&config, id, &addon_name).await,
                KubernetesCommands::AddonDelete {
                    id,
                    addon_name
                } => commands::kubernetes::addon_delete(&config, id, &addon_name).await,
                KubernetesCommands::PresetList => {
                    commands::kubernetes::preset_list(&config, format).await
                }
                KubernetesCommands::VersionList => {
                    commands::kubernetes::version_list(&config, format).await
                }
                KubernetesCommands::NetworkDrivers => {
                    commands::kubernetes::list_network_drivers(&config, format).await
                }
                KubernetesCommands::Kubeconfig {
                    id
                } => commands::kubernetes::kubeconfig(&config, id).await,
                KubernetesCommands::Resources {
                    id
                } => commands::kubernetes::resources(&config, id, format).await
            }
        }
        Commands::Registry(cmd) => {
            let token = ensure_token(cli.token.as_deref(), cli.profile.as_deref())?;
            let config = authenticated(token);
            match cmd {
                RegistryCommands::List {
                    limit,
                    offset
                } => commands::registry::list(&config, limit, offset, format).await,
                RegistryCommands::Info {
                    id
                } => commands::registry::info(&config, id, format).await,
                RegistryCommands::Create {
                    name
                } => commands::registry::create(&config, &name, format).await,
                RegistryCommands::Delete {
                    id
                } => commands::registry::delete(&config, id).await,
                RegistryCommands::Update {
                    id,
                    description
                } => commands::registry::update(&config, id, description.as_deref(), format).await,
                RegistryCommands::RepoList {
                    id
                } => commands::registry::repo_list(&config, id, format).await,
                RegistryCommands::PresetList => {
                    commands::registry::preset_list(&config, format).await
                }
            }
        }
        Commands::Balancer(cmd) => {
            let token = ensure_token(cli.token.as_deref(), cli.profile.as_deref())?;
            let config = authenticated(token);
            match cmd {
                BalancerCommands::List {
                    limit,
                    offset
                } => commands::balancers::list(&config, limit, offset, format).await,
                BalancerCommands::Info {
                    id
                } => commands::balancers::info(&config, id, format).await,
                BalancerCommands::Create {
                    name
                } => commands::balancers::create(&config, &name, format).await,
                BalancerCommands::Delete {
                    id
                } => commands::balancers::delete(&config, id).await,
                BalancerCommands::Update {
                    id,
                    name
                } => commands::balancers::update(&config, id, name.as_deref(), format).await,
                BalancerCommands::RuleList {
                    id
                } => commands::balancers::rule_list(&config, id, format).await,
                BalancerCommands::RuleCreate {
                    id
                } => commands::balancers::rule_create(&config, id, format).await,
                BalancerCommands::RuleDelete {
                    id,
                    rule_id
                } => commands::balancers::rule_delete(&config, id, rule_id).await,
                BalancerCommands::IpList {
                    id
                } => commands::balancers::ip_list(&config, id, format).await,
                BalancerCommands::IpAdd {
                    id,
                    ip
                } => commands::balancers::ip_add(&config, id, &ip).await,
                BalancerCommands::IpRemove {
                    id,
                    ip
                } => commands::balancers::ip_remove(&config, id, &ip).await,
                BalancerCommands::PresetList => {
                    commands::balancers::preset_list(&config, format).await
                }
            }
        }
        Commands::Domain(cmd) => {
            let token = ensure_token(cli.token.as_deref(), cli.profile.as_deref())?;
            let config = authenticated(token);
            match cmd {
                DomainCommands::List {
                    limit,
                    offset
                } => commands::domains::list(&config, limit, offset, format).await,
                DomainCommands::Info {
                    id
                } => commands::domains::info(&config, id, format).await,
                DomainCommands::Check {
                    domain
                } => commands::domains::check(&config, domain).await,
                DomainCommands::Add {
                    domain
                } => commands::domains::add(&config, domain, format).await,
                DomainCommands::Delete {
                    id
                } => commands::domains::delete(&config, id).await,
                DomainCommands::DnsList {
                    id
                } => commands::domains::dns_list(&config, id, format).await,
                DomainCommands::DnsAdd {
                    id,
                    record_type,
                    value
                } => commands::domains::dns_add(&config, id, record_type, value, format).await,
                DomainCommands::DnsDelete {
                    id,
                    record_id
                } => commands::domains::dns_delete(&config, id, record_id).await,
                DomainCommands::DnsUpdate {
                    id,
                    record_id,
                    record_type,
                    value
                } => {
                    commands::domains::dns_update(
                        &config,
                        id,
                        record_id,
                        record_type,
                        value,
                        format
                    )
                    .await
                }
                DomainCommands::NsList {
                    id
                } => commands::domains::ns_list(&config, id, format).await,
                DomainCommands::NsUpdate {
                    id,
                    ns1,
                    ns2
                } => commands::domains::ns_update(&config, id, ns1, ns2, format).await,
                DomainCommands::SubdomainList {
                    id
                } => commands::domains::subdomain_list(&config, id, format).await,
                DomainCommands::SubdomainAdd {
                    id,
                    name
                } => commands::domains::subdomain_add(&config, id, name, format).await,
                DomainCommands::SubdomainDelete {
                    id,
                    name
                } => commands::domains::subdomain_delete(&config, id, name).await,
                DomainCommands::RequestList => {
                    commands::domains::request_list(&config, format).await
                }
                DomainCommands::TldList => commands::domains::tld_list(&config, format).await,
                DomainCommands::AutoProlong {
                    id,
                    enabled
                } => commands::domains::auto_prolong(&config, id, enabled, format).await
            }
        }
        Commands::Firewall(cmd) => {
            let token = ensure_token(cli.token.as_deref(), cli.profile.as_deref())?;
            let config = authenticated(token);
            match cmd {
                FirewallCommands::List {
                    limit,
                    offset
                } => commands::firewall::list(&config, limit, offset, format).await,
                FirewallCommands::Info {
                    id
                } => commands::firewall::info(&config, &id, format).await,
                FirewallCommands::Create {
                    name
                } => commands::firewall::create(&config, &name, format).await,
                FirewallCommands::Delete {
                    id
                } => commands::firewall::delete(&config, &id).await,
                FirewallCommands::Update {
                    id,
                    name
                } => commands::firewall::update(&config, &id, name.as_deref(), format).await,
                FirewallCommands::RuleList {
                    id
                } => commands::firewall::rule_list(&config, &id, format).await,
                FirewallCommands::RuleCreate {
                    id
                } => commands::firewall::rule_create(&config, &id, format).await,
                FirewallCommands::RuleDelete {
                    id,
                    rule_id
                } => commands::firewall::rule_delete(&config, &id, &rule_id).await,
                FirewallCommands::ResourceList {
                    id
                } => commands::firewall::resource_list(&config, &id, format).await,
                FirewallCommands::ResourceAdd {
                    id,
                    resource_id
                } => commands::firewall::resource_add(&config, &id, &resource_id).await,
                FirewallCommands::ResourceRemove {
                    id,
                    resource_id
                } => commands::firewall::resource_remove(&config, &id, &resource_id).await
            }
        }
        Commands::Apps(cmd) => {
            let token = ensure_token(cli.token.as_deref(), cli.profile.as_deref())?;
            let config = authenticated(token);
            Box::pin(handle_apps(cmd, &config, format)).await
        }
        Commands::Image(cmd) => {
            let token = ensure_token(cli.token.as_deref(), cli.profile.as_deref())?;
            let config = authenticated(token);
            Box::pin(handle_image(cmd, &config, format)).await
        }
        Commands::Ip(cmd) => {
            let token = ensure_token(cli.token.as_deref(), cli.profile.as_deref())?;
            let config = authenticated(token);
            Box::pin(handle_ip(cmd, &config, format)).await
        }
        Commands::Vpc(cmd) => {
            let token = ensure_token(cli.token.as_deref(), cli.profile.as_deref())?;
            let config = authenticated(token);
            Box::pin(handle_vpc(cmd, &config, format)).await
        }
        Commands::Account(cmd) => {
            let token = ensure_token(cli.token.as_deref(), cli.profile.as_deref())?;
            let config = authenticated(token);
            match cmd {
                AccountCommands::Show => commands::account::show(&config, format).await,
                AccountCommands::Access => commands::account::access(&config, format).await
            }
        }
        #[cfg(feature = "auth")]
        Commands::Auth(cmd) => {
            let config_path = config::AppConfig::path()?;
            match cmd {
                AuthCommands::Flow => {
                    auth::run_auth_flow(&config_path).map_err(|e| TwcError::Api(e.to_string()))
                }
                AuthCommands::Status => {
                    auth::show_status(&config_path).map_err(|e| TwcError::Api(e.to_string()))
                }
                AuthCommands::Logout => {
                    auth::logout(&config_path).map_err(|e| TwcError::Api(e.to_string()))
                }
                AuthCommands::Token {
                    token
                } => auth::accept_token_direct(&token, &config_path)
                    .map_err(|e| TwcError::Api(e.to_string()))
            }
        }
        #[cfg(not(feature = "auth"))]
        Commands::Auth(_) => {
            eprintln!(
                "Error: auth feature not enabled. \
                 Rebuild with --features auth"
            );
            std::process::exit(1);
        }
        #[cfg(feature = "tui")]
        Commands::Dashboard {
            interval
        } => {
            let config = AppConfig::load()?;
            let token = ensure_token(cli.token.as_deref(), cli.profile.as_deref())?;
            let theme = config.theme;
            let prefs = config.dashboard.clone();
            Box::pin(run_dashboard(
                token,
                interval,
                theme,
                prefs,
                cli.profile.clone()
            ))
            .await
        }
        #[cfg(not(feature = "tui"))]
        Commands::Dashboard {
            ..
        } => {
            eprintln!(
                "Error: tui feature not enabled. \
                 Rebuild with --features tui"
            );
            std::process::exit(1);
        }
    }
}

#[cfg(feature = "tui")]
fn persist_dashboard_prefs(app: &tui::app::App) {
    let Ok(mut cfg) = AppConfig::load() else {
        return;
    };
    cfg.theme = app.theme;
    cfg.language = app.language;
    cfg.dashboard.hidden_widgets = app.hidden_widget_ids();
    cfg.dashboard.list_width_pct = app.list_width_pct;
    cfg.dashboard.hide_empty_tabs = app.hide_empty_tabs;
    let _ = cfg.save();
}

#[cfg(feature = "tui")]
#[expect(clippy::large_futures)]
async fn fetch_data(
    token: String,
    interval: u64,
    theme: crate::tui::themes::Theme
) -> tui::app::DashboardData {
    let config = authenticated(token);
    let mut tmp = tui::app::App::new_with_theme(interval, theme, None);
    refresh_all(&config, &mut tmp).await;
    tui::app::DashboardData::from_app(&tmp)
}

#[cfg(feature = "tui")]
fn spawn_refresh_loop(
    tx: tokio::sync::mpsc::UnboundedSender<tui::event::AppEvent>,
    token: String,
    theme: crate::tui::themes::Theme,
    interval: u64
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let period = tokio::time::Duration::from_secs(interval.max(2));
        loop {
            let data = Box::pin(fetch_data(token.clone(), interval, theme)).await;
            if tx.send(tui::event::AppEvent::Data(Box::new(data))).is_err() {
                break;
            }
            tokio::time::sleep(period).await;
        }
    })
}

#[cfg(feature = "tui")]
fn spawn_one_shot_refresh(
    tx: tokio::sync::mpsc::UnboundedSender<tui::event::AppEvent>,
    token: String,
    theme: crate::tui::themes::Theme,
    interval: u64
) {
    tokio::spawn(async move {
        let data = Box::pin(fetch_data(token, interval, theme)).await;
        let _ = tx.send(tui::event::AppEvent::Data(Box::new(data)));
    });
}

#[cfg(feature = "tui")]
async fn run_dashboard(
    mut token: String,
    interval: u64,
    theme: crate::tui::themes::Theme,
    prefs: crate::config::DashboardPrefs,
    profile: Option<String>
) -> Result<(), TwcError> {
    use crossterm::{
        execute,
        terminal::{
            EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode
        }
    };
    use ratatui::{Terminal, backend::CrosstermBackend};
    use tokio::sync::mpsc;

    enable_raw_mode().map_err(|e| TwcError::Io(e.to_string()))?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| TwcError::Io(e.to_string()))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| TwcError::Io(e.to_string()))?;

    let mut app = tui::app::App::new_with_theme(interval, theme, Some(token.clone()));
    app.apply_prefs(
        &prefs.hidden_widgets,
        prefs.list_width_pct,
        prefs.hide_empty_tabs
    );
    if let Ok(cfg) = AppConfig::load() {
        app.language = cfg.language;
        let mut names = vec!["default".to_string()];
        let mut profile_names: Vec<String> = cfg.profiles.keys().cloned().collect();
        profile_names.sort();
        names.extend(profile_names);
        app.profiles = names;
    }
    app.active_profile = profile.unwrap_or_else(|| "default".to_string());
    app.is_loading = true;
    draw_splash(&mut terminal);

    let (tx, mut rx) = mpsc::unbounded_channel();
    let event_tx = tx.clone();

    tokio::spawn(async move {
        tui::event::run_event_loop(event_tx).await;
    });

    let mut refresh_handle = spawn_refresh_loop(tx.clone(), token.clone(), theme, interval);

    while let Some(event) = rx.recv().await {
        if !tui::event::handle_event(&mut app, event) {
            break;
        }

        terminal
            .draw(|f| tui::ui::draw(f, &app))
            .map_err(|e| TwcError::Io(e.to_string()))?;

        if app.prefs_dirty {
            persist_dashboard_prefs(&app);
            app.prefs_dirty = false;
        }

        if app.refresh_requested {
            app.refresh_requested = false;
            spawn_one_shot_refresh(tx.clone(), token.clone(), theme, interval);
        }

        if let Some((drill_tab, drill_id, drill_name)) = app.take_drill_request() {
            use tui::app::LogLevel;
            let config = authenticated(token.clone());
            match fetch_drill(&config, drill_tab, drill_id, &drill_name).await {
                Ok(view) => {
                    app.log(LogLevel::Info, format!("opened {drill_name}"));
                    app.open_drill(view);
                }
                Err(e) => {
                    app.log(LogLevel::Error, format!("open {drill_name} failed: {e}"));
                }
            }
        }

        if let Some(action) = app.take_dispatch() {
            use tui::app::LogLevel;
            app.log(
                LogLevel::Info,
                format!("{} {}", action.kind.label(), action.resource_name)
            );
            let config = authenticated(token.clone());
            perform_action(&config, &mut app, action).await;
            spawn_one_shot_refresh(tx.clone(), token.clone(), theme, interval);
        }

        if let Some(form) = app.take_create_request() {
            let config = authenticated(token.clone());
            perform_create(&config, &mut app, form).await;
            spawn_one_shot_refresh(tx.clone(), token.clone(), theme, interval);
        }

        if let Some(profile) = app.take_switch_profile() {
            use tui::app::LogLevel;
            let lookup = (profile != "default").then_some(profile.as_str());
            match AppConfig::load().and_then(|c| c.token_for(lookup)) {
                Ok(Some(new_token)) => {
                    token = new_token;
                    refresh_handle.abort();
                    refresh_handle =
                        spawn_refresh_loop(tx.clone(), token.clone(), theme, interval);
                    app.active_profile.clone_from(&profile);
                    app.is_loading = true;
                    app.log(
                        LogLevel::Success,
                        format!("switched to profile '{profile}'")
                    );
                    app.status_message = Some(format!("Profile: {profile}"));
                }
                _ => {
                    app.log(
                        LogLevel::Error,
                        format!("profile '{profile}' has no token configured")
                    );
                }
            }
        }

        if let Some(req) = app.poll_stats_request() {
            spawn_stats_fetch(tx.clone(), token.clone(), req);
        }
    }

    drop(tx);

    disable_raw_mode().map_err(|e| TwcError::Io(e.to_string()))?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .map_err(|e| TwcError::Io(e.to_string()))?;
    terminal
        .show_cursor()
        .map_err(|e| TwcError::Io(e.to_string()))?;
    Ok(())
}

/// Spawns a background task that fetches live statistics for a resource and
/// sends them back to the event loop.
#[cfg(feature = "tui")]
fn spawn_stats_fetch(
    tx: tokio::sync::mpsc::UnboundedSender<tui::event::AppEvent>,
    token: String,
    req: tui::app::StatsRequest
) {
    tokio::spawn(async move {
        let config = authenticated(token);
        match fetch_resource_stats(&config, &req).await {
            Ok(stats) => {
                if stats.cpu.is_empty()
                    && stats.ram.is_empty()
                    && stats.net_in.is_empty()
                    && stats.net_out.is_empty()
                {
                    let _ = tx.send(tui::event::AppEvent::StatsError(format!(
                        "stats {}: no data points returned",
                        req.name
                    )));
                }
                let _ = tx.send(tui::event::AppEvent::Stats(Box::new(stats)));
            }
            Err(e) => {
                let _ = tx.send(tui::event::AppEvent::StatsError(format!(
                    "stats {}: {e}",
                    req.name
                )));
            }
        }
    });
}

/// Keeps the most recent `LIVE_STATS_SAMPLES` points of a series.
#[cfg(feature = "tui")]
const fn tail_skip(len: usize) -> usize {
    const LIVE_STATS_SAMPLES: usize = 60;
    len.saturating_sub(LIVE_STATS_SAMPLES)
}

/// Fetches live statistics for the selected server (current, non-deprecated
/// endpoint: CPU and network) or app (CPU, RAM and network) and maps them to a
/// unified [`tui::app::ResourceStats`] series.
#[cfg(feature = "tui")]
async fn fetch_resource_stats(
    config: &timeweb_rs::apis::configuration::Configuration,
    req: &tui::app::StatsRequest
) -> Result<tui::app::ResourceStats, String> {
    use tui::app::ResourceTab;

    match req.tab {
        ResourceTab::Servers => fetch_server_stats(config, req).await,
        ResourceTab::Apps => fetch_app_stats(config, req).await,
        _ => Err("live statistics are not available for this resource".to_string())
    }
}

/// Formats a UTC timestamp the way the statistics endpoints expect: ISO 8601
/// without a timezone offset (`2023-05-25T14:35:38`).
#[cfg(feature = "tui")]
fn stats_timestamp(at: chrono::DateTime<chrono::Utc>) -> String {
    at.format("%Y-%m-%dT%H:%M:%S").to_string()
}

/// Fetches server statistics via the current endpoint, which reports CPU load
/// and network throughput (it does not expose live RAM usage).
#[cfg(feature = "tui")]
async fn fetch_server_stats(
    config: &timeweb_rs::apis::configuration::Configuration,
    req: &tui::app::StatsRequest
) -> Result<tui::app::ResourceStats, String> {
    let id: i32 = req
        .id
        .parse()
        .map_err(|_| format!("invalid server id {}", req.id))?;
    let now = chrono::Utc::now();
    let time_from = stats_timestamp(now - chrono::Duration::hours(24));
    let keys = "system.cpu.util;network.request;network.response";

    let resp = timeweb_rs::apis::servers_api::get_server_statistics_new(
        config, id, &time_from, "24", keys
    )
    .await
    .map_err(|e| e.to_string())?;

    let mut stats = tui::app::ResourceStats {
        id: req.id.clone(),
        subject: req.name.clone(),
        ..Default::default()
    };

    for series in resp.statistics.into_iter().flatten() {
        let Some(name) = series.name.as_deref() else {
            continue;
        };
        let mut values: Vec<f64> = series
            .list
            .unwrap_or_default()
            .into_iter()
            .map(|p| p.value)
            .collect();
        let values = values.split_off(tail_skip(values.len()));
        match name {
            "system.cpu.util" => stats.cpu = values,
            "network.request" => stats.net_in = values,
            "network.response" => stats.net_out = values,
            _ => {}
        }
    }

    Ok(stats)
}

/// Fetches app statistics, which report CPU load, RAM usage and network
/// throughput as a single time-series response.
#[cfg(feature = "tui")]
async fn fetch_app_stats(
    config: &timeweb_rs::apis::configuration::Configuration,
    req: &tui::app::StatsRequest
) -> Result<tui::app::ResourceStats, String> {
    let now = chrono::Utc::now();
    let from = stats_timestamp(now - chrono::Duration::hours(24));
    let to = stats_timestamp(now);

    let resp = timeweb_rs::apis::apps_api::get_app_statistics(config, &req.id, &from, &to)
        .await
        .map_err(|e| e.to_string())?;

    let cpu: Vec<f64> = resp
        .cpu
        .iter()
        .skip(tail_skip(resp.cpu.len()))
        .map(|c| c.load)
        .collect();
    let ram: Vec<f64> = resp
        .ram
        .iter()
        .skip(tail_skip(resp.ram.len()))
        .map(|r| {
            if r.total > 0.0 {
                (r.used / r.total) * 100.0
            } else {
                0.0
            }
        })
        .collect();
    let net = &resp.network_traffic;
    let net_in: Vec<f64> = net
        .iter()
        .skip(tail_skip(net.len()))
        .map(|n| n.incoming)
        .collect();
    let net_out: Vec<f64> = net
        .iter()
        .skip(tail_skip(net.len()))
        .map(|n| n.outgoing)
        .collect();

    Ok(tui::app::ResourceStats {
        id: req.id.clone(),
        subject: req.name.clone(),
        cpu,
        ram,
        net_in,
        net_out
    })
}

#[cfg(feature = "tui")]
async fn fetch_drill(
    config: &timeweb_rs::apis::configuration::Configuration,
    tab: tui::app::ResourceTab,
    id: i32,
    name: &str
) -> Result<tui::app::DrillView, String> {
    use tui::app::{DrillItem, DrillView, ResourceTab};

    match tab {
        ResourceTab::Projects => {
            let resp = timeweb_rs::apis::projects_api::get_all_project_resources(config, id)
                .await
                .map_err(|e| e.to_string())?;
            let mut items = Vec::new();
            for s in &resp.servers {
                items.push(DrillItem {
                    kind:   "Server".to_string(),
                    name:   s.name.clone(),
                    detail: format!("{:?}", s.status)
                });
            }
            for d in &resp.databases {
                items.push(DrillItem {
                    kind:   "Database".to_string(),
                    name:   d.name.clone(),
                    detail: d.r#type.clone()
                });
            }
            for b in &resp.buckets {
                items.push(DrillItem {
                    kind:   "S3 bucket".to_string(),
                    name:   b.name.clone(),
                    detail: String::new()
                });
            }
            for c in &resp.clusters {
                items.push(DrillItem {
                    kind:   "Kubernetes".to_string(),
                    name:   c.name.clone(),
                    detail: format!("{:?}", c.status)
                });
            }
            for b in &resp.balancers {
                items.push(DrillItem {
                    kind:   "Balancer".to_string(),
                    name:   b.name.clone(),
                    detail: format!("{:?}", b.status)
                });
            }
            for d in &resp.dedicated_servers {
                items.push(DrillItem {
                    kind:   "Dedicated".to_string(),
                    name:   d.name.clone(),
                    detail: String::new()
                });
            }
            Ok(DrillView {
                title: format!("Project '{name}'  ({} resources)", items.len()),
                items,
                selected: 0
            })
        }
        _ => Err("this resource cannot be entered".to_string())
    }
}

/// Performs an in-dashboard resource creation submitted from a create form,
/// logging the outcome. Only resources with a simple create form are handled.
#[cfg(feature = "tui")]
async fn perform_create(
    config: &timeweb_rs::apis::configuration::Configuration,
    app: &mut tui::app::App,
    form: tui::app::CreateForm
) {
    use timeweb_rs::{apis::projects_api, models::CreateProject};
    use tui::app::{LogLevel, ResourceTab};

    let field = |i: usize| form.fields.get(i).map(|f| f.value.trim().to_owned());

    let result = match form.tab {
        ResourceTab::Projects => {
            let name = field(0).unwrap_or_default();
            let mut req = CreateProject::new(name);
            if let Some(desc) = field(1).filter(|d| !d.is_empty()) {
                req.description = Some(Some(desc));
            }
            projects_api::create_project(config, req)
                .await
                .map(|r| r.project.name)
                .map_err(|e| e.to_string())
        }
        _ => Err("creation not supported for this resource".to_string())
    };

    match result {
        Ok(name) => {
            let msg = format!("created '{name}'");
            app.log(LogLevel::Success, msg.clone());
            app.status_message = Some(msg);
            app.error_message = None;
        }
        Err(e) => {
            let msg = format!("create failed: {e}");
            app.log(LogLevel::Error, msg.clone());
            app.error_message = Some(msg);
        }
    }
}

#[cfg(feature = "tui")]
async fn perform_action(
    config: &timeweb_rs::apis::configuration::Configuration,
    app: &mut tui::app::App,
    pending: tui::app::PendingAction
) {
    use timeweb_rs::apis::{
        ai_agents_api, apps_api, balancers_api, container_registry_api, databases_api,
        dedicated_servers_api, knowledge_bases_api, kubernetes_api, projects_api, s3_api,
        servers_api
    };
    use tui::app::{ActionKind, ResourceTab};

    let id = pending.resource_id;
    let result = match (pending.tab, pending.kind) {
        (ResourceTab::Servers, ActionKind::Start) => servers_api::start_server(config, id)
            .await
            .map_err(|e| e.to_string()),
        (ResourceTab::Servers, ActionKind::Shutdown) => servers_api::shutdown_server(config, id)
            .await
            .map_err(|e| e.to_string()),
        (ResourceTab::Servers, ActionKind::Reboot) => servers_api::reboot_server(config, id)
            .await
            .map_err(|e| e.to_string()),
        (ResourceTab::Servers, ActionKind::Clone) => servers_api::clone_server(config, id)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string()),
        (ResourceTab::Servers, ActionKind::Delete) => {
            servers_api::delete_server(config, id, None, None)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
        (ResourceTab::Databases, ActionKind::Delete) => {
            databases_api::delete_database_cluster(config, id, None, None)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
        (ResourceTab::S3, ActionKind::Delete) => s3_api::delete_storage(config, id, None, None)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string()),
        (ResourceTab::Kubernetes, ActionKind::Delete) => {
            kubernetes_api::delete_cluster(config, id, None, None)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
        (ResourceTab::Balancers, ActionKind::Delete) => {
            balancers_api::delete_balancer(config, id, None, None)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
        (ResourceTab::Registry, ActionKind::Delete) => {
            container_registry_api::delete_registry(config, id)
                .await
                .map_err(|e| e.to_string())
        }
        (ResourceTab::Projects, ActionKind::Delete) => projects_api::delete_project(config, id)
            .await
            .map_err(|e| e.to_string()),
        (ResourceTab::DedicatedServers, ActionKind::Delete) => {
            dedicated_servers_api::delete_dedicated_server(config, id)
                .await
                .map_err(|e| e.to_string())
        }
        (ResourceTab::AiAgents, ActionKind::Delete) => ai_agents_api::delete_agent(config, id)
            .await
            .map_err(|e| e.to_string()),
        (ResourceTab::KnowledgeBases, ActionKind::Delete) => {
            knowledge_bases_api::delete_knowledgebase(config, id)
                .await
                .map_err(|e| e.to_string())
        }
        (ResourceTab::Apps, ActionKind::Delete) => apps_api::delete_app(config, &id.to_string())
            .await
            .map_err(|e| e.to_string()),
        _ => Err("action not supported for this resource".to_string())
    };

    match result {
        Ok(()) => {
            app.error_message = None;
            let msg = format!(
                "{} '{}' (id {}) — ok",
                pending.kind.label(),
                pending.resource_name,
                pending.resource_id
            );
            app.log(tui::app::LogLevel::Success, msg.clone());
            app.status_message = Some(msg);
        }
        Err(e) => {
            let msg = format!(
                "{} '{}' failed: {e}",
                pending.kind.label(),
                pending.resource_name
            );
            app.log(tui::app::LogLevel::Error, msg.clone());
            app.error_message = Some(msg);
        }
    }
}

/// Extracts the primary public IPv4 address of a server, preferring the
/// address marked as main and falling back to the first public one.
#[cfg(feature = "tui")]
fn server_public_ip(server: &timeweb_rs::models::Vds) -> String {
    use timeweb_rs::models::vds_networks_inner::Type;

    let mut fallback = None;
    for network in &server.networks {
        if !matches!(network.r#type, Type::Public) {
            continue;
        }
        for ip in network.ips.iter().flatten() {
            if ip.is_main {
                return ip.ip.clone();
            }
            if fallback.is_none() {
                fallback = Some(ip.ip.clone());
            }
        }
    }
    fallback.unwrap_or_default()
}

/// Fetches every page of a paginated list endpoint, advancing the offset by
/// the number of collected items until `meta.total` is reached or a page
/// comes back empty.
#[cfg(feature = "tui")]
#[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
async fn fetch_all_pages<'a, T, E, F>(mut fetch_page: F) -> Result<Vec<T>, E>
where
    F: FnMut(
        i32,
        i32
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(Vec<T>, i32), E>> + Send + 'a>
    >
{
    const PAGE_LIMIT: i32 = 100;

    let mut items: Vec<T> = Vec::new();
    loop {
        let (page, total) = fetch_page(PAGE_LIMIT, items.len() as i32).await?;
        if page.is_empty() {
            return Ok(items);
        }
        items.extend(page);
        if items.len() as i32 >= total {
            return Ok(items);
        }
    }
}

/// Describes what a floating IP is bound to, resolving server names from the
/// already-fetched server list and falling back to `type #id` for other
/// resource kinds.
#[cfg(feature = "tui")]
fn floating_ip_binding(
    ip: &timeweb_rs::models::FloatingIp,
    server_names: &std::collections::HashMap<i64, String>
) -> String {
    use timeweb_rs::models::FloatingIpResourceId;

    let Some(resource_id) = ip.resource_id.as_deref() else {
        return String::new();
    };
    let id_text = match resource_id {
        FloatingIpResourceId::Number(n) => format!("{n}"),
        FloatingIpResourceId::String(s) => s.clone()
    };
    let resource_type = ip.resource_type.clone().unwrap_or_default();
    if resource_type == "server"
        && let Ok(id) = id_text.parse::<i64>()
        && let Some(name) = server_names.get(&id)
    {
        return name.clone();
    }
    if resource_type.is_empty() {
        format!("#{id_text}")
    } else {
        format!("{resource_type} #{id_text}")
    }
}

/// Sums the sizes of all disks attached to a server, converting the API's
/// megabyte values to whole gigabytes.
#[cfg(feature = "tui")]
#[expect(clippy::cast_possible_truncation)]
fn server_disk_gb(server: &timeweb_rs::models::Vds) -> i32 {
    let total_mb: f64 = server.disks.iter().map(|d| d.size).sum();
    (total_mb / 1024.0).round() as i32
}

#[cfg(feature = "tui")]
#[expect(clippy::too_many_lines)]
#[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
async fn refresh_all(
    config: &timeweb_rs::apis::configuration::Configuration,
    app: &mut tui::app::App
) {
    use tui::app::{
        AccountInfo, AiAgentSummary, AppSummary, BalancerSummary, DatabaseSummary,
        DedicatedServerSummary, DomainSummary, FirewallSummary, FloatingIpSummary, ImageSummary,
        K8sSummary, KnowledgeBaseSummary, MailSummary, NetworkDriveSummary, ProjectSummary,
        RegistrySummary, S3Summary, ServerSummary, VpcSummary
    };

    let c = config.clone();
    let cfg = &c;

    let (
        account_res,
        servers_res,
        dbs_res,
        s3_res,
        k8s_res,
        projects_res,
        balancers_res,
        registries_res,
        domains_res,
        firewalls_res,
        floating_ips_res,
        images_res,
        network_drives_res,
        vpcs_res,
        dedicated_servers_res,
        mails_res,
        apps_res,
        ai_agents_res,
        knowledge_bases_res,
        ssh_keys_res,
        finances_res
    ) = tokio::join!(
        timeweb_rs::apis::account_api::get_account_status(&c),
        fetch_all_pages(move |limit, offset| {
            Box::pin(async move {
                timeweb_rs::apis::servers_api::get_servers(cfg, Some(limit), Some(offset))
                    .await
                    .map(|r| (r.servers, r.meta.total))
            })
        }),
        fetch_all_pages(move |limit, offset| {
            Box::pin(async move {
                timeweb_rs::apis::databases_api::get_database_clusters(
                    cfg,
                    Some(limit),
                    Some(offset)
                )
                .await
                .map(|r| (r.dbs, r.meta.total))
            })
        }),
        timeweb_rs::apis::s3_api::get_storages(&c),
        fetch_all_pages(move |limit, offset| {
            Box::pin(async move {
                timeweb_rs::apis::kubernetes_api::get_clusters(cfg, Some(limit), Some(offset))
                    .await
                    .map(|r| (r.clusters, r.meta.total))
            })
        }),
        timeweb_rs::apis::projects_api::get_projects(&c),
        fetch_all_pages(move |limit, offset| {
            Box::pin(async move {
                timeweb_rs::apis::balancers_api::get_balancers(cfg, Some(limit), Some(offset))
                    .await
                    .map(|r| (r.balancers, r.meta.total))
            })
        }),
        timeweb_rs::apis::container_registry_api::get_registries(&c),
        fetch_all_pages(move |limit, offset| {
            Box::pin(async move {
                timeweb_rs::apis::domains_api::get_domains(
                    cfg,
                    Some(limit),
                    Some(offset),
                    None,
                    None,
                    None,
                    None
                )
                .await
                .map(|r| (r.domains, r.meta.total))
            })
        }),
        fetch_all_pages(move |limit, offset| {
            Box::pin(async move {
                timeweb_rs::apis::firewall_api::get_groups(cfg, Some(limit), Some(offset))
                    .await
                    .map(|r| (r.groups, r.meta.total))
            })
        }),
        timeweb_rs::apis::floating_ip_api::get_floating_ips(&c),
        fetch_all_pages(move |limit, offset| {
            Box::pin(async move {
                timeweb_rs::apis::images_api::get_images(cfg, Some(limit), Some(offset))
                    .await
                    .map(|r| (r.images, r.meta.total))
            })
        }),
        timeweb_rs::apis::network_drives_api::get_network_drives(&c),
        timeweb_rs::apis::vpc_api::get_vpcs(&c),
        timeweb_rs::apis::dedicated_servers_api::get_dedicated_servers(&c),
        fetch_all_pages(move |limit, offset| {
            Box::pin(async move {
                timeweb_rs::apis::mail_api::get_all_mailboxes_v2(
                    cfg,
                    Some(limit),
                    Some(offset),
                    None
                )
                .await
                .map(|r| (r.mailboxes, r.meta.total))
            })
        }),
        timeweb_rs::apis::apps_api::get_apps(&c),
        timeweb_rs::apis::ai_agents_api::get_agents(&c),
        timeweb_rs::apis::knowledge_bases_api::get_knowledgebases_v2(&c),
        timeweb_rs::apis::ssh_api::get_keys(&c),
        timeweb_rs::apis::payments_api::get_finances(&c)
    );

    let err_of = |e: Option<String>, name: &str| e.map(|msg| format!("{name}: {msg}"));
    app.last_load_errors = [
        err_of(
            account_res.as_ref().err().map(ToString::to_string),
            "account"
        ),
        err_of(
            servers_res.as_ref().err().map(ToString::to_string),
            "servers"
        ),
        err_of(dbs_res.as_ref().err().map(ToString::to_string), "databases"),
        err_of(s3_res.as_ref().err().map(ToString::to_string), "s3"),
        err_of(
            k8s_res.as_ref().err().map(ToString::to_string),
            "kubernetes"
        ),
        err_of(
            projects_res.as_ref().err().map(ToString::to_string),
            "projects"
        ),
        err_of(
            balancers_res.as_ref().err().map(ToString::to_string),
            "balancers"
        ),
        err_of(
            registries_res.as_ref().err().map(ToString::to_string),
            "registries"
        ),
        err_of(
            domains_res.as_ref().err().map(ToString::to_string),
            "domains"
        ),
        err_of(
            firewalls_res.as_ref().err().map(ToString::to_string),
            "firewall"
        ),
        err_of(
            floating_ips_res.as_ref().err().map(ToString::to_string),
            "floating IPs"
        ),
        err_of(images_res.as_ref().err().map(ToString::to_string), "images"),
        err_of(
            network_drives_res.as_ref().err().map(ToString::to_string),
            "network drives"
        ),
        err_of(vpcs_res.as_ref().err().map(ToString::to_string), "VPCs"),
        err_of(
            dedicated_servers_res
                .as_ref()
                .err()
                .map(ToString::to_string),
            "dedicated servers"
        ),
        err_of(mails_res.as_ref().err().map(ToString::to_string), "mail"),
        err_of(apps_res.as_ref().err().map(ToString::to_string), "apps"),
        err_of(
            ai_agents_res.as_ref().err().map(ToString::to_string),
            "AI agents"
        ),
        err_of(
            knowledge_bases_res.as_ref().err().map(ToString::to_string),
            "knowledge bases"
        ),
        err_of(
            ssh_keys_res.as_ref().err().map(ToString::to_string),
            "SSH keys"
        ),
        err_of(
            finances_res.as_ref().err().map(ToString::to_string),
            "finances"
        )
    ]
    .into_iter()
    .flatten()
    .collect();

    let mut account_id: i64 = 0;
    let mut login = String::new();
    let mut balance = String::new();
    let mut account_status = String::from("active");

    if let Ok(resp) = account_res {
        account_id = resp.status.company_info.id;
        login = resp.status.login.clone().unwrap_or_default();
        if resp.status.is_blocked || resp.status.is_permanent_blocked {
            account_status = String::from("blocked");
        }
    }
    if let Ok(ref resp) = finances_res {
        let f = &resp.finances;
        balance = format!("{:.2} {}", f.balance, f.currency);
    }
    app.update_account(AccountInfo {
        login,
        account_id,
        balance,
        status: account_status
    });

    let mut server_names: std::collections::HashMap<i64, String> =
        std::collections::HashMap::new();
    if let Ok(servers) = servers_res {
        for s in &servers {
            server_names.insert(s.id, s.name.clone());
        }
        let summaries: Vec<ServerSummary> = servers
            .iter()
            .map(|s| ServerSummary {
                id:       s.id as i32,
                name:     s.name.clone(),
                status:   format!("{:?}", s.status),
                cpu:      s.cpu as i32,
                ram_mb:   s.ram as i32,
                disk_gb:  server_disk_gb(s),
                ip:       server_public_ip(s),
                location: s.location.clone()
            })
            .collect();
        app.update_servers(summaries);
    }

    if let Ok(dbs) = dbs_res {
        let summaries: Vec<DatabaseSummary> = dbs
            .iter()
            .map(|d| DatabaseSummary {
                id:      d.id as i32,
                name:    d.name.clone(),
                status:  format!("{:?}", d.status),
                engine:  d.r#type.clone(),
                size_mb: d
                    .disk
                    .as_ref()
                    .and_then(|disk| disk.as_deref())
                    .map_or(0, |disk| (disk.size / 1024.0) as i64)
            })
            .collect();
        app.update_databases(summaries);
    }

    if let Ok(resp) = s3_res {
        let summaries: Vec<S3Summary> = resp
            .buckets
            .iter()
            .map(|b| S3Summary {
                id:           b.id as i32,
                name:         b.name.clone(),
                region:       b.location.clone(),
                size_kb:      b.disk_stats.size as i64,
                object_count: b.object_amount as i64
            })
            .collect();
        app.update_s3(summaries);
    }

    if let Ok(clusters) = k8s_res {
        let summaries: Vec<K8sSummary> = clusters
            .iter()
            .map(|c| K8sSummary {
                id:      c.id,
                name:    c.name.clone(),
                status:  c.status.clone(),
                version: c.k8s_version.clone(),
                cpu:     c.cpu.unwrap_or(0),
                ram_mb:  c.ram.unwrap_or(0),
                disk_gb: c.disk.unwrap_or(0)
            })
            .collect();
        app.update_k8s(summaries);
    }

    if let Ok(resp) = projects_res {
        let mut count_handles = Vec::with_capacity(resp.projects.len());
        for p in &resp.projects {
            let cfg = c.clone();
            let project_id = p.id as i32;
            count_handles.push(tokio::spawn(async move {
                timeweb_rs::apis::projects_api::get_all_project_resources(&cfg, project_id)
                    .await
                    .map_or(0, |r| r.servers.len() as i32)
            }));
        }
        let mut summaries = Vec::with_capacity(resp.projects.len());
        for (p, handle) in resp.projects.iter().zip(count_handles) {
            summaries.push(ProjectSummary {
                id:           p.id as i32,
                name:         p.name.clone(),
                server_count: handle.await.unwrap_or(0)
            });
        }
        app.update_projects(summaries);
    }

    if let Ok(balancers) = balancers_res {
        let summaries: Vec<BalancerSummary> = balancers
            .iter()
            .map(|b| BalancerSummary {
                id:       b.id as i32,
                name:     b.name.clone(),
                status:   format!("{:?}", b.status),
                ip:       b.ips.first().cloned().unwrap_or_default(),
                location: b.location.clone()
            })
            .collect();
        app.update_balancers(summaries);
    }

    if let Ok(resp) = registries_res
        && let Some(registries) = resp.container_registry_list
    {
        let summaries: Vec<RegistrySummary> = registries
            .iter()
            .map(|r| RegistrySummary {
                id:        r.id,
                name:      r.name.clone(),
                disk_used: i64::from(r.disk_stats.used),
                disk_size: i64::from(r.disk_stats.size)
            })
            .collect();
        app.update_registries(summaries);
    }

    if let Ok(domains) = domains_res {
        let summaries: Vec<DomainSummary> = domains
            .iter()
            .map(|d| DomainSummary {
                id:           d.id as i32,
                name:         d.fqdn.clone(),
                status:       format!("{:?}", d.domain_status),
                auto_prolong: d.is_autoprolong_enabled.unwrap_or(false)
            })
            .collect();
        app.update_domains(summaries);
    }

    if let Ok(groups) = firewalls_res {
        let summaries: Vec<FirewallSummary> = groups
            .iter()
            .map(|g| FirewallSummary {
                id:     g.id.parse::<i32>().unwrap_or(0),
                name:   g.name.clone(),
                policy: g.policy.to_string()
            })
            .collect();
        app.update_firewalls(summaries);
    }

    if let Ok(resp) = floating_ips_res {
        let summaries: Vec<FloatingIpSummary> = resp
            .ips
            .iter()
            .map(|ip| FloatingIpSummary {
                id:          ip.id.parse::<i32>().unwrap_or(0),
                ip:          ip.ip.clone().unwrap_or_default(),
                status:      if ip.resource_id.is_some() {
                    String::from("attached")
                } else {
                    String::from("available")
                },
                server_name: floating_ip_binding(ip, &server_names)
            })
            .collect();
        app.update_floating_ips(summaries);
    }

    if let Ok(images) = images_res {
        let summaries: Vec<ImageSummary> = images
            .iter()
            .map(|img| ImageSummary {
                id:      img.id.parse::<i32>().unwrap_or(0),
                name:    img.name.clone(),
                size_mb: i64::from(img.size),
                status:  format!("{:?}", img.status)
            })
            .collect();
        app.update_images(summaries);
    }

    if let Ok(resp) = network_drives_res {
        let summaries: Vec<NetworkDriveSummary> = resp
            .network_drives
            .iter()
            .map(|nd| NetworkDriveSummary {
                id:      nd.id.parse::<i32>().unwrap_or(0),
                name:    nd.name.clone(),
                size_gb: nd.size as i64,
                status:  format!("{:?}", nd.status)
            })
            .collect();
        app.update_network_drives(summaries);
    }

    if let Ok(resp) = vpcs_res {
        let summaries: Vec<VpcSummary> = resp
            .vpcs
            .iter()
            .map(|v| VpcSummary {
                id:       v.id.parse::<i32>().unwrap_or(0),
                name:     v.name.clone(),
                subnet:   v.subnet_v4.clone(),
                location: v.location.clone()
            })
            .collect();
        app.update_vpcs(summaries);
    }

    if let Ok(resp) = dedicated_servers_res {
        let summaries: Vec<DedicatedServerSummary> = resp
            .dedicated_servers
            .iter()
            .map(|ds| DedicatedServerSummary {
                id:     ds.id as i32,
                name:   ds.name.clone(),
                status: format!("{:?}", ds.status),
                cpu:    ds.cpu_description.clone(),
                ram:    ds.ram_description.clone(),
                disk:   ds.hdd_description.clone(),
                ip:     ds.ip.clone().unwrap_or_default()
            })
            .collect();
        app.update_dedicated_servers(summaries);
    }

    if let Ok(mailboxes) = mails_res {
        let summaries: Vec<MailSummary> = mailboxes
            .iter()
            .map(|m| MailSummary {
                name:    format!("{}@{}", m.mailbox, m.fqdn),
                owner:   m.owner_full_name.clone(),
                comment: m.comment.clone()
            })
            .collect();
        app.update_mails(summaries);
    }

    if let Ok(resp) = apps_res {
        let summaries: Vec<AppSummary> = resp
            .apps
            .iter()
            .map(|a| AppSummary {
                id:       a.id as i32,
                name:     a.name.clone(),
                status:   format!("{:?}", a.status),
                ip:       a.ip.clone().unwrap_or_default(),
                location: a.location.clone().unwrap_or_default()
            })
            .collect();
        app.update_apps(summaries);
    }

    if let Ok(resp) = ai_agents_res {
        let summaries: Vec<AiAgentSummary> = resp
            .agents
            .iter()
            .map(|a| AiAgentSummary {
                id:           a.id as i32,
                name:         a.name.clone(),
                status:       format!("{:?}", a.status),
                tokens_used:  a.used_tokens as i64,
                tokens_total: a.total_tokens as i64
            })
            .collect();
        app.update_ai_agents(summaries);
    }

    if let Ok(resp) = knowledge_bases_res {
        let summaries: Vec<KnowledgeBaseSummary> = resp
            .knowledge_bases
            .iter()
            .map(|kb| KnowledgeBaseSummary {
                id:             kb.id as i32,
                name:           kb.name.clone(),
                document_count: kb.documents_count as i32,
                status:         format!("{:?}", kb.status)
            })
            .collect();
        app.update_knowledge_bases(summaries);
    }

    if let Ok(resp) = ssh_keys_res {
        let keys: Vec<String> = resp.ssh_keys.iter().map(|k| k.name.clone()).collect();
        app.update_ssh_keys(keys);
    }

    if let Ok(resp) = finances_res {
        let f = resp.finances;
        let data = vec![format!("Balance: {:.2} {}", f.balance, f.currency)];
        app.update_finances(data);
    }

    if app.last_load_errors.is_empty() {
        app.status_message = Some("Resources loaded successfully".to_string());
    } else {
        app.error_message = Some(format!(
            "{} resource loads failed \u{2014} see events log",
            app.last_load_errors.len()
        ));
    }
}

#[cfg(feature = "tui")]
fn draw_splash(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>
) {
    use ratatui::{
        layout::{Constraint, Direction, Layout},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph}
    };

    let _ = terminal.draw(|f| {
        let size = f.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3)
            ])
            .split(size);

        // Header
        let header = Line::from(vec![
            Span::styled(
                "twc-rs",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            ),
            Span::raw(" v"),
            Span::raw(env!("CARGO_PKG_VERSION")),
        ]);
        let header_widget = Paragraph::new(header).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
        );
        f.render_widget(header_widget, chunks[0]);

        // ASCII art + loading
        let ascii_art = vec![
            Line::from(""),
            Line::from(Span::styled(
                "    ╔══════════════════════════════════╗",
                Style::default().fg(Color::Cyan)
            )),
            Line::from(Span::styled(
                "    ║        Timeweb Cloud CLI          ║",
                Style::default().fg(Color::Cyan)
            )),
            Line::from(Span::styled(
                "    ╚══════════════════════════════════╝",
                Style::default().fg(Color::Cyan)
            )),
            Line::from(""),
            Line::from(Span::styled(
                "    Loading resources...",
                Style::default().fg(Color::Yellow)
            )),
            Line::from(Span::styled(
                "    (this may take a moment on first run)",
                Style::default().fg(Color::DarkGray)
            )),
        ];
        let art_widget = Paragraph::new(ascii_art).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
        );
        f.render_widget(art_widget, chunks[1]);

        // Status bar
        let status = Line::from(Span::styled(
            "Fetching account, servers, databases, S3, k8s, projects...",
            Style::default().fg(Color::DarkGray)
        ));
        let status_widget = Paragraph::new(status).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
        );
        f.render_widget(status_widget, chunks[2]);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_token_cli_flag_takes_priority() {
        let result = resolve_token(Some("my-token"), None);
        assert_eq!(result.unwrap(), "my-token");
    }

    #[test]
    fn resolve_token_missing_without_config() {
        let dir = tempfile::tempdir().unwrap();
        let orig = std::env::var("XDG_CONFIG_HOME").ok();
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", dir.path());
        }
        let result = resolve_token(None, None);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("no API token configured"));
        unsafe {
            match orig {
                Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
                None => std::env::remove_var("XDG_CONFIG_HOME")
            }
        }
    }
}
