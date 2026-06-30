// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

use rust_i18n::t;
use tabled::Tabled;
use timeweb_rs::{
    apis::{configuration::Configuration, servers_api},
    models
};

use crate::{error::TwcError, output::OutputFormat};

/// Formats a float identifier for display.
fn fmt_id<T: std::fmt::Display>(v: T) -> String {
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
        other => Err(TwcError::Api(
            t!("cli.server_invalid_zone", value => other).to_string()
        ))
    }
}

/// Creates a new cloud server from a preset and an OS image.
///
/// # Overview
///
/// Builds a preset-based [`models::CreateServer`] request and submits it.
/// On success the new server's id and name are printed. The request uses a
/// fixed bandwidth of 100 Mbps and disables `DDoS` guard by default; pass the
/// optional arguments to attach SSH keys, a comment, a project, or pin an
/// availability zone.
///
/// Rarely-used fields are intentionally deferred: custom configurator builds
/// (CPU/RAM/disk arrays), pre-installed software, cloud-init scripts, local
/// networks, custom network configuration, image-based installs, hostname and
/// avatar. These can be layered on later without changing the preset path.
///
/// # Errors
///
/// Returns [`TwcError::Api`] for an unknown availability zone or on network
/// and API failures.
pub async fn create(
    config: &Configuration,
    name: &str,
    preset_id: i32,
    os_id: i32,
    comment: Option<&str>,
    ssh_key_ids: &[i32],
    project_id: Option<i32>,
    availability_zone: Option<&str>
) -> Result<(), TwcError> {
    let mut body = models::CreateServer::new(name.to_owned());
    body.preset_id = Some(i64::from(preset_id));
    body.os_id = Some(i64::from(os_id));
    body.bandwidth = Some(100.0);
    body.is_ddos_guard = Some(false);

    if let Some(text) = comment {
        body.comment = Some(text.to_owned());
    }
    if !ssh_key_ids.is_empty() {
        body.ssh_keys_ids = Some(ssh_key_ids.iter().copied().map(f64::from).collect());
    }
    if let Some(project) = project_id {
        body.project_id = Some(i64::from(project));
    }
    if let Some(zone) = availability_zone {
        body.availability_zone = Some(parse_zone(zone)?);
    }

    let resp = servers_api::create_server(config, body).await?;
    println!(
        "{}",
        t!(
            "cli.server_created",
            id => fmt_id(resp.server.id),
            name => resp.server.name
        )
    );
    Ok(())
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
            let out = crate::output::serialized(format, &resp.servers)
                .transpose()?
                .unwrap_or_default();
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
            let out = crate::output::serialized(format, &resp.server)
                .transpose()?
                .unwrap_or_default();
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

/// Powers a server on.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn start(config: &Configuration, id: i32) -> Result<(), TwcError> {
    servers_api::start_server(config, id).await?;
    println!("{}", t!("cli.server_starting", id => id));
    Ok(())
}

/// Gracefully shuts a server down.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn shutdown(config: &Configuration, id: i32) -> Result<(), TwcError> {
    servers_api::shutdown_server(config, id).await?;
    println!("{}", t!("cli.server_shutting_down", id => id));
    Ok(())
}

/// Clones a server, printing the new server's id.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn clone(config: &Configuration, id: i32) -> Result<(), TwcError> {
    let resp = servers_api::clone_server(config, id).await?;
    println!("{}", t!("cli.server_cloned", id => fmt_id(resp.server.id)));
    Ok(())
}

/// Resets a server's root password (delivered by the provider).
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn reset_password(config: &Configuration, id: i32) -> Result<(), TwcError> {
    servers_api::reset_server_password(config, id).await?;
    println!("{}", t!("cli.server_password_reset", id => id));
    Ok(())
}

