// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Domain and DNS subcommands.

use clap::Subcommand;

/// Domain management subcommands.
#[derive(Subcommand, Debug)]
pub enum DomainCommands {
    /// List all domains on the account.
    List {
        /// Maximum number of domains to return.
        #[arg(long)]
        limit: Option<i32>,

        /// Number of domains to skip.
        #[arg(long)]
        offset: Option<i32>
    },
    /// Show detailed info for a domain.
    Info {
        /// Domain FQDN (e.g., example.com).
        #[arg(long)]
        id: String
    },
    /// Check if a domain is available for registration.
    Check {
        /// Domain name to check (e.g., example.com).
        #[arg(long)]
        domain: String
    },
    /// Add a domain to the account.
    Add {
        /// Domain FQDN to add (e.g., example.com).
        #[arg(long)]
        domain: String
    },
    /// Delete a domain from the account.
    Delete {
        /// Domain FQDN to delete (e.g., example.com).
        #[arg(long)]
        id: String
    },
    /// List DNS records for a domain.
    DnsList {
        /// Domain FQDN (e.g., example.com).
        #[arg(long)]
        id: String
    },
    /// Add a DNS record to a domain.
    DnsAdd {
        /// Domain FQDN (e.g., example.com).
        #[arg(long)]
        id: String,

        /// DNS record type (A, AAAA, CNAME, MX, TXT, SRV).
        #[arg(long)]
        record_type: String,

        /// DNS record value (e.g., IP address for A record).
        #[arg(long)]
        value: String
    },
    /// Delete a DNS record from a domain.
    DnsDelete {
        /// Domain FQDN (e.g., example.com).
        #[arg(long)]
        id: String,

        /// DNS record ID to delete.
        #[arg(long)]
        record_id: i32
    },
    /// Update a DNS record on a domain.
    DnsUpdate {
        /// Domain FQDN (e.g., example.com).
        #[arg(long)]
        id: String,

        /// DNS record ID to update.
        #[arg(long)]
        record_id: i32,

        /// New DNS record type (A, AAAA, CNAME, MX, TXT, SRV).
        #[arg(long)]
        record_type: String,

        /// New DNS record value.
        #[arg(long)]
        value: String
    },
    /// List name servers for a domain.
    NsList {
        /// Domain FQDN (e.g., example.com).
        #[arg(long)]
        id: String
    },
    /// Update name servers for a domain.
    NsUpdate {
        /// Domain FQDN (e.g., example.com).
        #[arg(long)]
        id: String,

        /// First name server (e.g., ns1.example.com).
        #[arg(short = '1', long)]
        ns1: String,

        /// Second name server (e.g., ns2.example.com).
        #[arg(short = '2', long)]
        ns2: String
    },
    /// List subdomains for a domain.
    SubdomainList {
        /// Domain FQDN (e.g., example.com).
        #[arg(long)]
        id: String
    },
    /// Add a subdomain to a domain.
    SubdomainAdd {
        /// Domain FQDN (e.g., example.com).
        #[arg(long)]
        id: String,

        /// Subdomain name (e.g., www).
        #[arg(long)]
        name: String
    },
    /// Delete a subdomain from a domain.
    SubdomainDelete {
        /// Domain FQDN (e.g., example.com).
        #[arg(long)]
        id: String,

        /// Subdomain name to delete (e.g., www).
        #[arg(long)]
        name: String
    },
    /// List domain registration/transfer/prolongation requests.
    RequestList,
    /// List available TLDs (top-level domains).
    TldList,
    /// Toggle auto-prolongation for a domain.
    AutoProlong {
        /// Domain FQDN (e.g., example.com).
        #[arg(long)]
        id: String,

        /// Enable (true) or disable (false) auto-prolongation.
        #[arg(long)]
        enabled: bool
    }
}
