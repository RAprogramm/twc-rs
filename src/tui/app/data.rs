// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Applying fetched data snapshots to the dashboard state.

use std::time::Instant;

use rust_i18n::t;

use super::{
    AccountInfo, AiAgentSummary, AppSummary, BalancerSummary, DashboardData, DatabaseSummary,
    DedicatedServerSummary, DomainSummary, FirewallSummary, FloatingIpSummary, ImageSummary,
    K8sSummary, KnowledgeBaseSummary, LogLevel, MailSummary, NetworkDriveSummary, ProjectSummary,
    RegistrySummary, S3Summary, ServerSummary, VpcSummary
};

impl super::App {
    /// Marks that a refresh is needed immediately.
    pub fn force_refresh(&mut self) {
        self.refresh_requested = true;
        self.log(LogLevel::Info, t!("app.log_manual_refresh").to_string());
    }

    /// Returns true when the refresh interval has elapsed.
    #[must_use]
    #[allow(dead_code)]
    pub fn needs_refresh(&self) -> bool {
        self.last_refresh.elapsed() >= self.refresh_interval
    }

    /// Updates account info.
    pub fn update_account(&mut self, info: AccountInfo) {
        self.account = info;
    }

    /// Updates server data.
    pub fn update_servers(&mut self, servers: Vec<ServerSummary>) {
        self.servers = servers;
        self.clamp_selection();
        self.last_refresh = Instant::now();
    }

    /// Updates database data.
    pub fn update_databases(&mut self, databases: Vec<DatabaseSummary>) {
        self.databases = databases;
        self.clamp_selection();
    }

    /// Updates S3 data.
    pub fn update_s3(&mut self, storages: Vec<S3Summary>) {
        self.s3_storages = storages;
        self.clamp_selection();
    }

    /// Updates Kubernetes data.
    pub fn update_k8s(&mut self, clusters: Vec<K8sSummary>) {
        self.k8s_clusters = clusters;
        self.clamp_selection();
    }

    /// Updates project data.
    pub fn update_projects(&mut self, projects: Vec<ProjectSummary>) {
        self.projects = projects;
        self.clamp_selection();
    }

    /// Updates balancer data.
    // JUSTIFY: Public API method for future API integration.
    #[allow(dead_code)]
    pub fn update_balancers(&mut self, balancers: Vec<BalancerSummary>) {
        self.balancers = balancers;
        self.clamp_selection();
    }

    /// Updates registry data.
    // JUSTIFY: Public API method for future API integration.
    #[allow(dead_code)]
    pub fn update_registries(&mut self, registries: Vec<RegistrySummary>) {
        self.registries = registries;
        self.clamp_selection();
    }

    /// Updates domain data.
    // JUSTIFY: Public API method for future API integration.
    #[allow(dead_code)]
    pub fn update_domains(&mut self, domains: Vec<DomainSummary>) {
        self.domains = domains;
        self.clamp_selection();
    }

    /// Updates firewall data.
    // JUSTIFY: Public API method for future API integration.
    #[allow(dead_code)]
    pub fn update_firewalls(&mut self, firewalls: Vec<FirewallSummary>) {
        self.firewalls = firewalls;
        self.clamp_selection();
    }

    /// Updates floating IP data.
    // JUSTIFY: Public API method for future API integration.
    #[allow(dead_code)]
    pub fn update_floating_ips(&mut self, ips: Vec<FloatingIpSummary>) {
        self.floating_ips = ips;
        self.clamp_selection();
    }

    /// Updates image data.
    // JUSTIFY: Public API method for future API integration.
    #[allow(dead_code)]
    pub fn update_images(&mut self, images: Vec<ImageSummary>) {
        self.images = images;
        self.clamp_selection();
    }

    /// Updates network drive data.
    // JUSTIFY: Public API method for future API integration.
    #[allow(dead_code)]
    pub fn update_network_drives(&mut self, drives: Vec<NetworkDriveSummary>) {
        self.network_drives = drives;
        self.clamp_selection();
    }