/// Compact row for the server preset list table.
#[derive(Tabled)]
struct PresetRow {
    #[tabled(rename = "ID")]
    id:          String,
    #[tabled(rename = "Location")]
    location:    String,
    #[tabled(rename = "CPU")]
    cpu:         String,
    #[tabled(rename = "RAM (MB)")]
    ram:         String,
    #[tabled(rename = "Disk (GB)")]
    disk:        String,
    #[tabled(rename = "Price")]
    price:       String,
    #[tabled(rename = "Description")]
    description: String
}

impl fmt::Display for PresetRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {} {}",
            self.id, self.location, self.cpu, self.ram, self.disk, self.price, self.description
        )
    }
}

/// Lists available server presets (ready-made configurations).
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list_presets(config: &Configuration, format: OutputFormat) -> Result<(), TwcError> {
    let resp = servers_api::get_servers_presets(config).await?;
    let rows: Vec<PresetRow> = resp
        .server_presets
        .iter()
        .map(|p| PresetRow {
            id:          fmt_id(p.id),
            location:    format!("{:?}", p.location),
            cpu:         fmt_id(p.cpu),
            ram:         fmt_id(p.ram),
            disk:        fmt_id(p.disk),
            price:       fmt_id(p.price),
            description: p.description.clone()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_presets_found"));
            } else {
                println!("{}", crate::output::render_table(&rows));
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.server_presets)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for p in &resp.server_presets {
                println!("{}", fmt_id(p.id));
            }
        }
    }
    Ok(())
}

/// Compact row for the OS image list table.
#[derive(Tabled)]
struct OsRow {
    #[tabled(rename = "ID")]
    id:      String,
    #[tabled(rename = "Family")]
    family:  String,
    #[tabled(rename = "Name")]
    name:    String,
    #[tabled(rename = "Version")]
    version: String
}

impl fmt::Display for OsRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.id, self.family, self.name, self.version
        )
    }
}

/// Lists installable operating system images.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list_os(config: &Configuration, format: OutputFormat) -> Result<(), TwcError> {
    let resp = servers_api::get_os_list(config).await?;
    let rows: Vec<OsRow> = resp
        .servers_os
        .iter()
        .map(|o| OsRow {
            id:      o.id.map(fmt_id).unwrap_or_default(),
            family:  o.family.clone().unwrap_or_default(),
            name:    o.name.clone().unwrap_or_default(),
            version: o.version.clone().unwrap_or_default()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_os_found"));
            } else {
                println!("{}", crate::output::render_table(&rows));
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.servers_os)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for o in &resp.servers_os {
                if let Some(id) = o.id {
                    println!("{}", fmt_id(id));
                }
            }
        }
    }
    Ok(())
}

/// Compact row for the software list table.
#[derive(Tabled)]
struct SoftwareRow {
    #[tabled(rename = "ID")]
    id:            String,
    #[tabled(rename = "Name")]
    name:          String,
    #[tabled(rename = "Installations")]
    installations: String
}

impl fmt::Display for SoftwareRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.id, self.name, self.installations)
    }
}

/// Lists available pre-installable software.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list_software(config: &Configuration, format: OutputFormat) -> Result<(), TwcError> {
    let resp = servers_api::get_software(config).await?;
    let rows: Vec<SoftwareRow> = resp
        .servers_software
        .iter()
        .map(|s| SoftwareRow {
            id:            s.id.map(fmt_id).unwrap_or_default(),
            name:          s.name.clone().unwrap_or_default(),
            installations: s.installations.map(fmt_id).unwrap_or_default()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_software_found"));
            } else {
                println!("{}", crate::output::render_table(&rows));
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.servers_software)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for s in &resp.servers_software {
                if let Some(id) = s.id {
                    println!("{}", fmt_id(id));
                }
            }
        }
    }
    Ok(())
}

/// Compact row for the configurator list table.
#[derive(Tabled)]
struct ConfiguratorRow {
    #[tabled(rename = "ID")]
    id:            String,
    #[tabled(rename = "Location")]
    location:      String,
    #[tabled(rename = "Disk Type")]
    disk_type:     String,
    #[tabled(rename = "CPU Frequency")]
    cpu_frequency: String
}

