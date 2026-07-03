// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Table row types for the domains command output.

use std::fmt;

use tabled::Tabled;

/// Compact row for the domain list table.
#[derive(Tabled)]
pub(super) struct DomainRow {
    #[tabled(rename = "ID")]
    pub(super) id:           String,
    #[tabled(rename = "FQDN")]
    pub(super) fqdn:         String,
    #[tabled(rename = "Status")]
    pub(super) status:       String,
    #[tabled(rename = "Expires")]
    pub(super) expires:      String,
    #[tabled(rename = "AutoProlong")]
    pub(super) auto_prolong: String,
    #[tabled(rename = "DaysLeft")]
    pub(super) days_left:    String
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
pub(super) struct DnsRecordRow {
    #[tabled(rename = "ID")]
    pub(super) id:       String,
    #[tabled(rename = "Type")]
    pub(super) r#type:   String,
    #[tabled(rename = "Value")]
    pub(super) value:    String,
    #[tabled(rename = "TTL")]
    pub(super) ttl:      String,
    #[tabled(rename = "Priority")]
    pub(super) priority: String
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
pub(super) struct NameServerRow {
    #[tabled(rename = "Host")]
    pub(super) host: String,
    #[tabled(rename = "IPs")]
    pub(super) ips:  String
}

impl fmt::Display for NameServerRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.host, self.ips)
    }
}

/// Compact row for the subdomain table.
#[derive(Tabled)]
pub(super) struct SubdomainRow {
    #[tabled(rename = "ID")]
    pub(super) id:   String,
    #[tabled(rename = "FQDN")]
    pub(super) fqdn: String,
    #[tabled(rename = "IP")]
    pub(super) ip:   String
}

impl fmt::Display for SubdomainRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.id, self.fqdn, self.ip)
    }
}

/// Compact row for the TLD table.
#[derive(Tabled)]
pub(super) struct TldRow {
    #[tabled(rename = "ID")]
    pub(super) id:            String,
    #[tabled(rename = "Name")]
    pub(super) name:          String,
    #[tabled(rename = "Price")]
    pub(super) price:         String,
    #[tabled(rename = "Registrar")]
    pub(super) registrar:     String,
    #[tabled(rename = "Published")]
    pub(super) is_published:  String,
    #[tabled(rename = "Registered")]
    pub(super) is_registered: String
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
pub(super) struct DomainRequestRow {
    #[tabled(rename = "ID")]
    pub(super) id:      String,
    #[tabled(rename = "FQDN")]
    pub(super) fqdn:    String,
    #[tabled(rename = "Type")]
    pub(super) r#type:  String,
    #[tabled(rename = "Date")]
    pub(super) date:    String,
    #[tabled(rename = "Message")]
    pub(super) message: String
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
