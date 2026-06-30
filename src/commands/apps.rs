// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

use rust_i18n::t;
use tabled::Tabled;
use timeweb_rs::apis::{apps_api, configuration::Configuration};

use crate::{error::TwcError, output::OutputFormat};

/// Formats a float identifier for display.
fn fmt_id<T: std::fmt::Display>(v: T) -> String {
    v.to_string()
}

/// Compact row for the app list table.
#[derive(Tabled)]
struct AppRow {
    #[tabled(rename = "ID")]
    id:       String,
    #[tabled(rename = "Name")]
    name:     String,
    #[tabled(rename = "Status")]
    status:   String,
    #[tabled(rename = "IP")]
    ip:       String,
    #[tabled(rename = "Location")]
    location: String
}

impl fmt::Display for AppRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.id, self.name, self.status, self.ip, self.location
        )
    }
}

/// Lists all apps.
///
/// # Overview
///
/// Fetches all apps from the Timeweb Cloud API and displays
/// them in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list(config: &Configuration, format: OutputFormat) -> Result<(), TwcError> {
    let resp = apps_api::get_apps(config).await?;

    let rows: Vec<AppRow> = resp
        .apps
        .iter()
        .map(|a| AppRow {
            id:       fmt_id(a.id),
            name:     a.name.clone(),
            status:   format!("{:?}", a.status),
            ip:       a.ip.clone().unwrap_or_default(),
            location: a.location.clone().unwrap_or_default()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_apps"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.apps)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for a in &resp.apps {
                println!("{}\t{}", fmt_id(a.id), a.name);
            }
        }
    }
    Ok(())
}

/// Shows detailed info for a single app.
///
/// # Overview
///
/// Fetches an app by its identifier and prints its key fields.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn info(config: &Configuration, id: &str, format: OutputFormat) -> Result<(), TwcError> {
    let resp = apps_api::get_app(config, id).await?;
    let app = &resp.app;

    match format {
        OutputFormat::Table => {
            println!("ID:         {}", fmt_id(app.id));
            println!("Name:       {}", app.name);
            println!("Type:       {:?}", app.r#type);
            println!("Status:     {:?}", app.status);
            println!("IP:         {}", app.ip.clone().unwrap_or_default());
            println!("Location:   {}", app.location.clone().unwrap_or_default());
            println!(
                "Preset ID:  {}",
                app.preset_id.map(fmt_id).unwrap_or_default()
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            if let Some(out) = crate::output::serialized(format, &resp.app) {
                println!("{}", out?);
            }
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", fmt_id(app.id), app.name);
        }
    }
    Ok(())
}

/// Deletes an app by its identifier.
///
/// # Overview
///
/// Sends a delete request for the specified app and prints a confirmation.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn delete(config: &Configuration, id: &str) -> Result<(), TwcError> {
    apps_api::delete_app(config, id).await?;
    println!("{}", t!("cli.app_deleted", id => id));
    Ok(())
}

/// Compact row for the app presets table.
#[derive(Tabled)]
struct PresetRow {
    #[tabled(rename = "ID")]
    id:    String,
    #[tabled(rename = "Kind")]
    kind:  String,
    #[tabled(rename = "CPU")]
    cpu:   String,
    #[tabled(rename = "RAM")]
    ram:   String,
    #[tabled(rename = "Disk")]
    disk:  String,
    #[tabled(rename = "Price")]
    price: String
}

/// Lists available app presets (backend and frontend tariffs).
///
/// # Overview
///
/// Fetches all app presets and renders them in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list_presets(config: &Configuration, format: OutputFormat) -> Result<(), TwcError> {
    let resp = apps_api::get_apps_presets(config, "").await?;

    let mut rows: Vec<PresetRow> = Vec::new();
    if let Some(backend) = &resp.backend_presets {
        for p in backend {
            rows.push(PresetRow {
                id:    fmt_id(p.id),
                kind:  "backend".to_owned(),
                cpu:   fmt_id(p.cpu),
                ram:   fmt_id(p.ram),
                disk:  fmt_id(p.disk),
                price: fmt_id(p.price)
            });
        }
    }
    if let Some(frontend) = &resp.frontend_presets {
        for p in frontend {
            rows.push(PresetRow {
                id:    fmt_id(p.id),
                kind:  "frontend".to_owned(),
                cpu:   "-".to_owned(),
                ram:   "-".to_owned(),
                disk:  fmt_id(p.disk),
                price: fmt_id(p.price)
            });
        }
    }

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_app_presets"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            if let Some(out) = crate::output::serialized(format, &resp) {
                println!("{}", out?);
            }
        }
        OutputFormat::Quiet => {
            for r in &rows {
                println!("{}\t{}", r.id, r.kind);
            }
        }
    }
    Ok(())
}

/// Compact row for the VCS providers table.
#[derive(Tabled)]
struct ProviderRow {
    #[tabled(rename = "Provider ID")]
    provider_id: String,
    #[tabled(rename = "Login")]
    login:       String,
    #[tabled(rename = "Type")]
    kind:        String
}

/// Lists configured VCS providers.
///
/// # Overview
///
/// Fetches VCS providers linked to the account and renders them.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list_vcs_providers(
    config: &Configuration,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = apps_api::get_providers(config).await?;

    let rows: Vec<ProviderRow> = resp
        .providers
        .iter()
        .map(|p| ProviderRow {
            provider_id: fmt_id(p.provider_id),
            login:       p.login.clone(),
            kind:        format!("{:?}", p.provider_type)
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_vcs_providers"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            if let Some(out) = crate::output::serialized(format, &resp.providers) {
                println!("{}", out?);
            }
        }
        OutputFormat::Quiet => {
            for p in &resp.providers {
                println!("{}\t{}", fmt_id(p.provider_id), p.login);
            }
        }
    }
    Ok(())
}

/// Compact row for the repositories table.
#[derive(Tabled)]
struct RepoRow {
    #[tabled(rename = "ID")]
    id:        String,
    #[tabled(rename = "Name")]
    name:      String,
    #[tabled(rename = "Full Name")]
    full_name: String,
    #[tabled(rename = "URL")]
    url:       String,
    #[tabled(rename = "Private")]
    private:   String
}

/// Lists repositories available from a connected VCS provider.
///
/// # Overview
///
/// Fetches repositories of the given VCS provider and renders them.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list_repositories(
    config: &Configuration,
    provider_id: &str,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = apps_api::get_repositories(config, provider_id).await?;

    let rows: Vec<RepoRow> = resp
        .repositories
        .iter()
        .map(|r| RepoRow {
            id:        fmt_id(r.id),
            name:      r.name.clone(),
            full_name: r.full_name.clone(),
            url:       r.url.clone(),
            private:   r.is_private.to_string()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_repositories"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            if let Some(out) = crate::output::serialized(format, &resp.repositories) {
                println!("{}", out?);
            }
        }
        OutputFormat::Quiet => {
            for r in &resp.repositories {
                println!("{}\t{}", fmt_id(r.id), r.name);
            }
        }
    }
    Ok(())
}
