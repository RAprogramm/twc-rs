// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

use rust_i18n::t;
use tabled::Tabled;
use timeweb_rs::{apis::balancers_api, models as bw};

use crate::{error::TwcError, output::OutputFormat};

/// Formats a float identifier for display.
fn fmt_id<T: std::fmt::Display>(v: T) -> String {
    v.to_string()
}

/// Compact row for the balancer list table.
#[derive(Tabled)]
struct BalancerRow {
    #[tabled(rename = "ID")]
    id:       String,
    #[tabled(rename = "Name")]
    name:     String,
    #[tabled(rename = "Status")]
    status:   String,
    #[tabled(rename = "Proto")]
    proto:    String,
    #[tabled(rename = "Algo")]
    algo:     String,
    #[tabled(rename = "IP")]
    ip:       String,
    #[tabled(rename = "Location")]
    location: String
}

impl fmt::Display for BalancerRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {} {}",
            self.id, self.name, self.status, self.proto, self.algo, self.ip, self.location
        )
    }
}

/// Compact row for the rule list table.
#[derive(Tabled)]
struct RuleRow {
    #[tabled(rename = "ID")]
    id:             String,
    #[tabled(rename = "Balancer Proto")]
    balancer_proto: String,
    #[tabled(rename = "Balancer Port")]
    balancer_port:  f64,
    #[tabled(rename = "Server Proto")]
    server_proto:   String,
    #[tabled(rename = "Server Port")]
    server_port:    f64
}

impl fmt::Display for RuleRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.id, self.balancer_proto, self.balancer_port, self.server_proto, self.server_port
        )
    }
}

/// Compact row for the IP list table.
#[derive(Tabled)]
struct IpRow {
    #[tabled(rename = "IP")]
    ip: String
}

impl fmt::Display for IpRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ip)
    }
}

/// Compact row for the preset list table.
#[derive(Tabled)]
struct PresetRow {
    #[tabled(rename = "ID")]
    id:              String,
    #[tabled(rename = "Description")]
    description:     String,
    #[tabled(rename = "Bandwidth")]
    bandwidth:       String,
    #[tabled(rename = "Replicas")]
    replica_count:   String,
    #[tabled(rename = "RPS")]
    request_per_sec: String,
    #[tabled(rename = "Price")]
    price:           String,
    #[tabled(rename = "Location")]
    location:        String
}

impl fmt::Display for PresetRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {} {}",
            self.id,
            self.description,
            self.bandwidth,
            self.replica_count,
            self.request_per_sec,
            self.price,
            self.location
        )
    }
}

/// Lists all balancers.
///
/// # Overview
///
/// Fetches balancers from the Timeweb Cloud API and displays them
/// in the requested output format.
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
    let resp = balancers_api::get_balancers(config, limit, offset).await?;

    let rows: Vec<BalancerRow> = resp
        .balancers
        .iter()
        .map(|b| BalancerRow {
            id:       fmt_id(b.id),
            name:     b.name.clone(),
            status:   format!("{:?}", b.status),
            proto:    format!("{:?}", b.proto),
            algo:     format!("{:?}", b.algo),
            ip:       b.ip.clone().unwrap_or_default(),
            location: format!("{:?}", b.location)
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_balancers"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.balancers)
                .expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for b in &resp.balancers {
                println!("{}\t{}", fmt_id(b.id), b.name);
            }
        }
    }
    Ok(())
}

