// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Application state for the TUI dashboard.

use std::{
    borrow::Cow,
    collections::VecDeque,
    time::{Duration, Instant}
};

use rust_i18n::t;

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
#[derive(Debug, Clone, Default)]
pub struct AccountInfo {
    pub login:      String,
    pub account_id: i64,
    pub balance:    String,
    pub status:     String
}

/// Summary of a single server.
#[derive(Debug, Clone)]
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

/// Summary of a single database.
#[derive(Debug, Clone)]
pub struct DatabaseSummary {
    pub id:      i32,
    pub name:    String,
    pub status:  String,
    pub engine:  String,
    pub size_mb: i64
}

/// Summary of a single S3 storage.
#[derive(Debug, Clone)]
pub struct S3Summary {
    pub id:           i32,
    pub name:         String,
    pub region:       String,
    pub size_bytes:   i64,
    pub bucket_count: i32
}

/// Summary of a single Kubernetes cluster.
#[derive(Debug, Clone)]
pub struct K8sSummary {
    pub id:         i32,
    pub name:       String,
    pub status:     String,
    pub version:    String,
    pub node_count: i32
}

/// Summary of a single project.
#[derive(Debug, Clone)]
pub struct ProjectSummary {
    pub id:           i32,
    pub name:         String,
    pub server_count: i32
}

/// Summary of a single load balancer.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct RegistrySummary {
    pub id:               i32,
    pub name:             String,
    pub region:           String,
    pub repository_count: i32
}

/// Summary of a single domain.
#[derive(Debug, Clone)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct DomainSummary {
    pub id:           i32,
    pub name:         String,
    pub status:       String,
    pub auto_prolong: bool
}

/// Summary of a single firewall.
#[derive(Debug, Clone)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct FirewallSummary {
    pub id:             i32,
    pub name:           String,
    pub rule_count:     i32,
    pub resource_count: i32
}

/// Summary of a single floating IP.
#[derive(Debug, Clone)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct FloatingIpSummary {
    pub id:          i32,
    pub ip:          String,
    pub status:      String,
    pub server_name: String
}

/// Summary of a single image.
#[derive(Debug, Clone)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct ImageSummary {
    pub id:      i32,
    pub name:    String,
    pub size_mb: i64,
    pub status:  String
}

/// Summary of a single network drive.
#[derive(Debug, Clone)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct NetworkDriveSummary {
    pub id:      i32,
    pub name:    String,
    pub size_gb: i64,
    pub status:  String
}

/// Summary of a single VPC.
#[derive(Debug, Clone)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct VpcSummary {
    pub id:           i32,
    pub name:         String,
    pub subnet_count: i32,
    pub status:       String
}

/// Summary of a single dedicated server.
#[derive(Debug, Clone)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct DedicatedServerSummary {
    pub id:      i32,
    pub name:    String,
    pub status:  String,
    pub cpu:     i32,
    pub ram_mb:  i32,
    pub disk_gb: i64
}

/// Summary of a single mail service.
#[derive(Debug, Clone)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct MailSummary {
    pub id:            i32,
    pub name:          String,
    pub mailbox_count: i32,
    pub status:        String
}

/// Summary of a single application.
#[derive(Debug, Clone)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct AppSummary {
    pub id:           i32,
    pub name:         String,
    pub status:       String,
    pub deploy_count: i32
}

/// Summary of a single AI agent.
#[derive(Debug, Clone)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct AiAgentSummary {
    pub id:     i32,
    pub name:   String,
    pub status: String,
    pub model:  String
}

/// Summary of a single knowledge base.
#[derive(Debug, Clone)]
// JUSTIFY: Public API type for future API integration.
#[allow(dead_code)]
pub struct KnowledgeBaseSummary {
    pub id:             i32,
    pub name:           String,
    pub document_count: i32,
    pub status:         String
}

/// A kind of action that can be performed on a resource.
///
/// Each [`ResourceTab`] declares which kinds apply to it via
/// [`ResourceTab::actions`]; the dashboard loop maps a chosen
/// `(tab, kind)` pair to the matching Timeweb API call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    /// Power the resource on.
    Start,
    /// Gracefully power the resource off.
    Shutdown,
    /// Reboot the resource.
    Reboot,
    /// Create a clone of the resource.
    Clone,
    /// Permanently delete the resource.
    Delete
}

