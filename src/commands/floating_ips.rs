// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

use rust_i18n::t;
use tabled::Tabled;
use timeweb_rs::{
    apis::{configuration::Configuration, floating_ip_api},
    models
};

use crate::{error::TwcError, output::OutputFormat};

/// Formats a string identifier for display.
fn fmt_id(v: &str) -> String {
    v.to_string()
}

/// Parses an availability zone code (e.g. `spb-1`) into the SDK enum.
///
/// # Errors
///
/// Returns [`TwcError::Api`] for an unrecognized zone code.
fn parse_zone(s: &str) -> Result<models::AvailabilityZone, TwcError> {
    match s.to_lowercase().as_str() {
        "spb-1" => Ok(models::AvailabilityZone::Spb1),
        "spb-2" => Ok(models::AvailabilityZone::Spb2),
        "spb-3" => Ok(models::AvailabilityZone::Spb3),
        "spb-4" => Ok(models::AvailabilityZone::Spb4),
        "msk-1" => Ok(models::AvailabilityZone::Msk1),
        "nsk-1" => Ok(models::AvailabilityZone::Nsk1),
        "ams-1" => Ok(models::AvailabilityZone::Ams1),
        "ala-1" => Ok(models::AvailabilityZone::Ala1),
        "fra-1" => Ok(models::AvailabilityZone::Fra1),
        other => Err(TwcError::Api(format!(
            "unknown availability zone: {other} \
             (expected one of spb-1, spb-2, spb-3, spb-4, msk-1, nsk-1, ams-1, ala-1, fra-1)"
        )))
    }
}

/// Renders a floating IP resource id (number or string) for display.
fn fmt_resource_id(id: Option<&models::FloatingIpResourceId>) -> String {
    match id {
        Some(models::FloatingIpResourceId::Number(n)) => format!("{n}"),
        Some(models::FloatingIpResourceId::String(s)) => s.clone(),
        None => String::new()
    }
}

/// Compact row for the floating IP list table.
#[derive(Tabled)]
struct FloatingIpRow {
    #[tabled(rename = "ID")]
    id:       String,
    #[tabled(rename = "IP")]
    ip:       String,
    #[tabled(rename = "Zone")]
    zone:     String,
    #[tabled(rename = "Resource")]
    resource: String
}

impl fmt::Display for FloatingIpRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {} {}", self.id, self.ip, self.zone, self.resource)
    }
}

/// Lists all floating IPs.
///
/// # Overview
///
/// Fetches all floating IPs from the Timeweb Cloud API and displays
/// them in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list(config: &Configuration, format: OutputFormat) -> Result<(), TwcError> {
    let resp = floating_ip_api::get_floating_ips(config).await?;

    let rows: Vec<FloatingIpRow> = resp
        .ips
        .iter()
        .map(|f| FloatingIpRow {
            id:       f.id.clone(),
            ip:       f.ip.clone().unwrap_or_default(),
            zone:     format!("{:?}", f.availability_zone),
            resource: f.resource_type.clone().unwrap_or_default()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_floating_ips"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.ips)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for f in &resp.ips {
                println!("{}\t{}", f.id, f.ip.clone().unwrap_or_default());
            }
        }
    }
    Ok(())
}

/// Deletes a floating IP by ID.
///
/// # Overview
///
/// Sends a delete request for the specified floating IP.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn delete(config: &Configuration, id: &str) -> Result<(), TwcError> {
    floating_ip_api::delete_floating_ip(config, id).await?;
    println!("{}", t!("cli.floating_ip_deleted", id => id));
    Ok(())
}

/// Shows detailed information about a single floating IP.
///
/// # Overview
///
/// Fetches one floating IP by ID and renders its id, address, zone and the
/// resource it is bound to (if any).
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn info(config: &Configuration, id: &str, format: OutputFormat) -> Result<(), TwcError> {
    let resp = floating_ip_api::get_floating_ip(config, id).await?;
    let f = &resp.ip;

    match format {
        OutputFormat::Table => {
            println!("ID:             {}", fmt_id(&f.id));
            println!("IP:             {}", f.ip.clone().unwrap_or_default());
            println!("Zone:           {}", f.availability_zone);
            println!(
                "Resource type:  {}",
                f.resource_type.clone().unwrap_or_default()
            );
            println!(
                "Resource ID:    {}",
                fmt_resource_id(f.resource_id.as_deref())
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            if let Some(out) = crate::output::serialized(format, f.as_ref()) {
                println!("{}", out?);
            }
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", fmt_id(&f.id), f.ip.clone().unwrap_or_default());
        }
    }
    Ok(())
}

/// Creates a new floating IP in the given availability zone.
///
/// # Overview
///
/// Requests a new floating IP. `DDoS` guard is disabled by default; the address
/// of the freshly created IP is printed on success.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on an unknown zone or on network/API failures.
pub async fn create(
    config: &Configuration,
    availability_zone: &str,
    format: OutputFormat
) -> Result<(), TwcError> {
    let zone = parse_zone(availability_zone)?;
    let req = models::CreateFloatingIp::new(false, zone);
    let resp = floating_ip_api::create_floating_ip(config, req).await?;
    let f = &resp.ip;
    let ip = f.ip.clone().unwrap_or_default();

    match format {
        OutputFormat::Table => {
            println!(
                "{}",
                t!("cli.floating_ip_created", id => fmt_id(&f.id), ip => ip)
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            if let Some(out) = crate::output::serialized(format, f.as_ref()) {
                println!("{}", out?);
            }
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", fmt_id(&f.id), ip);
        }
    }
    Ok(())
}

/// Binds a floating IP to a resource.
///
/// # Overview
///
/// Attaches the floating IP to the resource identified by `resource_id`.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn attach(config: &Configuration, id: &str, resource_id: i32) -> Result<(), TwcError> {
    let req = models::BindFloatingIp::new(
        "server".to_string(),
        models::BindFloatingIpResourceId::Number(f64::from(resource_id))
    );
    floating_ip_api::bind_floating_ip(config, id, req).await?;
    println!(
        "{}",
        t!("cli.floating_ip_attached", id => id, resource_id => resource_id)
    );
    Ok(())
}

/// Detaches a floating IP from its resource.
///
/// # Overview
///
/// Unbinds the floating IP so it is no longer attached to any resource.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn detach(config: &Configuration, id: &str) -> Result<(), TwcError> {
    floating_ip_api::unbind_floating_ip(config, id).await?;
    println!("{}", t!("cli.floating_ip_detached", id => id));
    Ok(())
}

/// Updates a floating IP's comment.
///
/// # Overview
///
/// Sends a partial update for the floating IP and confirms the change.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn set(config: &Configuration, id: &str, comment: Option<&str>) -> Result<(), TwcError> {
    let mut update = models::UpdateFloatingIp::new();
    if let Some(c) = comment {
        update.comment = Some(c.to_string());
    }
    floating_ip_api::update_floating_ip(config, id, update).await?;
    println!("{}", t!("cli.floating_ip_updated", id => id));
    Ok(())
}
