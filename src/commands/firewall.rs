// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

use rust_i18n::t;
use tabled::Tabled;
use timeweb_rs::{apis::firewall_api, models as fw};

use crate::{error::TwcError, output::OutputFormat};

/// Formats a string identifier for display.
fn fmt_id(v: &str) -> String {
    v.to_string()
}

/// Compact row for the firewall group list table.
#[derive(Tabled)]
struct GroupRow {
    #[tabled(rename = "ID")]
    id:         String,
    #[tabled(rename = "Name")]
    name:       String,
    #[tabled(rename = "Policy")]
    policy:     String,
    #[tabled(rename = "Created")]
    created_at: String,
    #[tabled(rename = "Updated")]
    updated_at: String
}

impl fmt::Display for GroupRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.id, self.name, self.policy, self.created_at, self.updated_at
        )
    }
}

/// Compact row for the firewall rule list table.
#[derive(Tabled)]
struct RuleRow {
    #[tabled(rename = "ID")]
    id:          String,
    #[tabled(rename = "Direction")]
    direction:   String,
    #[tabled(rename = "Protocol")]
    protocol:    String,
    #[tabled(rename = "Port")]
    port:        String,
    #[tabled(rename = "Description")]
    description: String
}

impl fmt::Display for RuleRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.id, self.direction, self.protocol, self.port, self.description
        )
    }
}

/// Compact row for the resource list table.
#[derive(Tabled)]
struct ResourceRow {
    #[tabled(rename = "ID")]
    id:     String,
    #[tabled(rename = "Type")]
    r#type: String
}

impl fmt::Display for ResourceRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.id, self.r#type)
    }
}

/// Lists all firewall groups.
///
/// # Overview
///
/// Fetches firewall groups from the Timeweb Cloud API and displays them
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
    let resp = firewall_api::get_groups(config, limit, offset).await?;

    let rows: Vec<GroupRow> = resp
        .groups
        .iter()
        .map(|g| GroupRow {
            id:         fmt_id(&g.id),
            name:       g.name.clone(),
            policy:     format!("{:?}", g.policy),
            created_at: g
                .created_at
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, false),
            updated_at: g
                .updated_at
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_firewall_groups"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out =
                crate::output::serialized(format, &resp.groups).expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for g in &resp.groups {
                println!("{}\t{}", fmt_id(&g.id), g.name);
            }
        }
    }
    Ok(())
}

/// Shows detailed info for a single firewall group.
///
/// # Overview
///
/// Fetches firewall group details by ID and displays them.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn info(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: &str,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = firewall_api::get_group(config, id).await?;
    let g = &resp.group;

    match format {
        OutputFormat::Table => {
            println!("ID:            {}", fmt_id(&g.id));
            println!("Name:          {}", g.name);
            println!("Description:   {}", g.description);
            println!("Policy:        {:?}", g.policy);
            println!(
                "Created at:    {}",
                g.created_at
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
            );
            println!(
                "Updated at:    {}",
                g.updated_at
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out =
                crate::output::serialized(format, &resp.group).expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            println!("{}\t{}\t{:?}", fmt_id(&g.id), g.name, g.policy);
        }
    }
    Ok(())
}

/// Creates a new firewall group.
///
/// # Overview
///
/// Creates a firewall group with the given name and default settings.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn create(
    config: &timeweb_rs::apis::configuration::Configuration,
    name: &str,
    format: OutputFormat
) -> Result<(), TwcError> {
    let req = fw::FirewallGroupInApi::new(name.to_string());
    let resp = firewall_api::create_group(config, req, None).await?;
    let g = &resp.group;

    match format {
        OutputFormat::Table => {
            println!(
                "{}",
                t!("cli.firewall_group_created", name => g.name, id => fmt_id(&g.id))
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out =
                crate::output::serialized(format, &resp.group).expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", fmt_id(&g.id), g.name);
        }
    }
    Ok(())
}

/// Deletes a firewall group by ID.
///
/// # Overview
///
/// Sends a delete request for the specified firewall group.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn delete(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: &str
) -> Result<(), TwcError> {
    firewall_api::delete_group(config, id).await?;
    println!("{}", t!("cli.firewall_group_deleted", id => id));
    Ok(())
}