impl ActionKind {
    /// Returns the label shown in the action menu and confirmation prompt.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Start => "Start",
            Self::Shutdown => "Shutdown",
            Self::Reboot => "Reboot",
            Self::Clone => "Clone",
            Self::Delete => "Delete"
        }
    }

    /// Returns the localized label shown in menus and confirmation prompts.
    #[must_use]
    pub fn display_label(self) -> Cow<'static, str> {
        match self {
            Self::Start => t!("app.action_start"),
            Self::Shutdown => t!("app.action_shutdown"),
            Self::Reboot => t!("app.action_reboot"),
            Self::Clone => t!("app.action_clone"),
            Self::Delete => t!("app.action_delete")
        }
    }

    /// Returns true when the action is destructive and irreversible.
    ///
    /// Destructive actions require an extra confirmation step.
    #[must_use]
    pub const fn is_destructive(self) -> bool {
        matches!(self, Self::Delete)
    }
}

/// A resource action awaiting confirmation, or ready to be dispatched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingAction {
    /// The resource category the action targets.
    pub tab:           ResourceTab,
    /// The action to perform.
    pub kind:          ActionKind,
    /// Target resource id.
    pub resource_id:   i32,
    /// Target resource name, for display.
    pub resource_name: String
}

/// A single row inside a drill-in view (a resource contained in a parent).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrillItem {
    /// Resource kind label (e.g. "Server", "Database").
    pub kind:   String,
    /// Resource name.
    pub name:   String,
    /// Short secondary detail (status, engine, ...).
    pub detail: String
}

/// A drill-in view showing the contents of a selected resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrillView {
    /// Title describing what was drilled into.
    pub title:    String,
    /// Contained resources.
    pub items:    Vec<DrillItem>,
    /// Index of the highlighted row.
    pub selected: usize
}

/// A request to fetch live hardware statistics for a resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatsRequest {
    /// The resource category (only
    /// [`ResourceTab::Servers`]/[`ResourceTab::Apps`] expose live
    /// statistics).
    pub tab:  ResourceTab,
    /// The resource id as a string (servers and apps use different id types).
    pub id:   String,
    /// Resource name, shown as the metrics-panel subject.
    pub name: String
}

/// Live hardware statistics time-series for the selected resource.
#[derive(Debug, Clone, Default)]
pub struct ResourceStats {
    /// The resource id these statistics belong to.
    pub id:      String,
    /// Resource name shown in the metrics-panel title.
    pub subject: String,
    /// CPU load percentage over time.
    pub cpu:     Vec<f64>,
    /// RAM usage percentage over time.
    pub ram:     Vec<f64>,
    /// Incoming network bytes over time.
    pub net_in:  Vec<f64>,
    /// Outgoing network bytes over time.
    pub net_out: Vec<f64>
}

/// A context action menu opened over the selected resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionMenu {
    /// The resource category the menu targets.
    pub tab:           ResourceTab,
    /// Target resource id.
    pub resource_id:   i32,
    /// Target resource name, for display.
    pub resource_name: String,
    /// Available actions, in display order.
    pub actions:       Vec<ActionKind>,
    /// Index of the highlighted action.
    pub selected:      usize
}

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
    SshKeys,
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
        use ActionKind::{Clone, Delete, Reboot, Shutdown, Start};
        match self {
            Self::Servers => &[Start, Shutdown, Reboot, Clone, Delete],
            Self::Databases
            | Self::S3
            | Self::Kubernetes
            | Self::Balancers
            | Self::Registry
            | Self::Projects
            | Self::DedicatedServers
            | Self::AiAgents
            | Self::KnowledgeBases
            | Self::Apps => &[Delete],
            _ => &[]
        }
    }

    /// All tabs, in display order.
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
/// Which panel is currently highlighted.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Focus {
    /// Resource tabs bar at the top.
    ResourceTabs,
    /// Resource list panel (left side).
    #[default]
    ResourceList,
    /// Details panel (right side).
    Details
}

/// Holds all runtime state for the TUI dashboard.
#[allow(clippy::struct_excessive_bools)]
pub struct App {
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
    pub selected:          usize,
    pub active_tab:        ResourceTab,
    pub theme:             super::themes::Theme,
    pub token:             Option<String>,
    pub cpu_history:       VecDeque<f64>,
    pub ram_history:       VecDeque<f64>,
    pub net_in_history:    VecDeque<f64>,
    pub net_out_history:   VecDeque<f64>,
    pub last_refresh:      Instant,
    pub refresh_interval:  Duration,
    pub running:           bool,
    pub show_help:         bool,
    pub status_message:    Option<String>,
    pub error_message:     Option<String>,
    pub is_loading:        bool,
    pub widgets:           super::widgets::WidgetRegistry,
    pub focus:             Focus,
    pub action_menu:       Option<ActionMenu>,
    pub confirm:           Option<PendingAction>,
    pub dispatch:          Option<PendingAction>,
    pub palette:           Option<super::command_palette::CommandPalette>,
    pub list_width_pct:    u16,
    pub anim_tick:         u64,
    pub prefs_dirty:       bool,
    pub logs:              VecDeque<LogEntry>,
    pub last_load_errors:  Vec<String>,
    pub refresh_requested: bool,
    pub drill:             Option<DrillView>,
    pub drill_request:     Option<(ResourceTab, i32, String)>,
    pub filter:            String,
    pub filter_editing:    bool,
    pub hide_empty_tabs:   bool,
    pub language:          crate::config::Language,
    pub stats_subject:     Option<String>,
    pub stats_loaded_for:  Option<String>
}

