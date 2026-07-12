// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Per-resource summary view-models and the full dashboard data snapshot.

/// Severity of an event-log entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum LogLevel {
    /// Neutral informational event.
    Info,
    /// A successful outcome.
    Success,
    /// A non-fatal warning.
    Warn,
    /// A failure.
    Error
}

/// A single entry in the dashboard event log.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Severity of the entry.
    pub level: LogLevel,
    /// Human-readable message.
    pub text:  String
}

/// Account information from the API.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AccountInfo {
    pub login:      String,
    pub account_id: i64,
    pub balance:    String,
    pub status:     String
}

/// Summary of a single server.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[allow(dead_code)]
pub struct ServerSummary {
    pub id:       i32,
    pub name:     String,
    pub status:   String,
    pub cpu:      i32,
    pub ram_mb:   i32,
    pub disk_gb:  i32,
    pub ip:       String,
    pub location: String
}

/// Summary of a single database cluster, carrying every field the list
/// endpoint exposes (the password is deliberately not kept: summaries are
/// persisted to the on-disk snapshot).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DatabaseSummary {
    pub id:           i32,
    pub name:         String,
    pub status:       String,
    pub engine:       String,
    pub size_mb:      i64,
    #[serde(default)]
    pub disk_used_mb: i64,
    #[serde(default)]
    pub created_at:   String,
    #[serde(default)]
    pub location:     String,
    #[serde(default)]
    pub port:         i32,
    #[serde(default)]
    pub public_ip:    String,
    #[serde(default)]
    pub local_ip:     String,
    #[serde(default)]
    pub preset_id:    i32,
    #[serde(default)]
    pub hash_type:    String,
    #[serde(default)]
    pub local_only:   bool,
    #[serde(default)]
    pub config:       Vec<(String, String)>
}

/// Summary of a single S3 storage.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct S3Summary {
    pub id:           i32,
    pub name:         String,
    pub region:       String,
    pub size_kb:      i64,
    pub object_count: i64
}

/// Summary of a single Kubernetes cluster.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct K8sSummary {
    pub id:      i32,
    pub name:    String,
    pub status:  String,
    pub version: String,
    pub cpu:     i32,
    pub ram_mb:  i32,
    pub disk_gb: i32
}

/// Summary of a single project with per-type resource counts.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ProjectSummary {
    pub id:              i32,
    pub name:            String,
    pub server_count:    i32,
    pub database_count:  i32,
    pub bucket_count:    i32,
    pub cluster_count:   i32,
    pub balancer_count:  i32,
    pub dedicated_count: i32,
    pub app_count:       i32
}

impl ProjectSummary {
    /// Returns the total number of resources across all types.
    #[must_use]
    pub const fn resource_count(&self) -> i32 {
        self.server_count
            + self.database_count
            + self.bucket_count
            + self.cluster_count
            + self.balancer_count
            + self.dedicated_count
            + self.app_count
    }
}

/// Summary of a single load balancer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct BalancerSummary {
    pub id:       i32,
    pub name:     String,
    pub status:   String,
    pub ip:       String,
    pub location: String
}

/// Summary of a single container registry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct RegistrySummary {
    pub id:        i32,
    pub name:      String,
    pub disk_used: i64,
    pub disk_size: i64
}

/// Summary of a single domain.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct DomainSummary {
    pub id:           i32,
    pub name:         String,
    pub status:       String,
    pub auto_prolong: bool
}

/// Summary of a single firewall.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct FirewallSummary {
    pub id:     String,
    pub name:   String,
    pub policy: String
}

/// Summary of a single floating IP.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct FloatingIpSummary {
    pub id:          String,
    pub ip:          String,
    pub status:      String,
    pub server_name: String
}

/// Summary of a single image.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct ImageSummary {
    pub id:      String,
    pub name:    String,
    pub size_mb: i64,
    pub status:  String
}

/// Summary of a single network drive.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct NetworkDriveSummary {
    pub id:      String,
    pub name:    String,
    pub size_gb: i64,
    pub status:  String
}

