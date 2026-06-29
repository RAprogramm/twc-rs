// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

use tabled::{Table, Tabled};
use timeweb_rs::apis::container_registry_api;

use crate::{error::TwcError, output::OutputFormat};

/// Compact row for the registry list table.
#[derive(Tabled)]
struct RegistryRow {
    #[tabled(rename = "ID")]
    id:          String,
    #[tabled(rename = "Name")]
    name:        String,
    #[tabled(rename = "Description")]
    description: String,
    #[tabled(rename = "Preset ID")]
    preset_id:   String,
    #[tabled(rename = "Disk (GB)")]
    disk:        String,
    #[tabled(rename = "Created")]
    created:     String
}

impl fmt::Display for RegistryRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {}",
            self.id, self.name, self.description, self.preset_id, self.disk, self.created
        )
    }
}

/// Compact row for the repository list table.
#[derive(Tabled)]
struct RepositoryRow {
    #[tabled(rename = "Name")]
    name:   String,
    #[tabled(rename = "Tag")]
    tag:    String,
    #[tabled(rename = "Digest")]
    digest: String
}

impl fmt::Display for RepositoryRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.name, self.tag, self.digest)
    }
}

/// Compact row for the preset list table.
#[derive(Tabled)]
struct PresetRow {
    #[tabled(rename = "ID")]
    id:          String,
    #[tabled(rename = "Description")]
    description: String,
    #[tabled(rename = "Disk (GB)")]
    disk:        String,
    #[tabled(rename = "Price")]
    price:       String,
    #[tabled(rename = "Location")]
    location:    String
}

impl fmt::Display for PresetRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.id, self.description, self.disk, self.price, self.location
        )
    }
}

/// Formats a registry ID for display.
fn fmt_id(v: i32) -> String {
    v.to_string()
}

/// Formats disk stats for display.
fn fmt_disk(disk: &timeweb_rs::models::RegistryOutDiskStats) -> String {
    format!("{} / {}", disk.used, disk.size)
}

/// Lists all container registries.
///
/// # Overview
///
/// Fetches all container registries from the Timeweb Cloud API and
/// displays them in the requested output format.
///
/// Note: the `limit` and `offset` parameters are accepted for
/// interface consistency but are currently ignored because the
/// upstream API does not support pagination for this endpoint.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list(
    config: &timeweb_rs::apis::configuration::Configuration,
    limit: Option<i32>,
    offset: Option<i32>,
    format: OutputFormat
) -> Result<(), TwcError> {
    if limit.is_some() || offset.is_some() {
        eprintln!("Note: limit/offset are not supported by this API endpoint.");
    }

    let resp = container_registry_api::get_registries(config).await?;

    let registries = resp.container_registry_list.unwrap_or_default();

    let rows: Vec<RegistryRow> = registries
        .iter()
        .map(|r| RegistryRow {
            id:          fmt_id(r.id),
            name:        r.name.clone(),
            description: r.description.clone(),
            preset_id:   fmt_id(r.preset_id),
            disk:        fmt_disk(&r.disk_stats),
            created:     r.created_at.to_rfc3339()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("No registries found.");
            } else {
                let table = Table::new(&rows).to_string();
                println!("{table}");
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&registries)
                .map_err(|e| TwcError::Api(e.to_string()))?;
            println!("{json}");
        }
        OutputFormat::Quiet => {
            for r in &registries {
                println!("{}\t{}", fmt_id(r.id), r.name);
            }
        }
    }
    Ok(())
}

/// Shows detailed info for a single container registry.
///
/// # Overview
///
/// Fetches registry details by ID and displays them.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn info(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = container_registry_api::get_registry(config, id).await?;
    let r = &resp.container_registry;

    match format {
        OutputFormat::Table => {
            println!("ID:             {}", fmt_id(r.id));
            println!("Name:           {}", r.name);
            println!("Description:    {}", r.description);
            println!("Preset ID:      {}", fmt_id(r.preset_id));
            println!("Configurator ID: {}", fmt_id(r.configurator_id));
            println!("Project ID:     {}", fmt_id(r.project_id));
            println!("Disk:           {}", fmt_disk(&r.disk_stats));
            println!("Created at:     {}", r.created_at.to_rfc3339());
            println!("Updated at:     {}", r.updated_at.to_rfc3339());
        }
        OutputFormat::Json => {
            let json =
                serde_json::to_string_pretty(&r).map_err(|e| TwcError::Api(e.to_string()))?;
            println!("{json}");
        }
        OutputFormat::Quiet => {
            println!("{}\t{}\t{}", fmt_id(r.id), r.name, r.description);
        }
    }
    Ok(())
}

