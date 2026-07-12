// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Application state for the TUI dashboard.

use std::{
    borrow::Cow,
    collections::VecDeque,
    time::{Duration, Instant}
};

use rust_i18n::t;

mod actions;
mod data;
mod drill;
mod forms;
mod nav;
mod navigation;
mod palette;
mod stats;
mod summaries;
mod tabs;
#[cfg(test)]
mod tests;

pub use actions::{ActionKind, ActionMenu, PendingAction};
pub use data::DataSlice;
pub use drill::{DrillItem, DrillView};
pub use forms::CreateForm;
pub use nav::{NavItem, NavKind};
pub use stats::{ResourceStats, StatsRequest};
pub use summaries::*;
pub use tabs::ResourceTab;

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
    Details,
    /// Stats panel (right info column).
    Stats,
    /// Event log panel (bottom).
    Events
}

/// A direction for moving focus between widgets on the dashboard grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDir {
    Left,
    Right,
    Up,
    Down
}

/// Which pane of the sidebar layout owns the keyboard.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Pane {
    /// The sidebar navigation list.
    #[default]
    Sidebar,
    /// The content card grid.
    Content
}

/// Holds all runtime state for the TUI dashboard.
#[allow(clippy::struct_excessive_bools)]
pub struct App {
    pub account:             AccountInfo,
    pub servers:             Vec<ServerSummary>,
    pub databases:           Vec<DatabaseSummary>,
    pub s3_storages:         Vec<S3Summary>,
    pub k8s_clusters:        Vec<K8sSummary>,
    pub projects:            Vec<ProjectSummary>,
    pub balancers:           Vec<BalancerSummary>,
    pub registries:          Vec<RegistrySummary>,
    pub domains:             Vec<DomainSummary>,
    pub firewalls:           Vec<FirewallSummary>,
    pub floating_ips:        Vec<FloatingIpSummary>,
    pub images:              Vec<ImageSummary>,
    pub network_drives:      Vec<NetworkDriveSummary>,
    pub vpcs:                Vec<VpcSummary>,
    pub dedicated_servers:   Vec<DedicatedServerSummary>,
    pub mails:               Vec<MailSummary>,
    pub apps:                Vec<AppSummary>,
    pub ai_agents:           Vec<AiAgentSummary>,
    pub knowledge_bases:     Vec<KnowledgeBaseSummary>,
    pub ssh_keys:            Vec<String>,
    pub finances:            Vec<String>,
    pub selected:            usize,
    pub active_tab:          ResourceTab,
    pub theme:               super::themes::Theme,
    pub token:               Option<String>,
    pub cpu_history:         VecDeque<f64>,
    pub ram_history:         VecDeque<f64>,
    pub net_in_history:      VecDeque<f64>,
    pub net_out_history:     VecDeque<f64>,
    pub last_refresh:        Instant,
    pub refresh_interval:    Duration,
    pub running:             bool,
    pub show_help:           bool,
    pub status_message:      Option<String>,
    pub error_message:       Option<String>,
    pub is_loading:          bool,
    pub widgets:             super::widgets::WidgetRegistry,
    pub focus:               Focus,
    pub action_menu:         Option<ActionMenu>,
    pub confirm:             Option<PendingAction>,
    pub dispatch:            Option<PendingAction>,
    pub palette:             Option<super::command_palette::CommandPalette>,
    pub list_width_pct:      u16,
    pub anim_tick:           u64,
    pub prefs_dirty:         bool,
    pub logs:                VecDeque<LogEntry>,
    pub last_load_errors:    Vec<String>,
    pub cycle_load_errors:   Vec<String>,
    pub projects_pending:    u8,
    pub services_pending:    u8,
    pub initial_cycle_done:  bool,
    pub manual_refresh_spin: bool,
    pub drill_cache:         std::collections::HashMap<i32, DrillView>,
    pub drill_fetching_id:   Option<i32>,
    pub refresh_requested:   bool,
    pub drill:               Option<DrillView>,
    pub drill_request:       Option<(ResourceTab, i32, String)>,
    pub filter:              String,
    pub filter_editing:      bool,
    pub hide_empty_tabs:     bool,
    pub initial_tab_set:     bool,
    pub detail_scroll:       u16,
    pub focus_active:        bool,
    pub pane:                Pane,
    pub nav_selected:        usize,
    pub resource_cols:       usize,
    pub language:            crate::config::Language,
    pub stats_subject:       Option<String>,
    pub stats_loaded_for:    Option<String>,
    pub stats_requested:     Option<Instant>,
    pub create_form:         Option<CreateForm>,
    pub create_request:      Option<CreateForm>,
    pub profiles:            Vec<String>,
    pub active_profile:      String,
    pub switch_profile:      Option<String>
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
            cycle_load_errors: Vec::new(),
            projects_pending: 2,
            services_pending: 17,
            initial_cycle_done: false,
            manual_refresh_spin: false,
            drill_cache: std::collections::HashMap::new(),
            drill_fetching_id: None,
            refresh_requested: false,
            drill: None,
            drill_request: None,
            filter: String::new(),
            filter_editing: false,
            hide_empty_tabs: false,
            initial_tab_set: false,
            detail_scroll: 0,
            focus_active: false,
            pane: Pane::default(),
            nav_selected: 0,
            resource_cols: 1,
            language: crate::config::Language::default(),
            stats_subject: None,
            stats_loaded_for: None,
            stats_requested: None,
            create_form: None,
            create_request: None,
            profiles: Vec::new(),
            active_profile: "default".to_string(),
            switch_profile: None
        }
    }

    /// Takes a pending profile-switch request (the dashboard loop re-auths).
    pub fn take_switch_profile(&mut self) -> Option<String> {
        self.switch_profile.take()
    }

    /// Sets the UI language, applies it live, and marks preferences dirty.
    pub fn set_language(&mut self, language: crate::config::Language) {
        self.language = language;
        rust_i18n::set_locale(language.locale());
        self.prefs_dirty = true;
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

    /// Toggles the help overlay.
    pub const fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
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

    /// Quits the application.
    pub const fn quit(&mut self) {
        self.running = false;
    }
}