impl App {
    /// Creates a new `App` with default state.
    // JUSTIFY: Used in tests and as a convenience constructor.
    #[allow(dead_code)]
    #[must_use]
    pub fn new(refresh_secs: u64) -> Self {
        Self::new_with_theme(refresh_secs, super::themes::Theme::default(), None)
    }

    /// Creates a new `App` with a specific theme and optional token.
    #[must_use]
    pub fn new_with_theme(
        refresh_secs: u64,
        theme: super::themes::Theme,
        token: Option<String>
    ) -> Self {
        Self {
            account: AccountInfo::default(),
            servers: Vec::new(),
            databases: Vec::new(),
            s3_storages: Vec::new(),
            k8s_clusters: Vec::new(),
            projects: Vec::new(),
            balancers: Vec::new(),
            registries: Vec::new(),
            domains: Vec::new(),
            firewalls: Vec::new(),
            floating_ips: Vec::new(),
            images: Vec::new(),
            network_drives: Vec::new(),
            vpcs: Vec::new(),
            dedicated_servers: Vec::new(),
            mails: Vec::new(),
            apps: Vec::new(),
            ai_agents: Vec::new(),
            knowledge_bases: Vec::new(),
            ssh_keys: Vec::new(),
            finances: Vec::new(),
            selected: 0,
            active_tab: ResourceTab::Servers,
            theme,
            token,
            cpu_history: VecDeque::with_capacity(60),
            ram_history: VecDeque::with_capacity(60),
            net_in_history: VecDeque::with_capacity(60),
            net_out_history: VecDeque::with_capacity(60),
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(refresh_secs),
            running: true,
            show_help: false,
            status_message: None,
            error_message: None,
            is_loading: false,
            widgets: super::widgets::WidgetRegistry::new(),
            focus: Focus::ResourceList,
            action_menu: None,
            confirm: None,
            dispatch: None,
            palette: None,
            list_width_pct: 40,
            anim_tick: 0,
            prefs_dirty: false,
            logs: VecDeque::with_capacity(200),
            last_load_errors: Vec::new(),
            refresh_requested: false,
            drill: None,
            drill_request: None,
            filter: String::new(),
            filter_editing: false,
            hide_empty_tabs: false,
            language: crate::config::Language::default(),
            stats_subject: None,
            stats_loaded_for: None
        }
    }

    /// Polls the selected resource for live statistics.
    ///
    /// Returns a [`StatsRequest`] when the selected resource changed and
    /// exposes live statistics (servers and apps). Returns `None` when
    /// nothing changed; clears the metrics panel when the selection moved
    /// to a resource without live statistics.
    pub fn poll_stats_request(&mut self) -> Option<StatsRequest> {
        let idx = self.selected_real_index();
        let target = match self.active_tab {
            ResourceTab::Servers => self
                .servers
                .get(idx)
                .map(|s| (s.id.to_string(), s.name.clone())),
            ResourceTab::Apps => self
                .apps
                .get(idx)
                .map(|a| (a.id.to_string(), a.name.clone())),
            _ => None
        };

        let target_id = target.as_ref().map(|(id, _)| id.clone());
        if target_id == self.stats_loaded_for {
            return None;
        }
        self.stats_loaded_for = target_id;

        if let Some((id, name)) = target {
            Some(StatsRequest {
                tab: self.active_tab,
                id,
                name
            })
        } else {
            self.clear_stats();
            None
        }
    }

    /// Clears the metrics panel.
    fn clear_stats(&mut self) {
        self.stats_subject = None;
        self.cpu_history.clear();
        self.ram_history.clear();
        self.net_in_history.clear();
        self.net_out_history.clear();
    }

    /// Applies fetched statistics, ignoring stale results whose resource is no
    /// longer selected.
    pub fn apply_stats(&mut self, stats: ResourceStats) {
        if self.stats_loaded_for.as_deref() != Some(stats.id.as_str()) {
            return;
        }
        self.stats_subject = Some(stats.subject);
        self.cpu_history = stats.cpu.into_iter().collect();
        self.ram_history = stats.ram.into_iter().collect();
        self.net_in_history = stats.net_in.into_iter().collect();
        self.net_out_history = stats.net_out.into_iter().collect();
    }