/// Updates a firewall group by ID.
///
/// # Overview
///
/// Updates the firewall group name via the Timeweb Cloud API.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn update(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: &str,
    name: Option<&str>,
    format: OutputFormat
) -> Result<(), TwcError> {
    let mut update = fw::FirewallGroupInApi::new(String::new());
    if let Some(n) = name {
        update.name = n.to_string();
    }
    let resp = firewall_api::update_group(config, id, update).await?;
    let g = &resp.group;

    match format {
        OutputFormat::Table => {
            println!(
                "{}",
                t!("cli.firewall_group_updated", name => g.name, id => fmt_id(&g.id))
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out =
                crate::output::serialized(format, &resp.group).expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", fmt_id(&g.id), g.name);
        }
    }
    Ok(())
}

/// Lists rules for a firewall group.
///
/// # Overview
///
/// Fetches rules for the specified firewall group and displays them
/// in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn rule_list(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: &str,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = firewall_api::get_group_rules(config, id, None, None).await?;

    let rows: Vec<RuleRow> = resp
        .rules
        .iter()
        .map(|r| RuleRow {
            id:          fmt_id(&r.id),
            direction:   format!("{:?}", r.direction),
            protocol:    format!("{:?}", r.protocol),
            port:        r.port.clone().unwrap_or_else(|| String::from("*")),
            description: r.description.clone()
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
                let dir = format!("{:?}", r.direction);
                let proto = format!("{:?}", r.protocol);
                println!("{}\t{}\t{}", fmt_id(&r.id), dir, proto);
            }
        }
    }
    Ok(())
}

/// Creates a rule for a firewall group.
///
/// # Overview
///
/// Creates an ingress TCP rule allowing all traffic (0.0.0.0/0, port *)
/// on the specified firewall group.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn rule_create(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: &str,
    format: OutputFormat
) -> Result<(), TwcError> {
    let req = fw::FirewallRuleInApi::new(
        fw::FirewallRuleDirection::Ingress,
        fw::FirewallRuleProtocol::Tcp
    );
    let resp = firewall_api::create_group_rule(config, id, req).await?;
    let r = &resp.rule;

    match format {
        OutputFormat::Table => {
            let dir_str = format!("{:?}", r.direction);
            let proto_str = format!("{:?}", r.protocol);
            let port_str = r.port.as_deref().unwrap_or("*");
            println!(
                "{}",
                t!(
                    "cli.firewall_rule_created",
                    id => id,
                    rule_id => fmt_id(&r.id),
                    dir => dir_str,
                    proto => proto_str,
                    port => port_str
                )
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out =
                crate::output::serialized(format, &resp.rule).expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", fmt_id(&r.id), r.direction);
        }
    }
    Ok(())
}

/// Deletes a rule from a firewall group.
///
/// # Overview
///
/// Deletes the specified rule from the firewall group.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn rule_delete(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: &str,
    rule_id: &str
) -> Result<(), TwcError> {
    firewall_api::delete_group_rule(config, id, rule_id).await?;
    println!(
        "{}",
        t!("cli.firewall_rule_deleted", rule_id => rule_id, id => id)
    );
    Ok(())
}

/// Lists resources for a firewall group.
///
/// # Overview
///
/// Fetches resources linked to the specified firewall group and displays
/// them in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn resource_list(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: &str,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = firewall_api::get_group_resources(config, id, None, None).await?;

    let rows: Vec<ResourceRow> = resp
        .resources
        .iter()
        .map(|r| ResourceRow {
            id:     r.id.to_string(),
            r#type: format!("{:?}", r.r#type)
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_firewall_resources"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.resources)
                .expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for r in &resp.resources {
                let rtype = format!("{:?}", r.r#type);
                println!("{}\t{}", r.id, rtype);
            }
        }
    }
    Ok(())
}

/// Adds a resource to a firewall group.
///
/// # Overview
///
/// Links the specified resource to the firewall group.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn resource_add(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: &str,
    resource_id: &str
) -> Result<(), TwcError> {
    firewall_api::add_resource_to_group(config, id, resource_id, None).await?;
    println!(
        "{}",
        t!("cli.firewall_resource_added", resource_id => resource_id, id => id)
    );
    Ok(())
}

/// Removes a resource from a firewall group.
///
/// # Overview
///
/// Unlinks the specified resource from the firewall group.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn resource_remove(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: &str,
    resource_id: &str
) -> Result<(), TwcError> {
    firewall_api::delete_resource_from_group(config, id, resource_id, None).await?;
    println!(
        "{}",
        t!("cli.firewall_resource_removed", resource_id => resource_id, id => id)
    );
    Ok(())
}