impl fmt::Display for ConfiguratorRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.id, self.location, self.disk_type, self.cpu_frequency
        )
    }
}

/// Lists server configurators (custom-build options).
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list_configurators(
    config: &Configuration,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = servers_api::get_configurators(config).await?;
    let rows: Vec<ConfiguratorRow> = resp
        .server_configurators
        .iter()
        .map(|c| ConfiguratorRow {
            id:            fmt_id(c.id),
            location:      format!("{:?}", c.location),
            disk_type:     format!("{:?}", c.disk_type),
            cpu_frequency: c.cpu_frequency.clone()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_configurators_found"));
            } else {
                println!("{}", crate::output::render_table(&rows));
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.server_configurators)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for c in &resp.server_configurators {
                println!("{}", fmt_id(c.id));
            }
        }
    }
    Ok(())
}

/// Compact row for the server disk list table.
#[derive(Tabled)]
struct DiskRow {
    #[tabled(rename = "ID")]
    id:     String,
    #[tabled(rename = "Size (MB)")]
    size:   String,
    #[tabled(rename = "Used (MB)")]
    used:   String,
    #[tabled(rename = "Type")]
    r#type: String,
    #[tabled(rename = "Status")]
    status: String
}

impl fmt::Display for DiskRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.id, self.size, self.used, self.r#type, self.status
        )
    }
}

/// Lists the disks attached to a server.
///
/// # Overview
///
/// Fetches the disks of the specified server and displays them.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list_disks(
    config: &Configuration,
    id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = servers_api::get_server_disks(config, id).await?;
    let rows: Vec<DiskRow> = resp
        .server_disks
        .iter()
        .map(|d| DiskRow {
            id:     fmt_id(d.id),
            size:   fmt_id(d.size),
            used:   fmt_id(d.used),
            r#type: d.r#type.clone(),
            status: d.status.clone()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_server_disks_found"));
            } else {
                println!("{}", crate::output::render_table(&rows));
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.server_disks)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for d in &resp.server_disks {
                println!("{}", fmt_id(d.id));
            }
        }
    }
    Ok(())
}

/// Compact row for the server IP list table.
#[derive(Tabled)]
struct IpRow {
    #[tabled(rename = "IP")]
    ip:      String,
    #[tabled(rename = "Type")]
    r#type:  String,
    #[tabled(rename = "PTR")]
    ptr:     String,
    #[tabled(rename = "Main")]
    is_main: String
}

impl fmt::Display for IpRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.ip, self.r#type, self.ptr, self.is_main
        )
    }
}

/// Lists the IP addresses of a server.
///
/// # Overview
///
/// Fetches the IP addresses of the specified server and displays them.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list_ips(
    config: &Configuration,
    id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = servers_api::get_server_ips(config, id).await?;
    let rows: Vec<IpRow> = resp
        .server_ips
        .iter()
        .map(|i| IpRow {
            ip:      i.ip.clone(),
            r#type:  format!("{:?}", i.r#type),
            ptr:     i.ptr.clone(),
            is_main: fmt_id(i.is_main)
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_server_ips_found"));
            } else {
                println!("{}", crate::output::render_table(&rows));
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.server_ips)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for i in &resp.server_ips {
                println!("{}", i.ip);
            }
        }
    }
    Ok(())
}

/// Compact row for the server log/history table.
#[derive(Tabled)]
struct LogRow {
    #[tabled(rename = "ID")]
    id:        String,
    #[tabled(rename = "Logged At")]
    logged_at: String,
    #[tabled(rename = "Event")]
    event:     String
}

impl fmt::Display for LogRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.id, self.logged_at, self.event)
    }
}

