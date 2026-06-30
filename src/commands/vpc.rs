// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

use rust_i18n::t;
use tabled::Tabled;
use timeweb_rs::{
    apis::{configuration::Configuration, vpc_api},
    models
};

use crate::{error::TwcError, output::OutputFormat};

/// Formats an identifier for display.
fn fmt_id<T: std::fmt::Display>(v: T) -> String {
    v.to_string()
}

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

/// Shows detailed information about a single VPC.
///
/// # Overview
///
/// Fetches one VPC by ID and prints its identifier, name, subnet,
/// location, public IP and description.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn info(config: &Configuration, id: &str, format: OutputFormat) -> Result<(), TwcError> {
    let resp = vpc_api::get_vpc(config, id).await?;
    let v = &resp.vpc;

    match format {
        OutputFormat::Table => {
            println!("ID:            {}", fmt_id(&v.id));
            println!("Name:          {}", v.name);
            println!("Subnet:        {}", v.subnet_v4);
            println!("Location:      {}", v.location);
            println!("Public IP:     {}", v.public_ip.clone().unwrap_or_default());
            println!(
                "Description:   {}",
                v.description.clone().unwrap_or_default()
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            if let Some(out) = crate::output::serialized(format, &resp.vpc) {
                println!("{}", out?);
            }
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", fmt_id(&v.id), v.name);
        }
    }
    Ok(())
}

/// Creates a new VPC.
///
/// # Overview
///
/// Creates a VPC with the given name, IPv4 subnet mask and location,
/// then prints the new VPC identifier.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn create(
    config: &Configuration,
    name: &str,
    subnet_v4: &str,
    location: &str,
    format: OutputFormat
) -> Result<(), TwcError> {
    let req = models::CreateVpc::new(
        name.to_string(),
        subnet_v4.to_string(),
        location.to_string()
    );
    let resp = vpc_api::create_vpc(config, req).await?;
    let v = &resp.vpc;

    match format {
        OutputFormat::Table => {
            println!(
                "{}",
                t!("cli.vpc_created", name => v.name, id => fmt_id(&v.id))
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            if let Some(out) = crate::output::serialized(format, &resp.vpc) {
                println!("{}", out?);
            }
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", fmt_id(&v.id), v.name);
        }
    }
    Ok(())
}

/// Updates a VPC by ID.
///
/// # Overview
///
/// Updates the name and/or description of the specified VPC and
/// confirms the change.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn set(
    config: &Configuration,
    id: &str,
    name: Option<&str>,
    description: Option<&str>
) -> Result<(), TwcError> {
    let mut update = models::UpdateVpc::new();
    if let Some(n) = name {
        update.name = Some(n.to_string());
    }
    if let Some(d) = description {
        update.description = Some(d.to_string());
    }
    let resp = vpc_api::update_vpcs(config, id, update).await?;
    let v = &resp.vpc;
    println!(
        "{}",
        t!("cli.vpc_updated", name => v.name, id => fmt_id(&v.id))
    );
    Ok(())
}

/// Compact row for the VPC ports table.
#[derive(Tabled)]
struct VpcPortRow {
    #[tabled(rename = "ID")]
    id:       String,
    #[tabled(rename = "IP")]
    ip:       String,
    #[tabled(rename = "Resource")]
    resource: String
}

impl fmt::Display for VpcPortRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.id, self.ip, self.resource)
    }
}

/// Lists network ports of a VPC.
///
/// # Overview
///
/// Fetches all network ports inside the specified VPC and displays
/// each port's identifier, internal IPv4 address and attached resource.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list_ports(
    config: &Configuration,
    id: &str,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = vpc_api::get_vpc_ports(config, id).await?;

    let rows: Vec<VpcPortRow> = resp
        .vpc_ports
        .iter()
        .map(|p| VpcPortRow {
            id:       p.id.clone(),
            ip:       p.ipv4.clone(),
            resource: p.service.name.clone()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_vpc_ports"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            if let Some(out) = crate::output::serialized(format, &resp.vpc_ports) {
                println!("{}", out?);
            }
        }
        OutputFormat::Quiet => {
            for p in &resp.vpc_ports {
                println!("{}\t{}\t{}", fmt_id(&p.id), p.ipv4, p.service.name);
            }
        }
    }
    Ok(())
}
