// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Per-resource subcommand dispatch helpers used by the CLI entrypoint.

use crate::{
    cli::{AppsCommands, ImageCommands, IpCommands, ServerCommands, VpcCommands},
    commands,
    error::TwcError,
    output::OutputFormat
};

/// Dispatches a server subcommand. Kept separate so its future is boxed at the
/// call site, keeping the main `run` stack frame small.
pub(crate) async fn handle_server(
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

/// Merges the positional app selector with the legacy `--id` flag.
///
/// clap guarantees at least one of the two is present, so the fallback empty
/// string is unreachable in practice.
fn app_selector(app: Option<String>, id: Option<String>) -> String {
    app.or(id).unwrap_or_default()
}

/// Dispatches an apps subcommand (boxed at the call site to keep `run` small).
pub(crate) async fn handle_apps(
    cmd: AppsCommands,
    config: &timeweb_rs::apis::configuration::Configuration,
    format: OutputFormat
) -> Result<(), TwcError> {
    match cmd {
        AppsCommands::List => commands::apps::list(config, format).await,
        AppsCommands::Info {
            app,
            id
        } => {
            let id = commands::apps::resolve_app(config, &app_selector(app, id)).await?;
            commands::apps::info(config, &id, format).await
        }
        AppsCommands::Delete {
            app,
            id
        } => {
            let id = commands::apps::resolve_app(config, &app_selector(app, id)).await?;
            commands::apps::delete(config, &id).await
        }
        AppsCommands::ListPresets => commands::apps::list_presets(config, format).await,
        AppsCommands::ListVcsProviders => commands::apps::list_vcs_providers(config, format).await,
        AppsCommands::ListRepositories {
            provider_id
        } => commands::apps::list_repositories(config, &provider_id, format).await,
        AppsCommands::Logs {
            app,
            id,
            tail,
            since,
            today
        } => {
            let id = commands::apps::resolve_app(config, &app_selector(app, id)).await?;
            commands::apps::logs(config, &id, tail, since.as_deref(), today, format).await
        }
        AppsCommands::ListDeploys {
            app,
            id
        } => {
            let id = commands::apps::resolve_app(config, &app_selector(app, id)).await?;
            commands::apps::list_deploys(config, &id, format).await
        }
        AppsCommands::DeployLogs {
            app,
            id,
            deploy_id,
            debug
        } => {
            let id = commands::apps::resolve_app(config, &app_selector(app, id)).await?;
            commands::apps::deploy_logs(config, &id, deploy_id.as_deref(), debug, format).await
        }
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
pub(crate) async fn handle_image(
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
pub(crate) async fn handle_ip(
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
pub(crate) async fn handle_vpc(
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