    /// Sets the UI language, applies it live, and marks preferences dirty.
    pub fn set_language(&mut self, language: crate::config::Language) {
        self.language = language;
        rust_i18n::set_locale(language.locale());
        self.prefs_dirty = true;
    }

    /// Returns the number of items currently loaded for the given tab.
    #[must_use]
    pub const fn tab_count(&self, tab: ResourceTab) -> usize {
        match tab {
            ResourceTab::Servers => self.servers.len(),
            ResourceTab::Databases => self.databases.len(),
            ResourceTab::S3 => self.s3_storages.len(),
            ResourceTab::Kubernetes => self.k8s_clusters.len(),
            ResourceTab::Projects => self.projects.len(),
            ResourceTab::Balancers => self.balancers.len(),
            ResourceTab::Registry => self.registries.len(),
            ResourceTab::Domains => self.domains.len(),
            ResourceTab::Firewall => self.firewalls.len(),
            ResourceTab::FloatingIps => self.floating_ips.len(),
            ResourceTab::Images => self.images.len(),
            ResourceTab::NetworkDrives => self.network_drives.len(),
            ResourceTab::Vpc => self.vpcs.len(),
            ResourceTab::DedicatedServers => self.dedicated_servers.len(),
            ResourceTab::Mail => self.mails.len(),
            ResourceTab::Apps => self.apps.len(),
            ResourceTab::AiAgents => self.ai_agents.len(),
            ResourceTab::KnowledgeBases => self.knowledge_bases.len(),
            ResourceTab::SshKeys => self.ssh_keys.len(),
            ResourceTab::Finances => self.finances.len()
        }
    }

    /// Returns the tabs to display: all tabs, or only non-empty ones (plus the
    /// active tab) when empty tabs are hidden.
    #[must_use]
    pub fn visible_tabs(&self) -> Vec<ResourceTab> {
        if !self.hide_empty_tabs {
            return ResourceTab::ALL.to_vec();
        }
        let mut tabs: Vec<ResourceTab> = ResourceTab::ALL
            .into_iter()
            .filter(|t| self.tab_count(*t) > 0 || *t == self.active_tab)
            .collect();
        if tabs.is_empty() {
            tabs.push(self.active_tab);
        }
        tabs
    }

    /// Toggles hiding of empty tabs and marks preferences dirty.
    pub const fn toggle_hide_empty_tabs(&mut self) {
        self.hide_empty_tabs = !self.hide_empty_tabs;
        self.prefs_dirty = true;
    }

    /// Returns the display names of the current tab's items, in list order.
    #[must_use]
    pub fn current_item_names(&self) -> Vec<String> {
        match self.active_tab {
            ResourceTab::Servers => self.servers.iter().map(|s| s.name.clone()).collect(),
            ResourceTab::Databases => self.databases.iter().map(|d| d.name.clone()).collect(),
            ResourceTab::S3 => self.s3_storages.iter().map(|s| s.name.clone()).collect(),
            ResourceTab::Kubernetes => self.k8s_clusters.iter().map(|c| c.name.clone()).collect(),
            ResourceTab::Projects => self.projects.iter().map(|p| p.name.clone()).collect(),
            ResourceTab::Balancers => self.balancers.iter().map(|b| b.name.clone()).collect(),
            ResourceTab::Registry => self.registries.iter().map(|r| r.name.clone()).collect(),
            ResourceTab::Domains => self.domains.iter().map(|d| d.name.clone()).collect(),
            ResourceTab::Firewall => self.firewalls.iter().map(|f| f.name.clone()).collect(),
            ResourceTab::FloatingIps => self.floating_ips.iter().map(|f| f.ip.clone()).collect(),
            ResourceTab::Images => self.images.iter().map(|i| i.name.clone()).collect(),
            ResourceTab::NetworkDrives => {
                self.network_drives.iter().map(|n| n.name.clone()).collect()
            }
            ResourceTab::Vpc => self.vpcs.iter().map(|v| v.name.clone()).collect(),
            ResourceTab::DedicatedServers => self
                .dedicated_servers
                .iter()
                .map(|d| d.name.clone())
                .collect(),
            ResourceTab::Mail => self.mails.iter().map(|m| m.name.clone()).collect(),
            ResourceTab::Apps => self.apps.iter().map(|a| a.name.clone()).collect(),
            ResourceTab::AiAgents => self.ai_agents.iter().map(|a| a.name.clone()).collect(),
            ResourceTab::KnowledgeBases => self
                .knowledge_bases
                .iter()
                .map(|k| k.name.clone())
                .collect(),
            ResourceTab::SshKeys => self.ssh_keys.clone(),
            ResourceTab::Finances => self.finances.clone()
        }
    }