/// Shows the recent action history (logs) of a server.
///
/// # Overview
///
/// Fetches the most recent log entries of the specified server and
/// displays them, newest first.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn history(
    config: &Configuration,
    id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = servers_api::get_server_logs(config, id, None, None, Some("desc")).await?;
    let rows: Vec<LogRow> = resp
        .server_logs
        .iter()
        .map(|l| LogRow {
            id:        fmt_id(l.id),
            logged_at: l.logged_at.to_rfc3339(),
            event:     l.event.clone()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_server_logs_found"));
            } else {
                println!("{}", crate::output::render_table(&rows));
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.server_logs)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for l in &resp.server_logs {
                println!("{}", fmt_id(l.id));
            }
        }
    }
    Ok(())
}

/// Sets the NAT mode of a server's local network.
///
/// # Overview
///
/// Accepts one of `dnat_and_snat`, `snat`, or `no_nat` and applies it.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on an unknown mode or on network/API failures.
pub async fn set_nat_mode(
    config: &Configuration,
    id: i32,
    nat_mode: &str
) -> Result<(), TwcError> {
    let mode = match nat_mode {
        "dnat_and_snat" => models::update_server_nat_request::NatMode::DnatAndSnat,
        "snat" => models::update_server_nat_request::NatMode::Snat,
        "no_nat" => models::update_server_nat_request::NatMode::NoNat,
        other => {
            return Err(TwcError::Api(
                t!("cli.server_invalid_nat_mode", value => other).to_string()
            ));
        }
    };
    let body = models::UpdateServerNatRequest::new(mode);
    servers_api::update_server_nat(config, id, Some(body)).await?;
    println!(
        "{}",
        t!("cli.server_nat_mode_set", id => id, mode => nat_mode)
    );
    Ok(())
}

/// Sets the OS boot mode of a server.
///
/// # Overview
///
/// Accepts one of `default`, `single`, or `recovery_disk` and applies it.
/// The server is restarted after the boot mode changes.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on an unknown mode or on network/API failures.
pub async fn set_boot_mode(
    config: &Configuration,
    id: i32,
    boot_mode: &str
) -> Result<(), TwcError> {
    let mode = match boot_mode {
        "default" => models::update_server_os_boot_mode_request::BootMode::Default,
        "single" => models::update_server_os_boot_mode_request::BootMode::Single,
        "recovery_disk" => models::update_server_os_boot_mode_request::BootMode::RecoveryDisk,
        other => {
            return Err(TwcError::Api(
                t!("cli.server_invalid_boot_mode", value => other).to_string()
            ));
        }
    };
    let body = models::UpdateServerOsBootModeRequest::new(mode);
    servers_api::update_server_os_boot_mode(config, id, Some(body)).await?;
    println!(
        "{}",
        t!("cli.server_boot_mode_set", id => id, mode => boot_mode)
    );
    Ok(())
}

/// Resizes a server to a different preset.
///
/// # Overview
///
/// Applies the given preset to the server, leaving all other attributes
/// unchanged.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn resize(config: &Configuration, id: i32, preset_id: i32) -> Result<(), TwcError> {
    let body = models::UpdateServer {
        preset_id: Some(i64::from(preset_id)),
        ..Default::default()
    };
    servers_api::update_server(config, id, body).await?;
    println!(
        "{}",
        t!("cli.server_resized", id => id, preset => preset_id)
    );
    Ok(())
}

/// Reinstalls the operating system of a server.
///
/// # Overview
///
/// Applies the given OS image to the server, leaving all other attributes
/// unchanged. This wipes the server's data.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn reinstall(config: &Configuration, id: i32, os_id: i32) -> Result<(), TwcError> {
    let body = models::UpdateServer {
        os_id: Some(i64::from(os_id)),
        ..Default::default()
    };
    servers_api::update_server(config, id, body).await?;
    println!("{}", t!("cli.server_reinstalled", id => id, os => os_id));
    Ok(())
}

