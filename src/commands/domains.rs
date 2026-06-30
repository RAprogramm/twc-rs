// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

use rust_i18n::t;
use tabled::Tabled;
use timeweb_rs::{apis::domains_api, models as dm};

use crate::{error::TwcError, output::OutputFormat};

/// Formats an f64 identifier for display.
fn fmt_id<T: std::fmt::Display>(v: T) -> String {
    v.to_string()
}

/// Compact row for the domain list table.
#[derive(Tabled)]
struct DomainRow {
    #[tabled(rename = "ID")]
    id:           String,
    #[tabled(rename = "FQDN")]
    fqdn:         String,
    #[tabled(rename = "Status")]
    status:       String,
    #[tabled(rename = "Expires")]
    expires:      String,
    #[tabled(rename = "AutoProlong")]
    auto_prolong: String,
    #[tabled(rename = "DaysLeft")]
    days_left:    String
}

impl fmt::Display for DomainRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {}",
            self.id, self.fqdn, self.status, self.expires, self.auto_prolong, self.days_left
        )
    }
}

/// Compact row for the DNS record table.
#[derive(Tabled)]
struct DnsRecordRow {
    #[tabled(rename = "ID")]
    id:       String,
    #[tabled(rename = "Type")]
    r#type:   String,
    #[tabled(rename = "Value")]
    value:    String,
    #[tabled(rename = "TTL")]
    ttl:      String,
    #[tabled(rename = "Priority")]
    priority: String
}

impl fmt::Display for DnsRecordRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.id, self.r#type, self.value, self.ttl, self.priority
        )
    }
}

/// Compact row for the name server table.
#[derive(Tabled)]
struct NameServerRow {
    #[tabled(rename = "Host")]
    host: String,
    #[tabled(rename = "IPs")]
    ips:  String
}

impl fmt::Display for NameServerRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.host, self.ips)
    }
}

/// Compact row for the subdomain table.
#[derive(Tabled)]
struct SubdomainRow {
    #[tabled(rename = "ID")]
    id:   String,
    #[tabled(rename = "FQDN")]
    fqdn: String,
    #[tabled(rename = "IP")]
    ip:   String
}

impl fmt::Display for SubdomainRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.id, self.fqdn, self.ip)
    }
}

/// Compact row for the TLD table.
#[derive(Tabled)]
struct TldRow {
    #[tabled(rename = "ID")]
    id:            String,
    #[tabled(rename = "Name")]
    name:          String,
    #[tabled(rename = "Price")]
    price:         String,
    #[tabled(rename = "Registrar")]
    registrar:     String,
    #[tabled(rename = "Published")]
    is_published:  String,
    #[tabled(rename = "Registered")]
    is_registered: String
}

impl fmt::Display for TldRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {}",
            self.id, self.name, self.price, self.registrar, self.is_published, self.is_registered
        )
    }
}

/// Compact row for the domain request table.
#[derive(Tabled)]
struct DomainRequestRow {
    #[tabled(rename = "ID")]
    id:      String,
    #[tabled(rename = "FQDN")]
    fqdn:    String,
    #[tabled(rename = "Type")]
    r#type:  String,
    #[tabled(rename = "Date")]
    date:    String,
    #[tabled(rename = "Message")]
    message: String
}

impl fmt::Display for DomainRequestRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.id, self.fqdn, self.r#type, self.date, self.message
        )
    }
}