    /// Returns the indices of the current tab's items that match the filter,
    /// in list order. With no filter, returns every index.
    #[must_use]
    pub fn filtered_indices(&self) -> Vec<usize> {
        let names = self.current_item_names();
        if self.filter.is_empty() {
            return (0..names.len()).collect();
        }
        let needle = self.filter.to_lowercase();
        names
            .iter()
            .enumerate()
            .filter(|(_, name)| name.to_lowercase().contains(&needle))
            .map(|(i, _)| i)
            .collect()
    }

    /// Begins filter input for the current list.
    pub const fn start_filter(&mut self) {
        self.filter_editing = true;
        self.selected = 0;
    }

    /// Appends a character to the filter query.
    pub fn filter_push(&mut self, c: char) {
        self.filter.push(c);
        self.selected = 0;
    }

    /// Removes the last filter character; clears the filter when empty.
    pub fn filter_backspace(&mut self) {
        self.filter.pop();
        self.selected = 0;
    }

    /// Applies the filter and leaves input mode (keeps it active for nav).
    pub const fn filter_apply(&mut self) {
        self.filter_editing = false;
    }

    /// Clears the filter entirely and leaves input mode.
    pub fn filter_clear(&mut self) {
        self.filter.clear();
        self.filter_editing = false;
        self.selected = 0;
    }

    /// Returns true when the filter is being typed or is applied.
    #[must_use]
    pub const fn filter_active(&self) -> bool {
        self.filter_editing || !self.filter.is_empty()
    }

    /// Maps the visible selection to the real index into the unfiltered list.
    #[must_use]
    pub fn selected_real_index(&self) -> usize {
        self.filtered_indices()
            .get(self.selected)
            .copied()
            .unwrap_or(0)
    }

    /// Appends an entry to the event log, trimming to the last 200 entries.
    pub fn log(&mut self, level: LogLevel, text: impl Into<String>) {
        if self.logs.len() >= 200 {
            self.logs.pop_front();
        }
        self.logs.push_back(LogEntry {
            level,
            text: text.into()
        });
    }

    /// Returns the currently selected resource list length.
    #[must_use]
    pub fn current_list_len(&self) -> usize {
        self.filtered_indices().len()
    }

    /// Moves selection up.
    pub const fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Moves selection down.
    pub fn select_next(&mut self) {
        if self.selected + 1 < self.current_list_len() {
            self.selected += 1;
        }
    }

    /// Cycles to the next visible resource tab, resetting any filter.
    pub fn next_tab(&mut self) {
        let tabs = self.visible_tabs();
        let pos = tabs.iter().position(|t| *t == self.active_tab).unwrap_or(0);
        self.active_tab = tabs[(pos + 1) % tabs.len()];
        self.reset_after_tab_change();
    }

    /// Cycles to the previous visible resource tab, resetting any filter.
    pub fn previous_tab(&mut self) {
        let tabs = self.visible_tabs();
        let pos = tabs.iter().position(|t| *t == self.active_tab).unwrap_or(0);
        self.active_tab = tabs[(pos + tabs.len() - 1) % tabs.len()];
        self.reset_after_tab_change();
    }

    fn reset_after_tab_change(&mut self) {
        self.selected = 0;
        self.filter.clear();
        self.filter_editing = false;
    }

    /// Toggles the help overlay.
    pub const fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

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

    fn clamp_selection(&mut self) {
        let len = self.current_list_len();
        if len == 0 {
            self.selected = 0;
        } else if self.selected >= len {
            self.selected = len - 1;
        }
    }

    /// Appends a CPU sample (rolling 60-point window).
    // JUSTIFY: Part of the public API for future dashboard charts.
    #[allow(dead_code)]
    pub fn push_cpu(&mut self, value: f64) {
        if self.cpu_history.len() >= 60 {
            self.cpu_history.pop_front();
        }
        self.cpu_history.push_back(value);
    }

    /// Appends a RAM sample (rolling 60-point window).
    // JUSTIFY: Part of the public API for future dashboard charts.
    #[allow(dead_code)]
    pub fn push_ram(&mut self, value: f64) {
        if self.ram_history.len() >= 60 {
            self.ram_history.pop_front();
        }
        self.ram_history.push_back(value);
    }

    /// Appends a network-in sample.
    // JUSTIFY: Part of the public API for future dashboard charts.
    #[allow(dead_code)]
    pub fn push_net_in(&mut self, value: f64) {
        if self.net_in_history.len() >= 60 {
            self.net_in_history.pop_front();
        }
        self.net_in_history.push_back(value);
    }

    /// Appends a network-out sample.
    // JUSTIFY: Part of the public API for future dashboard charts.
    #[allow(dead_code)]
    pub fn push_net_out(&mut self, value: f64) {
        if self.net_out_history.len() >= 60 {
            self.net_out_history.pop_front();
        }
        self.net_out_history.push_back(value);
    }

