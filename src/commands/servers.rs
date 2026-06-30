// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

use rust_i18n::t;
use tabled::Tabled;
use timeweb_rs::apis::{configuration::Configuration, servers_api};

use crate::{error::TwcError, output::OutputFormat};

/// Formats a float identifier for display.
fn fmt_id<T: std::fmt::Display>(v: T) -> String {
    v.to_string()
}

/// Compact row for the server list table.
#[derive(Tabled)]
struct ServerRow {
    #[tabled(rename = "ID")]
    id:       String,
    #[tabled(rename = "Name")]
    name:     String,
    #[tabled(rename = "Status")]
    status:   String,
    #[tabled(rename = "CPU")]
    cpu:      String,
    #[tabled(rename = "RAM (MB)")]
    ram:      String,
    #[tabled(rename = "OS")]
    os:       String,
    #[tabled(rename = "Location")]
    location: String
}

impl fmt::Display for ServerRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {} {}",
            self.id, self.name, self.status, self.cpu, self.ram, self.os, self.location
        )
    }
}

/// Lists all cloud servers.
///
/// # Overview
///
/// Fetches servers from the Timeweb Cloud API and displays them
/// in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list(
    config: &Configuration,
    limit: Option<i32>,
    offset: Option<i32>,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = servers_api::get_servers(config, limit, offset).await?;

    let rows: Vec<ServerRow> = resp
        .servers
        .iter()
        .map(|s| ServerRow {
            id:       fmt_id(s.id),
            name:     s.name.clone(),
            status:   format!("{:?}", s.status),
            cpu:      fmt_id(s.cpu),
            ram:      fmt_id(s.ram),
            os:       format!("{:?} {}", s.os.name, s.os.version.as_deref().unwrap_or("")),
            location: format!("{:?}", s.location)
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_servers_found"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out =
                crate::output::serialized(format, &resp.servers).expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for s in &resp.servers {
                println!("{}", fmt_id(s.id));
            }
        }
    }
    Ok(())
}

/// Shows detailed info for a single server.
///
/// # Overview
///
/// Fetches server details by ID and displays them.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn info(config: &Configuration, id: i32, format: OutputFormat) -> Result<(), TwcError> {
    let resp = servers_api::get_server(config, id).await?;
    let server = &resp.server;

    match format {
        OutputFormat::Table => {
            println!("ID:             {}", fmt_id(server.id));
            println!("Name:           {}", server.name);
            println!("Status:         {:?}", server.status);
            println!("CPU:            {}", fmt_id(server.cpu));
            println!("RAM:            {} MB", fmt_id(server.ram));
            println!("CPU Frequency:  {}", server.cpu_frequency);
            println!(
                "OS:             {:?} {}",
                server.os.name,
                server.os.version.as_deref().unwrap_or("")
            );
            println!("Location:       {:?}", server.location);
            println!("Comment:        {}", server.comment);
            println!("Created at:     {}", server.created_at);
            println!("DDoS Guard:     {}", server.is_ddos_guard);
            println!("Dedicated CPU:  {}", server.is_dedicated_cpu);
            for net in &server.networks {
                println!("Network:        {net:?}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out =
                crate::output::serialized(format, &resp.server).expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            println!(
                "{}\t{}\t{:?}",
                fmt_id(server.id),
                server.name,
                server.status
            );
        }
    }
    Ok(())
}

/// Deletes a cloud server by ID.
///
/// # Overview
///
/// Sends a delete request for the specified server.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn delete(config: &Configuration, id: i32) -> Result<(), TwcError> {
    servers_api::delete_server(config, id, None, None).await?;
    println!("{}", t!("cli.server_deleted", id => id));
    Ok(())
}

/// Reboots a cloud server by ID.
///
/// # Overview
///
/// Sends a reboot request for the specified server.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn reboot(config: &Configuration, id: i32) -> Result<(), TwcError> {
    servers_api::reboot_server(config, id).await?;
    println!("{}", t!("cli.server_rebooting", id => id));
    Ok(())
}

#[cfg(test)]
mod tests;