/// Builds a [`dm::CreateDnsV2`] enum variant from a record type and value.
///
/// # Overview
///
/// Maps a string record type (A, AAAA, CNAME, MX, TXT, SRV) to the
/// corresponding SDK model constructor.
///
/// # Errors
///
/// Returns `Err` when the record type is not recognized.
fn build_dns_record(record_type: &str, value: String) -> Result<dm::CreateDnsV2, TwcError> {
    match record_type {
        "A" => Ok(dm::CreateDnsV2::A(Box::new(dm::A::new(
            dm::a_______::Type::A,
            value
        )))),
        "AAAA" => Ok(dm::CreateDnsV2::Aaaa(Box::new(dm::Aaaa::new(
            dm::aaaa_______::Type::Aaaa,
            value
        )))),
        "CNAME" => Ok(dm::CreateDnsV2::Cname(Box::new(dm::Cname::new(
            dm::cname_______::Type::Cname,
            value
        )))),
        "MX" => Ok(dm::CreateDnsV2::Mx(Box::new(dm::Mx::new(
            dm::mx_______::Type::Mx,
            value,
            10.0
        )))),
        "TXT" => Ok(dm::CreateDnsV2::Txt(Box::new(dm::Txt::new(
            dm::txt_______::Type::Txt,
            value
        )))),
        "SRV" => Ok(dm::CreateDnsV2::Srv(Box::new(dm::Srv::new(
            dm::srv_______::Type::Srv,
            value,
            "_tcp".to_string(),
            0.0,
            80.0,
            "localhost".to_string()
        )))),
        other => Err(TwcError::Api(format!(
            "unsupported DNS record type: {other} (use A, AAAA, CNAME, MX, TXT, SRV)"
        )))
    }
}

/// Lists all domains on the account.
///
/// # Overview
///
/// Fetches domains from the Timeweb Cloud API and displays them
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
    let resp = domains_api::get_domains(config, limit, offset, None, None, None, None).await?;

    let rows: Vec<DomainRow> = resp
        .domains
        .iter()
        .map(|d| DomainRow {
            id:           fmt_id(d.id),
            fqdn:         d.fqdn.clone(),
            status:       format!("{:?}", d.domain_status),
            expires:      d.expiration.clone(),
            auto_prolong: d
                .is_autoprolong_enabled
                .map_or_else(|| "n/a".to_string(), |v| v.to_string()),
            days_left:    format!("{:.0}", d.days_left)
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_domains"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.domains)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for d in &resp.domains {
                println!("{}\t{}", fmt_id(d.id), d.fqdn);
            }
        }
    }
    Ok(())
}

/// Shows detailed info for a single domain.
///
/// # Overview
///
/// Fetches domain details by FQDN and displays them.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn info(
    config: &timeweb_rs::apis::configuration::Configuration,
    fqdn: String,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = domains_api::get_domain(config, &fqdn).await?;
    let d = &resp.domain;

    match format {
        OutputFormat::Table => {
            println!("ID:                  {}", fmt_id(d.id));
            println!("FQDN:                {}", d.fqdn);
            println!("Status:              {:?}", d.domain_status);
            println!("Expiration:          {}", d.expiration);
            println!("Days Left:           {:.0}", d.days_left);
            println!(
                "Auto Prolong:        {}",
                d.is_autoprolong_enabled
                    .map_or_else(|| "n/a".to_string(), |v| v.to_string())
            );
            println!("Premium:             {}", d.is_premium);
            println!("Prolong Allowed:     {}", d.is_prolong_allowed);
            println!("Technical:           {}", d.is_technical);
            println!(
                "Whois Privacy:       {}",
                d.is_whois_privacy_enabled
                    .map_or_else(|| "n/a".to_string(), |v| v.to_string())
            );
            println!(
                "Linked IP:           {}",
                d.linked_ip.clone().unwrap_or_default()
            );
            println!(
                "Paid Till:           {}",
                d.paid_till.clone().unwrap_or_else(|| "n/a".to_string())
            );
            println!(
                "Provider:            {}",
                d.provider.clone().unwrap_or_else(|| "n/a".to_string())
            );
            println!(
                "TLD ID:              {}",
                d.tld_id
                    .map_or_else(|| "n/a".to_string(), |v| format!("{v:.0}"))
            );
            println!(
                "Person ID:           {}",
                d.person_id
                    .map_or_else(|| "n/a".to_string(), |v| format!("{v:.0}"))
            );
            println!("Subdomains:          {} subdomains", d.subdomains.len());
            for sub in &d.subdomains {
                println!(
                    "  - {} (ip: {})",
                    sub.fqdn,
                    sub.linked_ip.clone().unwrap_or_default()
                );
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.domain)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            println!("{}\t{}\t{:?}", fmt_id(d.id), d.fqdn, d.domain_status);
        }
    }
    Ok(())
}