/// Updates a server's name and/or comment.
///
/// # Overview
///
/// Sends a partial [`models::UpdateServer`] that touches only the provided
/// `name` and `comment`, leaving every other attribute unchanged.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn set(
    config: &Configuration,
    id: i32,
    name: Option<&str>,
    comment: Option<&str>
) -> Result<(), TwcError> {
    let body = models::UpdateServer {
        name: name.map(String::from),
        comment: comment.map(String::from),
        ..Default::default()
    };
    servers_api::update_server(config, id, body).await?;
    println!("{}", t!("cli.server_updated", id => id));
    Ok(())
}

/// Compact row for the server disk-backup list table.
#[derive(Tabled, serde::Serialize)]
struct BackupRow {
    #[tabled(rename = "ID")]
    id:         String,
    #[tabled(rename = "Disk ID")]
    disk_id:    String,
    #[tabled(rename = "Status")]
    status:     String,
    #[tabled(rename = "Created At")]
    created_at: String,
    #[tabled(rename = "Size (MB)")]
    size:       String
}

impl fmt::Display for BackupRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.id, self.disk_id, self.status, self.created_at, self.size
        )
    }
}

/// Lists the disk backups of a server.
///
/// # Overview
///
/// Server backups are stored per disk. This resolves the server's disks via
/// [`servers_api::get_server_disks`] and then fetches each disk's backups via
/// [`servers_api::get_server_disk_backups`], flattening them into a single
/// table that pairs every backup with its owning disk id.
///
/// # Errors
///
/// Returns [`TwcError::Api`] if a disk id does not fit in 32 bits or on
/// network and API failures.
pub async fn backup_list(
    config: &Configuration,
    id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let disks = servers_api::get_server_disks(config, id).await?;

    let mut rows: Vec<BackupRow> = Vec::new();
    for disk in &disks.server_disks {
        let disk_id = i32::try_from(disk.id).map_err(|e| TwcError::Api(e.to_string()))?;
        let resp = servers_api::get_server_disk_backups(config, id, disk_id).await?;
        for b in &resp.backups {
            rows.push(BackupRow {
                id:         fmt_id(b.id),
                disk_id:    fmt_id(disk.id),
                status:     format!("{:?}", b.status),
                created_at: b.created_at.clone(),
                size:       fmt_id(b.size)
            });
        }
    }

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_server_backups_found"));
            } else {
                println!("{}", crate::output::render_table(&rows));
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            if let Some(out) = crate::output::serialized(format, &rows) {
                println!("{}", out?);
            }
        }
        OutputFormat::Quiet => {
            for row in &rows {
                println!("{}", row.id);
            }
        }
    }
    Ok(())
}

/// Creates a disk backup for a server's primary disk.
///
/// # Overview
///
/// Resolves the server's disks via [`servers_api::get_server_disks`] and picks
/// the system disk (falling back to the first disk), then submits a
/// [`models::CreateServerDiskBackupRequest`] via
/// [`servers_api::create_server_disk_backup`]. The new backup id is printed on
/// success.
///
/// # Errors
///
/// Returns [`TwcError::Api`] if the server has no disks, if a disk id does not
/// fit in 32 bits, or on network and API failures.
pub async fn backup_create(
    config: &Configuration,
    id: i32,
    comment: Option<&str>
) -> Result<(), TwcError> {
    let disks = servers_api::get_server_disks(config, id).await?;
    let disk = disks
        .server_disks
        .iter()
        .find(|d| d.is_system)
        .or_else(|| disks.server_disks.first())
        .ok_or_else(|| TwcError::Api(t!("cli.server_no_disks", id => id).to_string()))?;
    let disk_id = i32::try_from(disk.id).map_err(|e| TwcError::Api(e.to_string()))?;

    let mut body = models::CreateServerDiskBackupRequest::new();
    body.comment = comment.map(String::from);

    let resp = servers_api::create_server_disk_backup(config, id, disk_id, Some(body)).await?;
    let backup_id = resp.backup.map(|b| fmt_id(b.id)).unwrap_or_default();
    println!("{}", t!("cli.server_backup_created", id => backup_id));
    Ok(())
}

#[cfg(test)]
mod tests;
