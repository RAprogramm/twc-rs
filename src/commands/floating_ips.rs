// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

use tabled::Tabled;
use timeweb_rs::apis::{configuration::Configuration, floating_ip_api};

use crate::{error::TwcError, output::OutputFormat};

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
                println!("No floating IPs found.");
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&resp.ips)
                .map_err(|e| TwcError::Api(e.to_string()))?;
            println!("{json}");
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
    println!("Floating IP {id} deleted.");
    Ok(())
}
