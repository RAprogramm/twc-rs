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

/// One independently loaded piece of dashboard data, streamed to the UI as
/// soon as its endpoint responds so fast resources paint before slow ones.
#[derive(Debug, Clone)]
pub enum DataSlice {
    Account(AccountInfo),
    Servers(Vec<ServerSummary>),
    Databases(Vec<DatabaseSummary>),
    S3(Vec<S3Summary>),
    K8s(Vec<K8sSummary>),
    Projects(Vec<ProjectSummary>),
    Balancers(Vec<BalancerSummary>),
    Registries(Vec<RegistrySummary>),
    Domains(Vec<DomainSummary>),
    Firewalls(Vec<FirewallSummary>),
    FloatingIps(Vec<FloatingIpSummary>),
    Images(Vec<ImageSummary>),
    NetworkDrives(Vec<NetworkDriveSummary>),
    Vpcs(Vec<VpcSummary>),
    DedicatedServers(Vec<DedicatedServerSummary>),
    Mails(Vec<MailSummary>),
    Apps(Vec<AppSummary>),
    AiAgents(Vec<AiAgentSummary>),
    KnowledgeBases(Vec<KnowledgeBaseSummary>),
    SshKeys(Vec<String>),
    Finances {
        balance: String,
        lines:   Vec<String>
    },
    /// A named endpoint failed: `"servers: <message>"`.
    Error(String)
}

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

    /// Begins a streamed load cycle: forgets the previous cycle's errors.
    pub fn load_started(&mut self) {
        self.cycle_load_errors.clear();
    }

    /// Applies one streamed slice the moment it arrives, so the UI shows
    /// each resource as soon as its endpoint responds.
    pub fn apply_slice(&mut self, slice: DataSlice) {
        match slice {
            DataSlice::Account(info) => self.account = info,
            DataSlice::Servers(v) => {
                self.servers = v;
                self.last_refresh = Instant::now();
            }
            DataSlice::Databases(v) => self.databases = v,
            DataSlice::S3(v) => self.s3_storages = v,
            DataSlice::K8s(v) => self.k8s_clusters = v,
            DataSlice::Projects(v) => self.projects = v,
            DataSlice::Balancers(v) => self.balancers = v,
            DataSlice::Registries(v) => self.registries = v,
            DataSlice::Domains(v) => self.domains = v,
            DataSlice::Firewalls(v) => self.firewalls = v,
            DataSlice::FloatingIps(v) => self.floating_ips = v,
            DataSlice::Images(v) => self.images = v,
            DataSlice::NetworkDrives(v) => self.network_drives = v,
            DataSlice::Vpcs(v) => self.vpcs = v,
            DataSlice::DedicatedServers(v) => self.dedicated_servers = v,
            DataSlice::Mails(v) => self.mails = v,
            DataSlice::Apps(v) => self.apps = v,
            DataSlice::AiAgents(v) => self.ai_agents = v,
            DataSlice::KnowledgeBases(v) => self.knowledge_bases = v,
            DataSlice::SshKeys(v) => self.ssh_keys = v,
            DataSlice::Finances {
                balance,
                lines
            } => {
                self.account.balance = balance;
                self.finances = lines;
            }
            DataSlice::Error(entry) => {
                if !self.last_load_errors.contains(&entry) {
                    self.log(
                        LogLevel::Error,
                        t!("app.log_load_failed", entry => entry).to_string()
                    );
                }
                self.cycle_load_errors.push(entry);
                return;
            }
        }
        self.is_loading = false;
        self.select_initial_tab();
        self.clamp_selection();
    }

    /// Finishes a streamed load cycle: logs recoveries, rolls the error set
    /// over, and sets the summary status message.
    pub fn load_finished(&mut self) {
        let recovered: Vec<String> = self
            .last_load_errors
            .iter()
            .filter(|e| !self.cycle_load_errors.contains(e))
            .cloned()
            .collect();
        for entry in recovered {
            let name = entry.split(':').next().unwrap_or(&entry).to_string();
            self.log(
                LogLevel::Success,
                t!("app.log_recovered", name => name).to_string()
            );
        }
        self.last_load_errors = self.cycle_load_errors.clone();
        self.is_loading = false;
        if self.last_load_errors.is_empty() {
            self.error_message = None;
            self.status_message = Some(t!("app.load_ok").to_string());
        } else {
            self.error_message =
                Some(t!("app.load_failures", n => self.last_load_errors.len()).to_string());
        }
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
        self.select_initial_tab();
        self.clamp_selection();
        self.is_loading = false;
        self.last_refresh = Instant::now();
    }
}
