// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Application state for the TUI dashboard.

use std::{
    collections::VecDeque,
    time::{Duration, Instant}
};

use crate::tui::widgets::project_manager::ProjectManager;

/// Account information from the API.
#[derive(Debug, Clone, Default)]
pub struct AccountInfo {
    pub account_id: f64,
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

/// Navigation depth level for vim-style navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavLevel {
    /// Moving focus between panels (h/l to switch).
    Overview,
    /// Interacting with content inside the focused panel.
    Inner
}

/// An action that can be triggered against a server from the dashboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerAction {
    /// Power the server on.
    Start,
    /// Gracefully power the server off.
    Shutdown,
    /// Reboot the server.
    Reboot,
    /// Create a clone of the server.
    Clone,
    /// Permanently delete the server.
    Delete
}

impl ServerAction {
    /// The actions offered for a server, in menu order.
    pub const ALL: [Self; 5] = [
        Self::Start,
        Self::Shutdown,
        Self::Reboot,
        Self::Clone,
        Self::Delete
    ];

    /// Returns the label shown in the action menu and confirmation prompt.
    #[must_use]
    pub const fn verb(self) -> &'static str {
        match self {
            Self::Start => "Start",
            Self::Shutdown => "Shutdown",
            Self::Reboot => "Reboot",
            Self::Clone => "Clone",
            Self::Delete => "Delete"
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

/// A server action awaiting confirmation, or ready to be dispatched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingAction {
    /// The action to perform.
    pub action:      ServerAction,
    /// Target server id.
    pub server_id:   i32,
    /// Target server name, for display.
    pub server_name: String
}

/// A context action menu opened over the selected resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionMenu {
    /// Target server id.
    pub server_id:   i32,
    /// Target server name, for display.
    pub server_name: String,
    /// Available actions, in display order.
    pub actions:     Vec<ServerAction>,
    /// Index of the highlighted action.
    pub selected:    usize
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

    /// Cycles to the next tab.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Servers => Self::Databases,
            Self::Databases => Self::S3,
            Self::S3 => Self::Kubernetes,
            Self::Kubernetes => Self::Projects,
            Self::Projects => Self::Balancers,
            Self::Balancers => Self::Registry,
            Self::Registry => Self::Domains,
            Self::Domains => Self::Firewall,
            Self::Firewall => Self::FloatingIps,
            Self::FloatingIps => Self::Images,
            Self::Images => Self::NetworkDrives,
            Self::NetworkDrives => Self::Vpc,
            Self::Vpc => Self::DedicatedServers,
            Self::DedicatedServers => Self::Mail,
            Self::Mail => Self::Apps,
            Self::Apps => Self::AiAgents,
            Self::AiAgents => Self::KnowledgeBases,
            Self::KnowledgeBases => Self::SshKeys,
            Self::SshKeys => Self::Finances,
            Self::Finances => Self::Servers
        }
    }

    /// Cycles to the previous tab.
    #[must_use]
    pub const fn previous(self) -> Self {
        match self {
            Self::Servers => Self::Finances,
            Self::Databases => Self::Servers,
            Self::S3 => Self::Databases,
            Self::Kubernetes => Self::S3,
            Self::Projects => Self::Kubernetes,
            Self::Balancers => Self::Projects,
            Self::Registry => Self::Balancers,
            Self::Domains => Self::Registry,
            Self::Firewall => Self::Domains,
            Self::FloatingIps => Self::Firewall,
            Self::Images => Self::FloatingIps,
            Self::NetworkDrives => Self::Images,
            Self::Vpc => Self::NetworkDrives,
            Self::DedicatedServers => Self::Vpc,
            Self::Mail => Self::DedicatedServers,
            Self::Apps => Self::Mail,
            Self::AiAgents => Self::Apps,
            Self::KnowledgeBases => Self::AiAgents,
            Self::SshKeys => Self::KnowledgeBases,
            Self::Finances => Self::SshKeys
        }
    }

    /// Returns the index of this tab.
    // JUSTIFY: Public API method for future UI integration.
    #[allow(dead_code)]
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
}
/// Which panel is currently focused.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Focus {
    /// Resource tabs bar at the top.
    ResourceTabs,
    /// Resource list panel (left side).
    #[default]
    ResourceList,
    /// Details panel (right side).
    Details
}

