// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! The resource tab registry: names, indexes and per-tab actions.

use std::borrow::Cow;

use rust_i18n::t;

use super::ActionKind;

/// Resource category in the left panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceTab {
    Servers,
    Databases,
    S3,
    Kubernetes,
    Projects,
    Balancers,
    Registry,
    Domains,
    Firewall,
    FloatingIps,
    Images,
    NetworkDrives,
    Vpc,
    DedicatedServers,
    Mail,
    Apps,
    AiAgents,
    KnowledgeBases,
    // JUSTIFY: Not yet reachable from the sidebar; data still loads and the
    // variants stay for the upcoming account section.
    #[allow(dead_code)]
    SshKeys,
    #[allow(dead_code)]
    Finances
}

impl ResourceTab {
    /// Returns all tab names.
    // JUSTIFY: Public API method for future UI integration.
    #[allow(dead_code)]
    #[must_use]
    pub const fn names() -> &'static [&'static str] {
        &[
            "Servers",
            "Databases",
            "S3",
            "Kubernetes",
            "Projects",
            "Balancers",
            "Registry",
            "Domains",
            "Firewall",
            "FloatingIps",
            "Images",
            "NetworkDrives",
            "Vpc",
            "DedicatedServers",
            "Mail",
            "Apps",
            "AiAgents",
            "KnowledgeBases",
            "SshKeys",
            "Finances"
        ]
    }

    /// Returns the localized display name for this tab.
    #[must_use]
    pub fn display_name(self) -> Cow<'static, str> {
        match self {
            Self::Servers => t!("tabs.servers"),
            Self::Databases => t!("tabs.databases"),
            Self::S3 => t!("tabs.s3"),
            Self::Kubernetes => t!("tabs.kubernetes"),
            Self::Projects => t!("tabs.projects"),
            Self::Balancers => t!("tabs.balancers"),
            Self::Registry => t!("tabs.registry"),
            Self::Domains => t!("tabs.domains"),
            Self::Firewall => t!("tabs.firewall"),
            Self::FloatingIps => t!("tabs.floating_ips"),
            Self::Images => t!("tabs.images"),
            Self::NetworkDrives => t!("tabs.network_drives"),
            Self::Vpc => t!("tabs.vpc"),
            Self::DedicatedServers => t!("tabs.dedicated_servers"),
            Self::Mail => t!("tabs.mail"),
            Self::Apps => t!("tabs.apps"),
            Self::AiAgents => t!("tabs.ai_agents"),
            Self::KnowledgeBases => t!("tabs.knowledge_bases"),
            Self::SshKeys => t!("tabs.ssh_keys"),
            Self::Finances => t!("tabs.finances")
        }
    }

    /// Returns the index of this tab.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Servers => 0,
            Self::Databases => 1,
            Self::S3 => 2,
            Self::Kubernetes => 3,
            Self::Projects => 4,
            Self::Balancers => 5,
            Self::Registry => 6,
            Self::Domains => 7,
            Self::Firewall => 8,
            Self::FloatingIps => 9,
            Self::Images => 10,
            Self::NetworkDrives => 11,
            Self::Vpc => 12,
            Self::DedicatedServers => 13,
            Self::Mail => 14,
            Self::Apps => 15,
            Self::AiAgents => 16,
            Self::KnowledgeBases => 17,
            Self::SshKeys => 18,
            Self::Finances => 19
        }
    }

    /// Returns the context actions available for this resource type,
    /// in menu display order.
    ///
    /// Tabs without wired actions return an empty slice, so the action
    /// menu does not open for them.
    #[must_use]
    pub const fn actions(self) -> &'static [ActionKind] {
        use ActionKind::{Backup, Clone, Delete, Reboot, Shutdown, Start};
        match self {
            Self::Servers => &[Start, Shutdown, Reboot, Clone, Delete],
            Self::Databases => &[Backup, Delete],
            Self::S3
            | Self::Kubernetes
            | Self::Balancers
            | Self::Registry
            | Self::Projects
            | Self::DedicatedServers
            | Self::AiAgents
            | Self::KnowledgeBases
            | Self::Apps
            | Self::Domains
            | Self::Firewall
            | Self::FloatingIps
            | Self::Images
            | Self::NetworkDrives
            | Self::Vpc => &[Delete],
            _ => &[]
        }
    }

    /// All tabs, in display order.
    // JUSTIFY: Retained for tests and future full-tab iteration.
    #[allow(dead_code)]
    pub const ALL: [Self; 20] = [
        Self::Servers,
        Self::Databases,
        Self::S3,
        Self::Kubernetes,
        Self::Projects,
        Self::Balancers,
        Self::Registry,
        Self::Domains,
        Self::Firewall,
        Self::FloatingIps,
        Self::Images,
        Self::NetworkDrives,
        Self::Vpc,
        Self::DedicatedServers,
        Self::Mail,
        Self::Apps,
        Self::AiAgents,
        Self::KnowledgeBases,
        Self::SshKeys,
        Self::Finances
    ];
}