    /// Updates VPC data.
    // JUSTIFY: Public API method for future API integration.
    #[allow(dead_code)]
    pub fn update_vpcs(&mut self, vpcs: Vec<VpcSummary>) {
        self.vpcs = vpcs;
        self.clamp_selection();
    }

    /// Updates dedicated server data.
    // JUSTIFY: Public API method for future API integration.
    #[allow(dead_code)]
    pub fn update_dedicated_servers(&mut self, servers: Vec<DedicatedServerSummary>) {
        self.dedicated_servers = servers;
        self.clamp_selection();
    }

    /// Updates mail data.
    // JUSTIFY: Public API method for future API integration.
    #[allow(dead_code)]
    pub fn update_mails(&mut self, mails: Vec<MailSummary>) {
        self.mails = mails;
        self.clamp_selection();
    }

    /// Updates application data.
    // JUSTIFY: Public API method for future API integration.
    #[allow(dead_code)]
    pub fn update_apps(&mut self, apps: Vec<AppSummary>) {
        self.apps = apps;
        self.clamp_selection();
    }

    /// Updates AI agent data.
    // JUSTIFY: Public API method for future API integration.
    #[allow(dead_code)]
    pub fn update_ai_agents(&mut self, agents: Vec<AiAgentSummary>) {
        self.ai_agents = agents;
        self.clamp_selection();
    }

    /// Updates knowledge base data.
    // JUSTIFY: Public API method for future API integration.
    #[allow(dead_code)]
    pub fn update_knowledge_bases(&mut self, bases: Vec<KnowledgeBaseSummary>) {
        self.knowledge_bases = bases;
        self.clamp_selection();
    }

    /// Updates SSH key data.
    // JUSTIFY: Public API method for future API integration.
    #[allow(dead_code)]
    pub fn update_ssh_keys(&mut self, keys: Vec<String>) {
        self.ssh_keys = keys;
        self.clamp_selection();
    }

    /// Updates finances data.
    // JUSTIFY: Public API method for future API integration.
    #[allow(dead_code)]
    pub fn update_finances(&mut self, data: Vec<String>) {
        self.finances = data;
        self.clamp_selection();
    }

    /// Applies a freshly fetched data snapshot, replacing all resource
    /// lists in one shot and clearing the loading state.
    pub fn apply_data(&mut self, data: DashboardData) {
        self.account = data.account;
        self.servers = data.servers;
        self.databases = data.databases;
        self.s3_storages = data.s3_storages;
        self.k8s_clusters = data.k8s_clusters;
        self.projects = data.projects;
        self.balancers = data.balancers;
        self.registries = data.registries;
        self.domains = data.domains;
        self.firewalls = data.firewalls;
        self.floating_ips = data.floating_ips;
        self.images = data.images;
        self.network_drives = data.network_drives;
        self.vpcs = data.vpcs;
        self.dedicated_servers = data.dedicated_servers;
        self.mails = data.mails;
        self.apps = data.apps;
        self.ai_agents = data.ai_agents;
        self.knowledge_bases = data.knowledge_bases;
        self.ssh_keys = data.ssh_keys;
        self.finances = data.finances;
        if data.error_message.is_some() {
            self.error_message = data.error_message;
        } else {
            self.error_message = None;
            self.status_message = data.status_message;
        }
        if data.load_errors != self.last_load_errors {
            let recovered: Vec<String> = self
                .last_load_errors
                .iter()
                .filter(|r| !data.load_errors.contains(r))
                .cloned()
                .collect();
            for entry in &data.load_errors {
                self.log(
                    LogLevel::Error,
                    t!("app.log_load_failed", entry => entry).to_string()
                );
            }
            for entry in recovered {
                let name = entry.split(':').next().unwrap_or(&entry);
                self.log(
                    LogLevel::Success,
                    t!("app.log_recovered", name => name).to_string()
                );
            }
            self.last_load_errors = data.load_errors;
        }
        self.clamp_selection();
        self.is_loading = false;
        self.last_refresh = Instant::now();
    }
}