/// Shows detailed info for a single balancer.
///
/// # Overview
///
/// Fetches balancer details by ID and displays them.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn info(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = balancers_api::get_balancer(config, id).await?;
    let b = &resp.balancer;

    match format {
        OutputFormat::Table => {
            println!("ID:                {}", fmt_id(b.id));
            println!("Name:              {}", b.name);
            println!("Status:            {:?}", b.status);
            println!("Protocol:          {:?}", b.proto);
            println!("Algorithm:         {:?}", b.algo);
            println!("Port:              {}", fmt_id(b.port));
            println!("Path:              {}", b.path);
            println!("IP:                {}", b.ip.clone().unwrap_or_default());
            println!(
                "Local IP:          {}",
                b.local_ip.clone().unwrap_or_default()
            );
            println!("Location:          {:?}", b.location);
            println!("Preset ID:         {}", fmt_id(b.preset_id));
            println!("SSL:               {}", b.is_ssl);
            println!("Sticky:            {}", b.is_sticky);
            println!("Keepalive:         {}", b.is_keepalive);
            println!("Use Proxy:         {}", b.is_use_proxy);
            println!("Fall:              {}", fmt_id(b.fall));
            println!("Rise:              {}", fmt_id(b.rise));
            println!("Interval:          {}", fmt_id(b.inter));
            println!("Timeout:           {}", fmt_id(b.timeout));
            println!("Maxconn:           {}", fmt_id(b.maxconn));
            println!("Connect Timeout:   {}", fmt_id(b.connect_timeout));
            println!("Client Timeout:    {}", fmt_id(b.client_timeout));
            println!("Server Timeout:    {}", fmt_id(b.server_timeout));
            println!("HTTPRequest Time:  {}", fmt_id(b.httprequest_timeout));
            println!("Created at:        {}", b.created_at);
            println!("Project ID:        {}", b.project_id);
            println!("Availability Zone: {:?}", b.availability_zone);
            println!("IPS:               {:?}", b.ips);
            println!("Rules:             {} rules", b.rules.len());
            for net in &b.networks {
                println!("Network:           {net:?}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out =
                crate::output::serialized(format, &resp.balancer).expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            println!("{}\t{}\t{:?}", fmt_id(b.id), b.name, b.status);
        }
    }
    Ok(())
}

/// Creates a new balancer.
///
/// # Overview
///
/// Creates a balancer with the given name and default settings.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn create(
    config: &timeweb_rs::apis::configuration::Configuration,
    name: &str,
    format: OutputFormat
) -> Result<(), TwcError> {
    let req = bw::CreateBalancer::new(
        name.to_string(),
        bw::create_balancer::Algo::Roundrobin,
        false,
        false,
        false,
        false,
        bw::create_balancer::Proto::Http,
        80.0,
        "/".to_string(),
        30.0,
        60.0,
        3.0,
        3.0,
        1
    );
    let resp = balancers_api::create_balancer(config, req).await?;
    let b = &resp.balancer;

    match format {
        OutputFormat::Table => {
            println!(
                "{}",
                t!("cli.balancer_created", name => b.name, id => fmt_id(b.id))
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out =
                crate::output::serialized(format, &resp.balancer).expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", fmt_id(b.id), b.name);
        }
    }
    Ok(())
}

/// Deletes a balancer by ID.
///
/// # Overview
///
/// Sends a delete request for the specified balancer.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn delete(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32
) -> Result<(), TwcError> {
    balancers_api::delete_balancer(config, id, None, None).await?;
    println!("{}", t!("cli.balancer_deleted", id => id));
    Ok(())
}

/// Updates a balancer by ID.
///
/// # Overview
///
/// Updates the balancer name via the Timeweb Cloud API.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn update(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32,
    name: Option<&str>,
    format: OutputFormat
) -> Result<(), TwcError> {
    let mut update = bw::UpdateBalancer::new();
    if let Some(n) = name {
        update.name = Some(n.to_string());
    }
    let resp = balancers_api::update_balancer(config, id, update).await?;
    let b = &resp.balancer;

    match format {
        OutputFormat::Table => {
            println!(
                "{}",
                t!("cli.balancer_updated", name => b.name, id => fmt_id(b.id))
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out =
                crate::output::serialized(format, &resp.balancer).expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", fmt_id(b.id), b.name);
        }
    }
    Ok(())
}

/// Lists rules for a balancer.
///
/// # Overview
///
/// Fetches rules for the specified balancer and displays them
/// in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn rule_list(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = balancers_api::get_balancer_rules(config, id).await?;

    let rows: Vec<RuleRow> = resp
        .rules
        .iter()
        .map(|r| RuleRow {
            id:             fmt_id(r.id),
            balancer_proto: format!("{:?}", r.balancer_proto),
            balancer_port:  r.balancer_port,
            server_proto:   format!("{:?}", r.server_proto),
            server_port:    r.server_port
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_rules"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out =
                crate::output::serialized(format, &resp.rules).expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for r in &resp.rules {
                println!("{}\t{}", fmt_id(r.id), r.balancer_port);
            }
        }
    }
    Ok(())
}