/// Checks domain availability for registration.
///
/// # Overview
///
/// Checks whether the given domain name is available for registration
/// via the Timeweb Cloud API.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn check(
    config: &timeweb_rs::apis::configuration::Configuration,
    fqdn: String
) -> Result<(), TwcError> {
    let resp = domains_api::check_domain(config, &fqdn).await?;

    if resp.is_domain_available {
        println!("{}", t!("cli.domain_available", fqdn => fqdn));
    } else {
        println!("{}", t!("cli.domain_not_available", fqdn => fqdn));
    }
    Ok(())
}

/// Adds a domain to the account.
///
/// # Overview
///
/// Registers a domain on the Timeweb Cloud account.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn add(
    config: &timeweb_rs::apis::configuration::Configuration,
    fqdn: String,
    format: OutputFormat
) -> Result<(), TwcError> {
    domains_api::add_domain(config, &fqdn).await?;

    match format {
        OutputFormat::Table => {
            println!("{}", t!("cli.domain_added", fqdn => fqdn));
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            println!("{{\"fqdn\": \"{fqdn}\", \"status\": \"added\"}}");
        }
        OutputFormat::Quiet => {
            println!("{fqdn}\tadded");
        }
    }
    Ok(())
}

/// Deletes a domain from the account.
///
/// # Overview
///
/// Removes the specified domain from the Timeweb Cloud account.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn delete(
    config: &timeweb_rs::apis::configuration::Configuration,
    fqdn: String
) -> Result<(), TwcError> {
    domains_api::delete_domain(config, &fqdn).await?;
    println!("{}", t!("cli.domain_deleted", fqdn => fqdn));
    Ok(())
}

/// Lists DNS records for a domain.
///
/// # Overview
///
/// Fetches DNS records for the specified domain and displays them
/// in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn dns_list(
    config: &timeweb_rs::apis::configuration::Configuration,
    fqdn: String,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = domains_api::get_domain_dns_records(config, &fqdn, None, None).await?;

    let rows: Vec<DnsRecordRow> = resp
        .dns_records
        .iter()
        .map(|r| {
            let priority_str = match &*r.data {
                dm::DnsRecordData {
                    priority: Some(p), ..
                } => format!("{p:.0}"),
                _ => String::new()
            };
            DnsRecordRow {
                id:       r
                    .id
                    .as_ref()
                    .and_then(|v| v.map(|vv| format!("{vv:.0}")))
                    .unwrap_or_default(),
                r#type:   format!("{:?}", r.r#type),
                value:    r.data.value.clone(),
                ttl:      r
                    .ttl
                    .map_or_else(|| "n/a".to_string(), |v| format!("{v:.0}")),
                priority: priority_str
            }
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_dns_records"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.dns_records)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for r in &resp.dns_records {
                let id_str =
                    r.id.as_ref()
                        .and_then(|v| v.map(|vv| format!("{vv:.0}")))
                        .unwrap_or_default();
                let type_str = format!("{:?}", r.r#type);
                println!("{id_str}\t{type_str}\t{}", r.data.value);
            }
        }
    }
    Ok(())
}

/// Adds a DNS record to a domain.
///
/// # Overview
///
/// Creates a new DNS record for the specified domain using the v2 API.
/// Supported record types: A, AAAA, CNAME, MX, TXT, SRV.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn dns_add(
    config: &timeweb_rs::apis::configuration::Configuration,
    fqdn: String,
    record_type: String,
    value: String,
    format: OutputFormat
) -> Result<(), TwcError> {
    let dns = build_dns_record(&record_type.to_uppercase(), value.clone())?;
    let _resp = domains_api::create_domain_dns_record_v2(config, &fqdn, dns).await?;

    match format {
        OutputFormat::Table => {
            println!(
                "{}",
                t!("cli.dns_record_added", rtype => record_type, fqdn => fqdn)
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            println!(
                "{{\"fqdn\": \"{fqdn}\", \"type\": \"{record_type}\", \"status\": \"added\"}}"
            );
        }
        OutputFormat::Quiet => {
            println!("{fqdn}\t{record_type}\t{value}\tadded");
        }
    }
    Ok(())
}

/// Deletes a DNS record from a domain.
///
/// # Overview
///
/// Removes the specified DNS record from the domain using the v2 API.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn dns_delete(
    config: &timeweb_rs::apis::configuration::Configuration,
    fqdn: String,
    record_id: i32
) -> Result<(), TwcError> {
    domains_api::delete_domain_dns_record_v2(config, &fqdn, record_id).await?;
    println!(
        "{}",
        t!("cli.dns_record_deleted", id => record_id, fqdn => fqdn)
    );
    Ok(())
}