    /// Returns the `(id, name)` of the selected item on the active tab,
    /// for tabs whose resources are addressable by a numeric id.
    #[must_use]
    pub fn selected_resource(&self) -> Option<(i32, String)> {
        let real = *self.filtered_indices().get(self.selected)?;
        match self.active_tab {
            ResourceTab::Servers => self.servers.get(real).map(|s| (s.id, s.name.clone())),
            ResourceTab::Databases => self.databases.get(real).map(|d| (d.id, d.name.clone())),
            ResourceTab::S3 => self.s3_storages.get(real).map(|s| (s.id, s.name.clone())),
            ResourceTab::Kubernetes => self.k8s_clusters.get(real).map(|c| (c.id, c.name.clone())),
            ResourceTab::Balancers => self.balancers.get(real).map(|b| (b.id, b.name.clone())),
            ResourceTab::Registry => self.registries.get(real).map(|r| (r.id, r.name.clone())),
            ResourceTab::Projects => self.projects.get(real).map(|p| (p.id, p.name.clone())),
            ResourceTab::DedicatedServers => self
                .dedicated_servers
                .get(real)
                .map(|d| (d.id, d.name.clone())),
            ResourceTab::AiAgents => self.ai_agents.get(real).map(|a| (a.id, a.name.clone())),
            ResourceTab::Apps => self.apps.get(real).map(|a| (a.id, a.name.clone())),
            ResourceTab::KnowledgeBases => self
                .knowledge_bases
                .get(real)
                .map(|k| (k.id, k.name.clone())),
            _ => None
        }
    }

    /// Opens the context action menu for the selected resource.
    ///
    /// No-op when the active tab declares no actions or nothing is selected.
    pub fn open_action_menu(&mut self) {
        let actions = self.active_tab.actions();
        if actions.is_empty() {
            return;
        }
        if let Some((id, name)) = self.selected_resource() {
            self.action_menu = Some(ActionMenu {
                tab:           self.active_tab,
                resource_id:   id,
                resource_name: name,
                actions:       actions.to_vec(),
                selected:      0
            });
        }
    }

    /// Closes the action menu without choosing anything.
    pub fn close_action_menu(&mut self) {
        self.action_menu = None;
    }

    /// Returns true when the active tab's selected resource can be entered
    /// to reveal contained resources (currently only projects).
    #[must_use]
    pub fn can_drill(&self) -> bool {
        matches!(self.active_tab, ResourceTab::Projects) && self.selected_resource().is_some()
    }

    /// Requests a drill-in into the selected resource; the loop fetches it.
    pub fn request_drill(&mut self) {
        if self.can_drill()
            && let Some((id, name)) = self.selected_resource()
        {
            self.drill_request = Some((self.active_tab, id, name));
        }
    }

    /// Takes the pending drill request for the loop to fetch.
    #[must_use]
    pub const fn take_drill_request(&mut self) -> Option<(ResourceTab, i32, String)> {
        self.drill_request.take()
    }

    /// Opens the drill-in view with fetched contents.
    pub fn open_drill(&mut self, view: DrillView) {
        self.drill = Some(view);
    }

    /// Closes the drill-in view, returning to the resource list.
    pub fn close_drill(&mut self) {
        self.drill = None;
    }

    /// Returns true while a drill-in view is open.
    #[must_use]
    pub const fn drill_open(&self) -> bool {
        self.drill.is_some()
    }

    /// Returns the open drill-in view, for rendering.
    #[must_use]
    pub const fn drill_view(&self) -> Option<&DrillView> {
        self.drill.as_ref()
    }

    /// Moves the drill selection down.
    pub const fn drill_next(&mut self) {
        if let Some(view) = self.drill.as_mut()
            && view.selected + 1 < view.items.len()
        {
            view.selected += 1;
        }
    }

    /// Moves the drill selection up.
    pub const fn drill_previous(&mut self) {
        if let Some(view) = self.drill.as_mut() {
            view.selected = view.selected.saturating_sub(1);
        }
    }

    /// Returns true while the action menu is open.
    #[must_use]
    pub const fn action_menu_open(&self) -> bool {
        self.action_menu.is_some()
    }

    /// Returns the open action menu, for rendering.
    #[must_use]
    pub const fn action_menu(&self) -> Option<&ActionMenu> {
        self.action_menu.as_ref()
    }

    /// Moves the action-menu highlight to the next item (wraps).
    pub const fn menu_next(&mut self) {
        if let Some(menu) = self.action_menu.as_mut()
            && !menu.actions.is_empty()
        {
            menu.selected = (menu.selected + 1) % menu.actions.len();
        }
    }