/// Creates a new container registry.
///
/// # Overview
///
/// Creates a registry with the given name and optional preset.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn create(
    config: &timeweb_rs::apis::configuration::Configuration,
    name: &str,
    format: OutputFormat
) -> Result<(), TwcError> {
    let req = timeweb_rs::models::RegistryIn::new(name.to_string());
    let resp = container_registry_api::create_registry(config, req).await?;
    let r = &resp.container_registry;

    match format {
        OutputFormat::Table => {
            println!("Registry '{}' created (id: {}).", r.name, fmt_id(r.id));
        }
        OutputFormat::Json => {
            let json =
                serde_json::to_string_pretty(&r).map_err(|e| TwcError::Api(e.to_string()))?;
            println!("{json}");
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", fmt_id(r.id), r.name);
        }
    }
    Ok(())
}

/// Deletes a container registry by ID.
///
/// # Overview
///
/// Sends a delete request for the specified registry.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn delete(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32
) -> Result<(), TwcError> {
    container_registry_api::delete_registry(config, id).await?;
    println!("Registry {id} deleted.");
    Ok(())
}

/// Updates a container registry.
///
/// # Overview
///
/// Updates the registry description via the Timeweb Cloud API.
///
/// Note: the `name` parameter is accepted for interface consistency
/// but is not supported by the upstream API. Only the description
/// field can be updated.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn update(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32,
    description: Option<&str>,
    format: OutputFormat
) -> Result<(), TwcError> {
    let mut edit = timeweb_rs::models::RegistryEdit::new();
    edit.description = description.map(String::from);

    let resp = container_registry_api::update_registry(config, id, edit).await?;
    let r = &resp.container_registry;

    match format {
        OutputFormat::Table => {
            println!("Registry '{}' updated (id: {}).", r.name, fmt_id(r.id));
        }
        OutputFormat::Json => {
            let json =
                serde_json::to_string_pretty(&r).map_err(|e| TwcError::Api(e.to_string()))?;
            println!("{json}");
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", fmt_id(r.id), r.name);
        }
    }
    Ok(())
}

/// Lists repositories for a container registry.
///
/// # Overview
///
/// Fetches all repositories for the specified registry and displays
/// them in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn repo_list(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = container_registry_api::get_registry_repositories(config, id).await?;
    let repos = resp.container_registries_repositories;

    let rows: Vec<RepositoryRow> = repos
        .iter()
        .map(|repo| RepositoryRow {
            name:   repo.name.clone(),
            tag:    repo.tags.tag.clone(),
            digest: repo.tags.digest.clone()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("No repositories found.");
            } else {
                let table = Table::new(&rows).to_string();
                println!("{table}");
            }
        }
        OutputFormat::Json => {
            let json =
                serde_json::to_string_pretty(&repos).map_err(|e| TwcError::Api(e.to_string()))?;
            println!("{json}");
        }
        OutputFormat::Quiet => {
            for repo in &repos {
                println!("{}\t{}", repo.name, repo.tags.tag);
            }
        }
    }
    Ok(())
}

/// Lists available container registry presets.
///
/// # Overview
///
/// Fetches all container registry presets from the Timeweb Cloud API
/// and displays them in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn preset_list(
    config: &timeweb_rs::apis::configuration::Configuration,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = container_registry_api::get_registry_presets(config).await?;
    let presets = resp.container_registry_presets;

    let rows: Vec<PresetRow> = presets
        .iter()
        .map(|p| PresetRow {
            id:          fmt_id(p.id),
            description: p.description_short.clone(),
            disk:        p.disk.to_string(),
            price:       format!("{:.2}", p.price),
            location:    p.location.clone().unwrap_or_default()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("No presets found.");
            } else {
                let table = Table::new(&rows).to_string();
                println!("{table}");
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&presets)
                .map_err(|e| TwcError::Api(e.to_string()))?;
            println!("{json}");
        }
        OutputFormat::Quiet => {
            for p in &presets {
                println!("{}\t{}\t{}", p.id, p.description, p.price);
            }
        }
    }
    Ok(())
}