/// Updates a DNS record on a domain.
///
/// # Overview
///
/// Updates an existing DNS record for the domain using the v2 API.
/// The `record_type` and `value` parameters define the new record content.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn dns_update(
    config: &timeweb_rs::apis::configuration::Configuration,
    fqdn: String,
    record_id: i32,
    record_type: String,
    value: String,
    format: OutputFormat
) -> Result<(), TwcError> {
    let dns = build_dns_record(&record_type.to_uppercase(), value)?;
    let _resp = domains_api::update_domain_dns_record_v2(config, &fqdn, record_id, dns).await?;

    match format {
        OutputFormat::Table => {
            println!(
                "{}",
                t!("cli.dns_record_updated", id => record_id, fqdn => fqdn)
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            println!(
                "{{\"fqdn\": \"{fqdn}\", \"record_id\": {record_id}, \"status\": \"updated\"}}"
            );
        }
        OutputFormat::Quiet => {
            println!("{fqdn}\t{record_id}\t{record_type}\tupdated");
        }
    }
    Ok(())
}

/// Lists name servers for a domain.
///
/// # Overview
///
/// Fetches name servers for the specified domain and displays them
/// in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn ns_list(
    config: &timeweb_rs::apis::configuration::Configuration,
    fqdn: String,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = domains_api::get_domain_name_servers(config, &fqdn).await?;

    let rows: Vec<NameServerRow> = resp
        .name_servers
        .iter()
        .flat_map(|ns| {
            ns.items
                .iter()
                .map(|item| NameServerRow {
                    host: item.host.clone(),
                    ips:  item.ips.join(", ")
                })
                .collect::<Vec<_>>()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_name_servers"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.name_servers)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for ns in &resp.name_servers {
                for item in &ns.items {
                    println!("{}\t{}", item.host, item.ips.join(","));
                }
            }
        }
    }
    Ok(())
}

/// Updates name servers for a domain.
///
/// # Overview
///
/// Sets new name servers for the specified domain.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn ns_update(
    config: &timeweb_rs::apis::configuration::Configuration,
    fqdn: String,
    ns1: String,
    ns2: String,
    format: OutputFormat
) -> Result<(), TwcError> {
    let update = dm::UpdateDomainNameServers::new(vec![
        dm::UpdateDomainNameServersNameServersInner::new(ns1.clone()),
        dm::UpdateDomainNameServersNameServersInner::new(ns2.clone()),
    ]);
    let resp = domains_api::update_domain_name_servers(config, &fqdn, update).await?;

    match format {
        OutputFormat::Table => {
            println!("{}", t!("cli.name_servers_updated", fqdn => fqdn));
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.name_servers)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            println!("{fqdn}\t{ns1}\t{ns2}\tupdated");
        }
    }
    Ok(())
}

/// Lists subdomains for a domain.
///
/// # Overview
///
/// Fetches subdomains from the domain info and displays them
/// in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn subdomain_list(
    config: &timeweb_rs::apis::configuration::Configuration,
    fqdn: String,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = domains_api::get_domain(config, &fqdn).await?;
    let d = &resp.domain;

    let rows: Vec<SubdomainRow> = d
        .subdomains
        .iter()
        .map(|s| SubdomainRow {
            id:   fmt_id(s.id),
            fqdn: s.fqdn.clone(),
            ip:   s.linked_ip.clone().unwrap_or_default()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_subdomains"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &d.subdomains)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for s in &d.subdomains {
                println!("{}\t{}", fmt_id(s.id), s.fqdn);
            }
        }
    }
    Ok(())
}