    /// Moves the action-menu highlight to the previous item (wraps).
    pub const fn menu_previous(&mut self) {
        if let Some(menu) = self.action_menu.as_mut() {
            let len = menu.actions.len();
            if len > 0 {
                menu.selected = (menu.selected + len - 1) % len;
            }
        }
    }

    /// Chooses the highlighted action: destructive actions open a
    /// confirmation prompt, others are queued for dispatch immediately.
    pub fn menu_select(&mut self) {
        let Some(menu) = self.action_menu.take() else {
            return;
        };
        let Some(&action) = menu.actions.get(menu.selected) else {
            return;
        };
        let pending = PendingAction {
            tab:           menu.tab,
            kind:          action,
            resource_id:   menu.resource_id,
            resource_name: menu.resource_name
        };
        if action.is_destructive() {
            self.confirm = Some(pending);
        } else {
            self.dispatch = Some(pending);
        }
    }

    /// Confirms the pending destructive action, queueing it for dispatch.
    pub fn confirm_action(&mut self) {
        self.dispatch = self.confirm.take();
    }

    /// Dismisses the pending action without dispatching it.
    pub fn cancel_action(&mut self) {
        self.confirm = None;
    }

    /// Returns the action awaiting confirmation, for rendering the modal.
    #[must_use]
    pub const fn pending_action(&self) -> Option<&PendingAction> {
        self.confirm.as_ref()
    }

    /// Returns true while a confirmation modal is open.
    #[must_use]
    pub const fn awaiting_confirm(&self) -> bool {
        self.confirm.is_some()
    }

    /// Takes the action queued for dispatch, if the user confirmed one.
    #[must_use]
    pub const fn take_dispatch(&mut self) -> Option<PendingAction> {
        self.dispatch.take()
    }

    /// Widgets the user can show/hide from the layout: `(id, label)`.
    pub const TOGGLEABLE_WIDGETS: [(&'static str, &'static str); 4] = [
        ("account", "Account header"),
        ("stats", "Stats panel"),
        ("token_info", "Token info panel"),
        ("events", "Events log")
    ];

    /// Applies persisted dashboard preferences: hides the given widgets and
    /// sets the resource-list width.
    pub fn apply_prefs(&mut self, hidden: &[String], list_width_pct: u16, hide_empty_tabs: bool) {
        for id in hidden {
            if self.is_widget_enabled(id) {
                self.widgets.toggle(id);
            }
        }
        if (10..=90).contains(&list_width_pct) {
            self.list_width_pct = list_width_pct;
        }
        self.hide_empty_tabs = hide_empty_tabs;
    }

    /// Returns true when the widget with `id` is registered and enabled.
    #[must_use]
    pub fn is_widget_enabled(&self, id: &str) -> bool {
        self.widgets
            .get(id)
            .is_some_and(super::widgets::Widget::enabled)
    }

    /// Toggles a widget's visibility and marks preferences dirty.
    pub fn toggle_widget(&mut self, id: &str) {
        self.widgets.toggle(id);
        self.prefs_dirty = true;
    }

