// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

use rust_i18n::t;
use tabled::Tabled;
use timeweb_rs::apis::{configuration::Configuration, vpc_api};

use crate::{error::TwcError, output::OutputFormat};

/// Compact row for the VPC list table.
#[derive(Tabled)]
struct VpcRow {
    #[tabled(rename = "ID")]
    id:        String,
    #[tabled(rename = "Name")]
    name:      String,
    #[tabled(rename = "Subnet")]
    subnet:    String,
    #[tabled(rename = "Location")]
    location:  String,
    #[tabled(rename = "PublicIP")]
    public_ip: String
}

impl fmt::Display for VpcRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.id, self.name, self.subnet, self.location, self.public_ip
        )
    }
}

/// Lists all VPCs.
///
/// # Overview
///
/// Fetches all VPCs from the Timeweb Cloud API and displays
/// them in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list(config: &Configuration, format: OutputFormat) -> Result<(), TwcError> {
    let resp = vpc_api::get_vpcs(config).await?;

    let rows: Vec<VpcRow> = resp
        .vpcs
        .iter()
        .map(|v| VpcRow {
            id:        v.id.clone(),
            name:      v.name.clone(),
            subnet:    v.subnet_v4.clone(),
            location:  v.location.clone(),
            public_ip: v.public_ip.clone().unwrap_or_default()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_vpcs"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.vpcs)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for v in &resp.vpcs {
                println!("{}\t{}", v.id, v.name);
            }
        }
    }
    Ok(())
}

/// Deletes a VPC by ID.
///
/// # Overview
///
/// Sends a delete request for the specified VPC.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn delete(config: &Configuration, id: &str) -> Result<(), TwcError> {
    vpc_api::delete_vpc(config, id).await?;
    println!("{}", t!("cli.vpc_deleted", id => id));
    Ok(())
}
