// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

#[cfg(feature = "auth")]
mod auth;
mod auth_cli;
mod cli;
mod cli_dispatch;
mod commands;
mod config;
#[cfg(feature = "tui")]
mod dashboard;
mod error;
mod jwt;
mod output;
#[cfg(feature = "tui")]
mod tui;

rust_i18n::i18n!("locales", fallback = "en");

use auth_cli::ensure_token;
use clap::Parser;
use cli::{
    AccountCommands, AuthCommands, BalancerCommands, Cli, Commands, ConfigCommands,
    DatabaseCommands, DomainCommands, FirewallCommands, KubernetesCommands, ProjectCommands,
    RegistryCommands, S3Commands, SshCommands
};
use cli_dispatch::{handle_apps, handle_image, handle_ip, handle_server, handle_vpc};
use config::AppConfig;
#[cfg(feature = "tui")]
use dashboard::run_dashboard;
use error::TwcError;
use output::OutputFormat;
use rust_i18n::t;
use timeweb_rs::authenticated;

fn main() {
    use clap::CommandFactory as _;
    clap_complete::env::CompleteEnv::with_factory(Cli::command).complete();

    let Ok(runtime) = tokio::runtime::Runtime::new() else {
        eprintln!("Error: failed to start the async runtime");
        std::process::exit(1);
    };
    if let Err(e) = runtime.block_on(run()) {
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
        Commands::Doctor => commands::doctor::run(format),
        Commands::Update {
            check
        } => commands::update::run(check).await,
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