/// Adds a subdomain to a domain.
///
/// # Overview
///
/// Creates a new subdomain for the specified domain.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn subdomain_add(
    config: &timeweb_rs::apis::configuration::Configuration,
    fqdn: String,
    subdomain: String,
    format: OutputFormat
) -> Result<(), TwcError> {
    let _resp = domains_api::add_subdomain(config, &fqdn, &subdomain).await?;

    match format {
        OutputFormat::Table => {
            println!(
                "{}",
                t!("cli.domain_subdomain_added", subdomain => subdomain, fqdn => fqdn)
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            println!(
                "{{\"fqdn\": \"{fqdn}\", \"subdomain\": \"{subdomain}\", \"status\": \"added\"}}"
            );
        }
        OutputFormat::Quiet => {
            println!("{fqdn}\t{subdomain}\tadded");
        }
    }
    Ok(())
}

/// Deletes a subdomain from a domain.
///
/// # Overview
///
/// Removes the specified subdomain from the domain.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn subdomain_delete(
    config: &timeweb_rs::apis::configuration::Configuration,
    fqdn: String,
    subdomain: String
) -> Result<(), TwcError> {
    domains_api::delete_subdomain(config, &fqdn, &subdomain).await?;
    println!(
        "{}",
        t!("cli.domain_subdomain_deleted", subdomain => subdomain, fqdn => fqdn)
    );
    Ok(())
}

/// Lists domain requests (registration/transfer/prolongation).
///
/// # Overview
///
/// Fetches domain requests from the Timeweb Cloud API and displays them
/// in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn request_list(
    config: &timeweb_rs::apis::configuration::Configuration,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = domains_api::get_domain_requests(config, None).await?;

    let rows: Vec<DomainRequestRow> = resp
        .requests
        .iter()
        .map(|r| DomainRequestRow {
            id:      fmt_id(r.id),
            fqdn:    r.fqdn.clone(),
            r#type:  format!("{:?}", r.r#type),
            date:    r.date.to_rfc3339(),
            message: r.message.clone().unwrap_or_default()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_domain_requests"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.requests)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for r in &resp.requests {
                let type_str = format!("{:?}", r.r#type);
                println!("{}\t{}\t{type_str}", fmt_id(r.id), r.fqdn);
            }
        }
    }
    Ok(())
}

/// Lists available TLDs (top-level domains).
///
/// # Overview
///
/// Fetches TLDs from the Timeweb Cloud API and displays them
/// in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn tld_list(
    config: &timeweb_rs::apis::configuration::Configuration,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = domains_api::get_tlds(config, None, None).await?;

    let rows: Vec<TldRow> = resp
        .top_level_domains
        .iter()
        .map(|t| TldRow {
            id:            fmt_id(t.id),
            name:          t.name.clone(),
            price:         format!("{:.2}", t.price),
            registrar:     format!("{:?}", t.registrar),
            is_published:  t.is_published.to_string(),
            is_registered: t.is_registered.to_string()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_tlds"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.top_level_domains)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for t in &resp.top_level_domains {
                let price_str = format!("{:.2}", t.price);
                println!("{}\t{}\t{price_str}", t.name, fmt_id(t.id));
            }
        }
    }
    Ok(())
}

/// Toggles auto-prolongation for a domain.
///
/// # Overview
///
/// Enables or disables automatic domain renewal for the specified domain.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn auto_prolong(
    config: &timeweb_rs::apis::configuration::Configuration,
    fqdn: String,
    enabled: bool,
    format: OutputFormat
) -> Result<(), TwcError> {
    let update = dm::UpdateDomain {
        is_autoprolong_enabled: Some(enabled),
        linked_ip:              None
    };
    let resp = domains_api::update_domain_auto_prolongation(config, &fqdn, update).await?;
    let d = &resp.domain;

    match format {
        OutputFormat::Table => {
            let msg = if enabled {
                t!("cli.domain_autoprolong_enabled", fqdn => d.fqdn, id => fmt_id(d.id))
            } else {
                t!("cli.domain_autoprolong_disabled", fqdn => d.fqdn, id => fmt_id(d.id))
            };
            println!("{msg}");
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.domain)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            let state = if enabled { "on" } else { "off" };
            println!("{fqdn}\t{state}");
        }
    }
    Ok(())
}