/// Summary of a single VPC.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct VpcSummary {
    pub id:       String,
    pub name:     String,
    pub subnet:   String,
    pub location: String
}

/// Summary of a single dedicated server.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct DedicatedServerSummary {
    pub id:     i32,
    pub name:   String,
    pub status: String,
    pub cpu:    String,
    pub ram:    String,
    pub disk:   String,
    pub ip:     String
}

/// Summary of a single mail service.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct MailSummary {
    pub name:    String,
    pub owner:   String,
    pub comment: String
}

/// Summary of a single application.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct AppSummary {
    pub id:          i32,
    pub name:        String,
    pub status:      String,
    pub ip:          String,
    pub location:    String,
    pub app_type:    String,
    pub framework:   String,
    pub language:    String,
    pub branch:      String,
    pub commit:      String,
    pub auto_deploy: bool,
    pub comment:     String,
    pub domains:     Vec<String>
}

/// Summary of a single AI agent.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct AiAgentSummary {
    pub id:           i32,
    pub name:         String,
    pub status:       String,
    pub tokens_used:  i64,
    pub tokens_total: i64
}

/// Summary of a single knowledge base.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct KnowledgeBaseSummary {
    pub id:             i32,
    pub name:           String,
    pub document_count: i32,
    pub status:         String
}

/// An owned snapshot of all dashboard data, applied in one shot via
/// `App::apply_data` and persisted between runs for instant startup.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DashboardData {
    pub account:           AccountInfo,
    pub servers:           Vec<ServerSummary>,
    pub databases:         Vec<DatabaseSummary>,
    pub s3_storages:       Vec<S3Summary>,
    pub k8s_clusters:      Vec<K8sSummary>,
    pub projects:          Vec<ProjectSummary>,
    pub balancers:         Vec<BalancerSummary>,
    pub registries:        Vec<RegistrySummary>,
    pub domains:           Vec<DomainSummary>,
    pub firewalls:         Vec<FirewallSummary>,
    pub floating_ips:      Vec<FloatingIpSummary>,
    pub images:            Vec<ImageSummary>,
    pub network_drives:    Vec<NetworkDriveSummary>,
    pub vpcs:              Vec<VpcSummary>,
    pub dedicated_servers: Vec<DedicatedServerSummary>,
    pub mails:             Vec<MailSummary>,
    pub apps:              Vec<AppSummary>,
    pub ai_agents:         Vec<AiAgentSummary>,
    pub knowledge_bases:   Vec<KnowledgeBaseSummary>,
    pub ssh_keys:          Vec<String>,
    pub finances:          Vec<String>,
    pub error_message:     Option<String>,
    pub status_message:    Option<String>,
    pub load_errors:       Vec<String>
}

impl DashboardData {
    /// Clones the resource data out of a populated `App`, for persisting the
    /// startup snapshot.
    #[must_use]
    pub fn from_app(app: &super::App) -> Self {
        Self {
            account:           app.account.clone(),
            servers:           app.servers.clone(),
            databases:         app.databases.clone(),
            s3_storages:       app.s3_storages.clone(),
            k8s_clusters:      app.k8s_clusters.clone(),
            projects:          app.projects.clone(),
            balancers:         app.balancers.clone(),
            registries:        app.registries.clone(),
            domains:           app.domains.clone(),
            firewalls:         app.firewalls.clone(),
            floating_ips:      app.floating_ips.clone(),
            images:            app.images.clone(),
            network_drives:    app.network_drives.clone(),
            vpcs:              app.vpcs.clone(),
            dedicated_servers: app.dedicated_servers.clone(),
            mails:             app.mails.clone(),
            apps:              app.apps.clone(),
            ai_agents:         app.ai_agents.clone(),
            knowledge_bases:   app.knowledge_bases.clone(),
            ssh_keys:          app.ssh_keys.clone(),
            finances:          app.finances.clone(),
            error_message:     None,
            status_message:    None,
            load_errors:       Vec::new()
        }
    }
}