impl Focus {
    /// Returns the display label for the focus target.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ResourceTabs => "Tabs",
            Self::ResourceList => "List",
            Self::Details => "Details"
        }
    }

    /// Moves focus to the left neighbor.
    #[must_use]
    pub const fn left(self) -> Self {
        match self {
            Self::Details => Self::ResourceList,
            Self::ResourceList | Self::ResourceTabs => Self::ResourceTabs
        }
    }

    /// Moves focus to the right neighbor.
    #[must_use]
    pub const fn right(self) -> Self {
        match self {
            Self::ResourceTabs => Self::ResourceList,
            Self::ResourceList | Self::Details => Self::Details
        }
    }
}

/// Holds all runtime state for the TUI dashboard.
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
    pub net_in_history:    VecDeque<u64>,
    pub net_out_history:   VecDeque<u64>,
    pub last_refresh:      Instant,
    pub refresh_interval:  Duration,
    pub running:           bool,
    pub show_help:         bool,
    pub status_message:    Option<String>,
    pub error_message:     Option<String>,
    pub is_loading:        bool,
    pub widgets:           super::widgets::WidgetRegistry,
    pub project_manager:   ProjectManager,
    pub focus:             Focus,
    pub nav_level:         NavLevel,
    pub action_menu:       Option<ActionMenu>,
    pub confirm:           Option<PendingAction>,
    pub dispatch:          Option<PendingAction>
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
            project_manager: ProjectManager::new(),
            focus: Focus::ResourceList,
            nav_level: NavLevel::Overview,
            action_menu: None,
            confirm: None,
            dispatch: None
        }
    }

    /// Returns the currently selected resource list length.
    #[must_use]
    pub const fn current_list_len(&self) -> usize {
        match self.active_tab {
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

    /// Moves selection up.
    pub const fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Moves selection down.
    pub const fn select_next(&mut self) {
        if self.selected + 1 < self.current_list_len() {
            self.selected += 1;
        }
    }

    /// Cycles to the next resource tab.
    pub const fn next_tab(&mut self) {
        self.active_tab = self.active_tab.next();
        self.selected = 0;
    }

    /// Toggles the help overlay.
    pub const fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Marks that a refresh is needed immediately.
    pub fn force_refresh(&mut self) {
        self.last_refresh = Instant::now()
            .checked_sub(self.refresh_interval)
            .unwrap_or_else(Instant::now);
    }

    /// Returns true when the refresh interval has elapsed.
    #[must_use]
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

    const fn clamp_selection(&mut self) {
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
    pub fn push_net_in(&mut self, value: u64) {
        if self.net_in_history.len() >= 60 {
            self.net_in_history.pop_front();
        }
        self.net_in_history.push_back(value);
    }

    /// Appends a network-out sample.
    // JUSTIFY: Part of the public API for future dashboard charts.
    #[allow(dead_code)]
    pub fn push_net_out(&mut self, value: u64) {
        if self.net_out_history.len() >= 60 {
            self.net_out_history.pop_front();
        }
        self.net_out_history.push_back(value);
    }

    /// Returns the selected server, but only on the Servers tab.
    #[must_use]
    pub fn selected_server(&self) -> Option<&ServerSummary> {
        if matches!(self.active_tab, ResourceTab::Servers) {
            self.servers.get(self.selected)
        } else {
            None
        }
    }

    /// Opens the context action menu for the selected server.
    ///
    /// No-op unless the Servers tab is active and a server is selected.
    pub fn open_action_menu(&mut self) {
        if let Some(server) = self.selected_server() {
            self.action_menu = Some(ActionMenu {
                server_id:   server.id,
                server_name: server.name.clone(),
                actions:     ServerAction::ALL.to_vec(),
                selected:    0
            });
        }
    }

    /// Closes the action menu without choosing anything.
    pub fn close_action_menu(&mut self) {
        self.action_menu = None;
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
    pub fn menu_next(&mut self) {
        if let Some(menu) = self.action_menu.as_mut()
            && !menu.actions.is_empty()
        {
            menu.selected = (menu.selected + 1) % menu.actions.len();
        }
    }

    /// Moves the action-menu highlight to the previous item (wraps).
    pub fn menu_previous(&mut self) {
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
            action,
            server_id: menu.server_id,
            server_name: menu.server_name
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
    pub fn take_dispatch(&mut self) -> Option<PendingAction> {
        self.dispatch.take()
    }

    /// Quits the application.
    pub const fn quit(&mut self) {
        self.running = false;
    }
}

#[cfg(test)]
mod tests;