    /// Returns the localized display label for a toggleable widget id.
    #[must_use]
    pub fn widget_display_label(id: &str) -> Cow<'static, str> {
        match id {
            "account" => t!("app.widget_account"),
            "stats" => t!("app.widget_stats"),
            "token_info" => t!("app.widget_token_info"),
            "events" => t!("app.widget_events"),
            _ => Cow::Borrowed("")
        }
    }

    /// Returns the ids of currently hidden toggleable widgets, for persisting.
    #[must_use]
    pub fn hidden_widget_ids(&self) -> Vec<String> {
        Self::TOGGLEABLE_WIDGETS
            .iter()
            .filter(|(id, _)| !self.is_widget_enabled(id))
            .map(|(id, _)| (*id).to_string())
            .collect()
    }

    /// Switches the active theme and marks preferences dirty.
    pub const fn set_theme(&mut self, theme: super::themes::Theme) {
        self.theme = theme;
        self.prefs_dirty = true;
    }

    /// Advances the animation tick (drives skeleton shimmer).
    pub const fn tick(&mut self) {
        self.anim_tick = self.anim_tick.wrapping_add(1);
    }

    /// Returns true while the command palette is open.
    #[must_use]
    pub const fn palette_open(&self) -> bool {
        self.palette.is_some()
    }

    /// Opens the command palette, populated for the current context.
    pub fn open_palette(&mut self) {
        let commands = self.build_palette_commands();
        self.palette = Some(super::command_palette::CommandPalette::new(commands));
    }

    /// Closes the command palette.
    pub fn close_palette(&mut self) {
        self.palette = None;
    }

    /// Feeds a character to the open palette query.
    pub fn palette_input(&mut self, c: char) {
        if let Some(p) = self.palette.as_mut() {
            p.push_char(c);
        }
    }

    /// Deletes the last palette query character.
    pub fn palette_backspace(&mut self) {
        if let Some(p) = self.palette.as_mut() {
            p.backspace();
        }
    }

    /// Moves the palette selection down.
    pub const fn palette_next(&mut self) {
        if let Some(p) = self.palette.as_mut() {
            p.next();
        }
    }

    /// Moves the palette selection up.
    pub const fn palette_previous(&mut self) {
        if let Some(p) = self.palette.as_mut() {
            p.previous();
        }
    }

    /// Runs the highlighted palette command, then closes the palette.
    pub fn palette_run_selected(&mut self) {
        let id = self
            .palette
            .as_ref()
            .and_then(|p| p.selected_command())
            .map(|c| c.id.clone());
        if let Some(id) = id {
            self.run_command(&id);
        }
        self.close_palette();
    }

    fn build_palette_commands(&self) -> Vec<super::command_palette::Command> {
        use super::command_palette::Command;
        let mut commands = Vec::new();

        if let Some((_, name)) = self.selected_resource() {
            for action in self.active_tab.actions() {
                commands.push(Command {
                    id:    format!("action:{}", action.label().to_lowercase()),
                    title: format!("{} {name}", action.display_label()),
                    hint:  t!("app.hint_action").to_string()
                });
            }
        }

        for (id, _) in Self::TOGGLEABLE_WIDGETS {
            let verb = if self.is_widget_enabled(id) {
                t!("app.palette_hide")
            } else {
                t!("app.palette_show")
            };
            commands.push(Command {
                id:    format!("widget:{id}"),
                title: format!("{verb} {}", Self::widget_display_label(id)),
                hint:  t!("app.hint_layout").to_string()
            });
        }

        for theme in super::themes::Theme::ALL {
            commands.push(Command {
                id:    format!("theme:{}", theme.id()),
                title: format!("Theme: {}", theme.label()),
                hint:  t!("app.hint_theme").to_string()
            });
        }

        commands.push(Command {
            id:    "tabs:toggle_empty".to_string(),
            title: if self.hide_empty_tabs {
                t!("app.palette_show_empty_tabs").to_string()
            } else {
                t!("app.palette_hide_empty_tabs").to_string()
            },
            hint:  t!("app.hint_layout").to_string()
        });

        commands.push(Command {
            id:    "lang:en".to_string(),
            title: rust_i18n::t!("palette.language_english").to_string(),
            hint:  t!("app.hint_language").to_string()
        });
        commands.push(Command {
            id:    "lang:ru".to_string(),
            title: rust_i18n::t!("palette.language_russian").to_string(),
            hint:  t!("app.hint_language").to_string()
        });

        commands
    }

    fn run_command(&mut self, id: &str) {
        if id == "tabs:toggle_empty" {
            self.toggle_hide_empty_tabs();
        } else if id == "lang:en" {
            self.set_language(crate::config::Language::En);
        } else if id == "lang:ru" {
            self.set_language(crate::config::Language::Ru);
        } else if let Some(rest) = id.strip_prefix("theme:") {
            if let Some(theme) = super::themes::Theme::ALL
                .into_iter()
                .find(|t| t.id() == rest)
            {
                self.set_theme(theme);
            }
        } else if let Some(widget_id) = id.strip_prefix("widget:") {
            self.toggle_widget(widget_id);
        } else if let Some(action_label) = id.strip_prefix("action:")
            && let Some((resource_id, resource_name)) = self.selected_resource()
            && let Some(&kind) = self
                .active_tab
                .actions()
                .iter()
                .find(|a| a.label().eq_ignore_ascii_case(action_label))
        {
            let pending = PendingAction {
                tab: self.active_tab,
                kind,
                resource_id,
                resource_name
            };
            if kind.is_destructive() {
                self.confirm = Some(pending);
            } else {
                self.dispatch = Some(pending);
            }
        }
    }

    /// Quits the application.
    pub const fn quit(&mut self) {
        self.running = false;
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

/// An owned snapshot of all dashboard data, fetched off the UI thread and
/// applied via [`App::apply_data`]. Cloned out of a throwaway [`App`] by the
/// background refresh task.
#[derive(Debug, Clone)]
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
    /// Clones the resource data out of an [`App`] populated by `refresh_all`.
    #[must_use]
    pub fn from_app(app: &App) -> Self {
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
            error_message:     app.error_message.clone(),
            status_message:    app.status_message.clone(),
            load_errors:       app.last_load_errors.clone()
        }
    }
}

#[cfg(test)]
mod tests;