/// Creates a rule for a balancer.
///
/// # Overview
///
/// Creates a forwarding rule that maps a balancer port/protocol to a
/// server port/protocol.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn rule_create(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let req = bw::CreateRule::new(
        bw::create_rule::BalancerProto::Http,
        80.0,
        bw::create_rule::ServerProto::Http,
        8080.0
    );
    let resp = balancers_api::create_balancer_rule(config, id, req).await?;
    let r = &resp.rule;

    match format {
        OutputFormat::Table => {
            let proto_str = format!("{:?}", r.balancer_proto);
            let server_str = format!("{:?}", r.server_proto);
            println!(
                "{}",
                t!(
                    "cli.balancer_rule_created",
                    id => id,
                    rule_id => fmt_id(r.id),
                    proto => proto_str,
                    bport => r.balancer_port,
                    server => server_str,
                    sport => r.server_port
                )
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out =
                crate::output::serialized(format, &resp.rule).expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", fmt_id(r.id), r.balancer_port);
        }
    }
    Ok(())
}

/// Deletes a rule from a balancer.
///
/// # Overview
///
/// Deletes the specified rule from the balancer.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn rule_delete(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32,
    rule_id: i32
) -> Result<(), TwcError> {
    let resp = balancers_api::get_balancer_rules(config, id).await?;

    let target = resp.rules.iter().find(|r| r.id == i64::from(rule_id));

    let Some(_rule) = target else {
        return Err(TwcError::Api(format!(
            "rule {rule_id} not found on balancer {id}"
        )));
    };

    balancers_api::delete_balancer_rule(config, id, rule_id).await?;
    println!(
        "{}",
        t!("cli.balancer_rule_deleted", rule_id => rule_id, id => id)
    );
    Ok(())
}

/// Lists IPs for a balancer.
///
/// # Overview
///
/// Fetches IP addresses bound to the specified balancer and displays
/// them in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn ip_list(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = balancers_api::get_balancer_ips(config, id).await?;

    let rows: Vec<IpRow> = resp
        .ips
        .iter()
        .map(|ip| IpRow {
            ip: ip.clone()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_balancer_ips"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out =
                crate::output::serialized(format, &resp.ips).expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for ip in &resp.ips {
                println!("{ip}");
            }
        }
    }
    Ok(())
}

/// Adds an IP to a balancer.
///
/// # Overview
///
/// Attaches the specified IP address to the balancer.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn ip_add(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32,
    ip: &str
) -> Result<(), TwcError> {
    let req = bw::AddIpsToBalancerRequest::new(vec![ip.to_string()]);
    balancers_api::add_ips_to_balancer(config, id, req).await?;
    println!("{}", t!("cli.balancer_ip_added", ip => ip, id => id));
    Ok(())
}

/// Removes an IP from a balancer.
///
/// # Overview
///
/// Detaches the specified IP address from the balancer.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn ip_remove(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32,
    ip: &str
) -> Result<(), TwcError> {
    let req = bw::AddIpsToBalancerRequest::new(vec![ip.to_string()]);
    balancers_api::delete_ips_from_balancer(config, id, req).await?;
    println!("{}", t!("cli.balancer_ip_removed", ip => ip, id => id));
    Ok(())
}

/// Lists available balancer presets.
///
/// # Overview
///
/// Fetches balancer presets from the Timeweb Cloud API and displays them
/// in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn preset_list(
    config: &timeweb_rs::apis::configuration::Configuration,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = balancers_api::get_balancers_presets(config).await?;

    let rows: Vec<PresetRow> = resp
        .balancers_presets
        .iter()
        .map(|p| PresetRow {
            id:              fmt_id(p.id),
            description:     p.description_short.clone(),
            bandwidth:       fmt_id(p.bandwidth),
            replica_count:   fmt_id(p.replica_count),
            request_per_sec: p.request_per_second.clone(),
            price:           fmt_id(p.price),
            location:        format!("{:?}", p.location)
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_balancer_presets"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.balancers_presets)
                .expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for p in &resp.balancers_presets {
                println!(
                    "{}\t{}\t{}",
                    fmt_id(p.id),
                    p.description_short,
                    fmt_id(p.price)
                );
            }
        }
    }
    Ok(())
}
