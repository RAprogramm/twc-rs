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

use clap::Parser;
use cli::{
    AuthCommands, BalancerCommands, Cli, Commands, ConfigCommands, DatabaseCommands,
    DomainCommands, KubernetesCommands, ProjectCommands, RegistryCommands, S3Commands,
    ServerCommands, SshCommands
};
use config::AppConfig;
use error::TwcError;
use output::OutputFormat;
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
fn resolve_token(cli_token: Option<&str>) -> Result<String, TwcError> {
    if let Some(token) = cli_token {
        return Ok(token.to_string());
    }

    let app_config = AppConfig::load()?;
    if let Some(token) = app_config.token {
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
fn ensure_token(cli_token: Option<&str>) -> Result<String, TwcError> {
    resolve_token(cli_token).or_else(|_| prompt_and_save_token())
}

/// Shows an interactive prompt to get a token, then saves it.
fn prompt_and_save_token() -> Result<String, TwcError> {
    use colored::Colorize as _;
    use dialoguer::Select;

    println!("\n  {}\n", "No API token configured.".yellow().bold());

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
    use std::io::Read;

    eprint!("Paste your API token and press Ctrl+D: ");
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| TwcError::Io(e.to_string()))?;

    let token = buf.trim().to_string();
    if token.is_empty() {
        return Err(TwcError::Api("empty token".to_string()));
    }
    let masked = mask_token(&token);
    println!("  ✓ Token received: {masked}");
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
    println!(
        "\n  {} ({masked})\n",
        "Token saved. You won't be prompted again.".green().bold()
    );
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), TwcError> {
    let cli = Cli::parse();
    let format = OutputFormat::parse(&cli.format).map_err(TwcError::Api)?;

    match cli.command {
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
                token
            } => {
                let mut cfg = AppConfig::load()?;
                cfg.token = Some(token);
                cfg.save()?;
                println!("Token saved.");
                Ok(())
            }
        },
        Commands::Server(cmd) => {
            let token = ensure_token(cli.token.as_deref())?;
            let config = authenticated(token);
            match cmd {
                ServerCommands::List {
                    limit,
                    offset
                } => commands::servers::list(&config, limit, offset, format).await,
                ServerCommands::Info {
                    id
                } => commands::servers::info(&config, id, format).await,
                ServerCommands::Delete {
                    id
                } => commands::servers::delete(&config, id).await,
                ServerCommands::Reboot {
                    id
                } => commands::servers::reboot(&config, id).await
            }
        }
        Commands::Ssh(cmd) => {
            let token = ensure_token(cli.token.as_deref())?;
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
                } => commands::ssh_keys::delete(&config, id).await
            }
        }
        Commands::Project(cmd) => {
            let token = ensure_token(cli.token.as_deref())?;
            let config = authenticated(token);
            match cmd {
                ProjectCommands::List => commands::projects::list(&config, format).await,
                ProjectCommands::Create {
                    name,
                    description
                } => commands::projects::create(&config, &name, description.as_deref()).await,
                ProjectCommands::Delete {
                    id
                } => commands::projects::delete(&config, id).await
            }
        }
        Commands::Database(cmd) => {
            let token = ensure_token(cli.token.as_deref())?;
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
                DatabaseCommands::Restart {
                    id
                } => commands::databases::restart(&config, id).await,
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
            }
        }
        Commands::S3(cmd) => {
            let token = ensure_token(cli.token.as_deref())?;
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
                S3Commands::PresetList => commands::s3::preset_list(&config, format).await
            }
        }
        Commands::Kubernetes(cmd) => {
            let token = ensure_token(cli.token.as_deref())?;
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
                KubernetesCommands::Kubeconfig {
                    id
                } => commands::kubernetes::kubeconfig(&config, id).await,
                KubernetesCommands::Resources {
                    id
                } => commands::kubernetes::resources(&config, id, format).await
            }
        }
        Commands::Registry(cmd) => {
            let token = ensure_token(cli.token.as_deref())?;
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
            let token = ensure_token(cli.token.as_deref())?;
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
            let token = ensure_token(cli.token.as_deref())?;
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
            let token = ensure_token(cli.token.as_deref())?;
            run_dashboard(token, interval, config.theme).await
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
async fn run_dashboard(
    token: String,
    interval: u64,
    theme: crate::tui::themes::Theme
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

    // Show loading screen while fetching initial data
    let config = authenticated(token.clone());
    draw_splash(&mut terminal).await;
    app.is_loading = true;
    refresh_all(&config, &mut app).await;
    app.is_loading = false;

    let (tx, mut rx) = mpsc::unbounded_channel();
    let event_tx = tx.clone();

    tokio::spawn(async move {
        tui::event::run_event_loop(event_tx).await;
    });

    loop {
        if let Some(event) = rx.recv().await {
            if !tui::event::handle_event(&mut app, event) {
                break;
            }

            terminal
                .draw(|f| tui::ui::draw(f, &app))
                .map_err(|e| TwcError::Io(e.to_string()))?;

            if app.needs_refresh() {
                let config = authenticated(token.clone());
                refresh_all(&config, &mut app).await;
            }
        } else {
            break;
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

#[cfg(feature = "tui")]
#[allow(deprecated)]
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
    let mut has_error = false;

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
        timeweb_rs::apis::servers_api::get_servers(&c, None, None),
        timeweb_rs::apis::databases_api::get_databases(&c, None, None),
        timeweb_rs::apis::s3_api::get_storages(&c),
        timeweb_rs::apis::kubernetes_api::get_clusters(&c, None, None),
        timeweb_rs::apis::projects_api::get_projects(&c),
        timeweb_rs::apis::balancers_api::get_balancers(&c, None, None),
        timeweb_rs::apis::container_registry_api::get_registries(&c),
        timeweb_rs::apis::domains_api::get_domains(&c, None, None, None, None, None, None),
        timeweb_rs::apis::firewall_api::get_groups(&c, None, None),
        timeweb_rs::apis::floating_ip_api::get_floating_ips(&c),
        timeweb_rs::apis::images_api::get_images(&c, None, None),
        timeweb_rs::apis::network_drives_api::get_network_drives(&c),
        timeweb_rs::apis::vpc_api::get_vpcs(&c),
        timeweb_rs::apis::dedicated_servers_api::get_dedicated_servers(&c),
        timeweb_rs::apis::mail_api::get_mailboxes(&c, None, None, None),
        timeweb_rs::apis::apps_api::get_apps(&c),
        timeweb_rs::apis::ai_agents_api::get_agents(&c),
        timeweb_rs::apis::knowledge_bases_api::get_knowledgebases(&c),
        timeweb_rs::apis::ssh_api::get_keys(&c),
        timeweb_rs::apis::payments_api::get_finances(&c)
    );

    let mut account_id = 0.0;
    let mut balance = String::new();

    if let Ok(resp) = account_res {
        account_id = resp.status.company_info.id;
    } else {
        has_error = true;
        app.error_message = Some("Failed to load account".to_string());
    }
    if let Ok(ref resp) = finances_res {
        let f = &resp.finances;
        balance = format!("{:.2} {}", f.balance, f.currency);
    } else {
        has_error = true;
        app.error_message = Some("Failed to load balance".to_string());
    }
    app.update_account(AccountInfo {
        account_id,
        balance,
        status: String::from("active")
    });

    if let Ok(resp) = servers_res {
        let summaries: Vec<ServerSummary> = resp
            .servers
            .iter()
            .map(|s| ServerSummary {
                id:       s.id as i32,
                name:     s.name.clone(),
                status:   format!("{:?}", s.status),
                cpu:      s.cpu as i32,
                ram_mb:   s.ram as i32,
                disk_gb:  0,
                ip:       String::new(),
                location: format!("{:?}", s.location)
            })
            .collect();
        app.update_servers(summaries);
    } else {
        has_error = true;
        app.error_message = Some("Failed to load servers".to_string());
    }

    if let Ok(resp) = dbs_res {
        let summaries: Vec<DatabaseSummary> = resp
            .dbs
            .iter()
            .map(|d| DatabaseSummary {
                id:      d.id as i32,
                name:    d.name.clone(),
                status:  format!("{:?}", d.status),
                engine:  format!("{:?}", d.r#type),
                size_mb: 0
            })
            .collect();
        app.update_databases(summaries);
    } else if !has_error {
        app.status_message = Some("No databases available".to_string());
    }

    if let Ok(resp) = s3_res {
        let summaries: Vec<S3Summary> = resp
            .buckets
            .iter()
            .map(|b| S3Summary {
                id:           b.id as i32,
                name:         b.name.clone(),
                region:       b.location.clone(),
                size_bytes:   b.disk_stats.size as i64,
                bucket_count: 0
            })
            .collect();
        app.update_s3(summaries);
    } else if !has_error {
        app.status_message = Some("No S3 storages available".to_string());
    }

    if let Ok(resp) = k8s_res {
        let summaries: Vec<K8sSummary> = resp
            .clusters
            .iter()
            .map(|c| K8sSummary {
                id:         c.id,
                name:       c.name.clone(),
                status:     c.status.clone(),
                version:    c.k8s_version.clone(),
                node_count: c.cpu.unwrap_or(0)
            })
            .collect();
        app.update_k8s(summaries);
    } else if !has_error {
        app.status_message = Some("No Kubernetes clusters available".to_string());
    }

    if let Ok(resp) = projects_res {
        let summaries: Vec<ProjectSummary> = resp
            .projects
            .iter()
            .map(|p| ProjectSummary {
                id:           p.id as i32,
                name:         p.name.clone(),
                server_count: 0
            })
            .collect();
        app.update_projects(summaries);
    } else if !has_error {
        app.status_message = Some("No projects available".to_string());
    }

    if let Ok(resp) = balancers_res {
        let summaries: Vec<BalancerSummary> = resp
            .balancers
            .iter()
            .map(|b| BalancerSummary {
                id:       b.id as i32,
                name:     b.name.clone(),
                status:   format!("{:?}", b.status),
                ip:       b.ips.first().cloned().unwrap_or_default(),
                location: format!("{:?}", b.location)
            })
            .collect();
        app.update_balancers(summaries);
    } else if !has_error {
        app.status_message = Some("No balancers available".to_string());
    }

    if let Ok(resp) = registries_res {
        if let Some(registries) = resp.container_registry_list {
            let summaries: Vec<RegistrySummary> = registries
                .iter()
                .map(|r| RegistrySummary {
                    id:               r.id,
                    name:             r.name.clone(),
                    region:           String::new(),
                    repository_count: 0
                })
                .collect();
            app.update_registries(summaries);
        } else if !has_error {
            app.status_message = Some("No registries available".to_string());
        }
    } else {
        has_error = true;
        app.error_message = Some("Failed to load registries".to_string());
    }

    if let Ok(resp) = domains_res {
        let summaries: Vec<DomainSummary> = resp
            .domains
            .iter()
            .map(|d| DomainSummary {
                id:           d.id as i32,
                name:         d.fqdn.clone(),
                status:       format!("{:?}", d.domain_status),
                auto_prolong: d.is_autoprolong_enabled.unwrap_or(false)
            })
            .collect();
        app.update_domains(summaries);
    } else if !has_error {
        app.status_message = Some("No domains available".to_string());
    }

    if let Ok(resp) = firewalls_res {
        let summaries: Vec<FirewallSummary> = resp
            .groups
            .iter()
            .map(|g| FirewallSummary {
                id:             g.id.parse::<i32>().unwrap_or(0),
                name:           g.name.clone(),
                rule_count:     0,
                resource_count: 0
            })
            .collect();
        app.update_firewalls(summaries);
    } else if !has_error {
        app.status_message = Some("No firewalls available".to_string());
    }

    if let Ok(resp) = floating_ips_res {
        let summaries: Vec<FloatingIpSummary> = resp
            .ips
            .iter()
            .map(|ip| FloatingIpSummary {
                id:          ip.id.parse::<i32>().unwrap_or(0),
                ip:          ip.ip.clone().unwrap_or_default(),
                status:      String::new(),
                server_name: String::new()
            })
            .collect();
        app.update_floating_ips(summaries);
    } else if !has_error {
        app.status_message = Some("No floating IPs available".to_string());
    }

    if let Ok(resp) = images_res {
        let summaries: Vec<ImageSummary> = resp
            .images
            .iter()
            .map(|img| ImageSummary {
                id:      img.id.parse::<i32>().unwrap_or(0),
                name:    img.name.clone(),
                size_mb: img.size as i64,
                status:  format!("{:?}", img.status)
            })
            .collect();
        app.update_images(summaries);
    } else if !has_error {
        app.status_message = Some("No images available".to_string());
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
    } else if !has_error {
        app.status_message = Some("No network drives available".to_string());
    }

    if let Ok(resp) = vpcs_res {
        let summaries: Vec<VpcSummary> = resp
            .vpcs
            .iter()
            .map(|v| VpcSummary {
                id:           v.id.parse::<i32>().unwrap_or(0),
                name:         v.name.clone(),
                subnet_count: v.busy_address.len() as i32,
                status:       String::new()
            })
            .collect();
        app.update_vpcs(summaries);
    } else if !has_error {
        app.status_message = Some("No VPCs available".to_string());
    }

    if let Ok(resp) = dedicated_servers_res {
        let summaries: Vec<DedicatedServerSummary> = resp
            .dedicated_servers
            .iter()
            .map(|ds| DedicatedServerSummary {
                id:      ds.id as i32,
                name:    ds.name.clone(),
                status:  format!("{:?}", ds.status),
                cpu:     0,
                ram_mb:  0,
                disk_gb: 0
            })
            .collect();
        app.update_dedicated_servers(summaries);
    } else if !has_error {
        app.status_message = Some("No dedicated servers available".to_string());
    }

    if let Ok(resp) = mails_res {
        let summaries: Vec<MailSummary> = resp
            .mailboxes
            .iter()
            .map(|m| MailSummary {
                id:            0,
                name:          m.fqdn.clone(),
                mailbox_count: 1,
                status:        String::new()
            })
            .collect();
        app.update_mails(summaries);
    } else if !has_error {
        app.status_message = Some("No mailboxes available".to_string());
    }

    if let Ok(resp) = apps_res {
        let summaries: Vec<AppSummary> = resp
            .apps
            .iter()
            .map(|a| AppSummary {
                id:           a.id as i32,
                name:         a.name.clone(),
                status:       format!("{:?}", a.status),
                deploy_count: 0
            })
            .collect();
        app.update_apps(summaries);
    } else if !has_error {
        app.status_message = Some("No apps available".to_string());
    }

    if let Ok(resp) = ai_agents_res {
        let summaries: Vec<AiAgentSummary> = resp
            .agents
            .iter()
            .map(|a| AiAgentSummary {
                id:     a.id as i32,
                name:   a.name.clone(),
                status: format!("{:?}", a.status),
                model:  String::new()
            })
            .collect();
        app.update_ai_agents(summaries);
    } else if !has_error {
        app.status_message = Some("No AI agents available".to_string());
    }

    if let Ok(resp) = knowledge_bases_res {
        let summaries: Vec<KnowledgeBaseSummary> = resp
            .knowledgebases
            .iter()
            .map(|kb| KnowledgeBaseSummary {
                id:             kb.id as i32,
                name:           kb.name.clone(),
                document_count: kb.documents.len() as i32,
                status:         format!("{:?}", kb.status)
            })
            .collect();
        app.update_knowledge_bases(summaries);
    } else if !has_error {
        app.status_message = Some("No knowledge bases available".to_string());
    }

    if let Ok(resp) = ssh_keys_res {
        let keys: Vec<String> = resp.ssh_keys.iter().map(|k| k.name.clone()).collect();
        app.update_ssh_keys(keys);
    } else if !has_error {
        app.status_message = Some("No SSH keys available".to_string());
    }

    if let Ok(resp) = finances_res {
        let f = resp.finances;
        let data = vec![format!("Balance: {:.2} {}", f.balance, f.currency)];
        app.update_finances(data);
    } else if !has_error {
        app.status_message = Some("No finance data available".to_string());
    }

    if !has_error {
        app.status_message = Some("Resources loaded successfully".to_string());
    }
}

#[cfg(feature = "tui")]
async fn draw_splash(
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
        let result = resolve_token(Some("my-token"));
        assert_eq!(result.unwrap(), "my-token");
    }

    #[test]
    fn resolve_token_missing_without_config() {
        let dir = tempfile::tempdir().unwrap();
        let orig = std::env::var("XDG_CONFIG_HOME").ok();
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", dir.path());
        }
        let result = resolve_token(None);
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
