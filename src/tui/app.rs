// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Application state for the TUI dashboard.

use std::{
    collections::VecDeque,
    time::{Duration, Instant}
};

/// Account information from the API.
#[derive(Debug, Clone, Default)]
pub struct AccountInfo {
    pub account_id: f64,
    pub balance:    String,
    pub status:     String
}

/// Summary of a single server.
#[derive(Debug, Clone)]
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

/// Resource category in the left panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceTab {
    Servers,
    Databases,
    S3,
    Kubernetes,
    Projects
}

impl ResourceTab {
    /// Returns all tab names.
    pub fn names() -> &'static [&'static str] {
        &["Servers", "Databases", "S3", "Kubernetes", "Projects"]
    }

    /// Cycles to the next tab.
    pub fn next(self) -> Self {
        match self {
            Self::Servers => Self::Databases,
            Self::Databases => Self::S3,
            Self::S3 => Self::Kubernetes,
            Self::Kubernetes => Self::Projects,
            Self::Projects => Self::Servers
        }
    }

    /// Returns the index of this tab.
    pub fn index(self) -> usize {
        match self {
            Self::Servers => 0,
            Self::Databases => 1,
            Self::S3 => 2,
            Self::Kubernetes => 3,
            Self::Projects => 4
        }
    }
}

/// Holds all runtime state for the TUI dashboard.
pub struct App {
    pub account:          AccountInfo,
    pub servers:          Vec<ServerSummary>,
    pub databases:        Vec<DatabaseSummary>,
    pub s3_storages:      Vec<S3Summary>,
    pub k8s_clusters:     Vec<K8sSummary>,
    pub projects:         Vec<ProjectSummary>,
    pub selected:         usize,
    pub active_tab:       ResourceTab,
    pub theme:            super::themes::Theme,
    pub cpu_history:      VecDeque<f64>,
    pub ram_history:      VecDeque<f64>,
    pub net_in_history:   VecDeque<u64>,
    pub net_out_history:  VecDeque<u64>,
    pub last_refresh:     Instant,
    pub refresh_interval: Duration,
    pub running:          bool,
    pub show_help:        bool,
    pub status_message:   Option<String>,
    pub error_message:    Option<String>
}

impl App {
    /// Creates a new `App` with default state.
    pub fn new(refresh_secs: u64) -> Self {
        Self {
            account:          AccountInfo::default(),
            servers:          Vec::new(),
            databases:        Vec::new(),
            s3_storages:      Vec::new(),
            k8s_clusters:     Vec::new(),
            projects:         Vec::new(),
            selected:         0,
            active_tab:       ResourceTab::Servers,
            theme:            super::themes::Theme::default(),
            cpu_history:      VecDeque::with_capacity(60),
            ram_history:      VecDeque::with_capacity(60),
            net_in_history:   VecDeque::with_capacity(60),
            net_out_history:  VecDeque::with_capacity(60),
            last_refresh:     Instant::now(),
            refresh_interval: Duration::from_secs(refresh_secs),
            running:          true,
            show_help:        false,
            status_message:   None,
            error_message:    None
        }
    }

    /// Returns the currently selected resource list length.
    pub fn current_list_len(&self) -> usize {
        match self.active_tab {
            ResourceTab::Servers => self.servers.len(),
            ResourceTab::Databases => self.databases.len(),
            ResourceTab::S3 => self.s3_storages.len(),
            ResourceTab::Kubernetes => self.k8s_clusters.len(),
            ResourceTab::Projects => self.projects.len()
        }
    }

    /// Moves selection up.
    pub fn select_previous(&mut self) {
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

    /// Cycles to the next resource tab.
    pub fn next_tab(&mut self) {
        self.active_tab = self.active_tab.next();
        self.selected = 0;
    }

    /// Toggles the help overlay.
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Marks that a refresh is needed immediately.
    pub fn force_refresh(&mut self) {
        self.last_refresh = Instant::now() - self.refresh_interval;
    }

    /// Returns true when the refresh interval has elapsed.
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

    fn clamp_selection(&mut self) {
        let len = self.current_list_len();
        if len == 0 {
            self.selected = 0;
        } else if self.selected >= len {
            self.selected = len - 1;
        }
    }

    /// Appends a CPU sample (rolling 60-point window).
    pub fn push_cpu(&mut self, value: f64) {
        if self.cpu_history.len() >= 60 {
            self.cpu_history.pop_front();
        }
        self.cpu_history.push_back(value);
    }

    /// Appends a RAM sample (rolling 60-point window).
    pub fn push_ram(&mut self, value: f64) {
        if self.ram_history.len() >= 60 {
            self.ram_history.pop_front();
        }
        self.ram_history.push_back(value);
    }

    /// Appends a network-in sample.
    pub fn push_net_in(&mut self, value: u64) {
        if self.net_in_history.len() >= 60 {
            self.net_in_history.pop_front();
        }
        self.net_in_history.push_back(value);
    }

    /// Appends a network-out sample.
    pub fn push_net_out(&mut self, value: u64) {
        if self.net_out_history.len() >= 60 {
            self.net_out_history.pop_front();
        }
        self.net_out_history.push_back(value);
    }

    /// Quits the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}

#[cfg(test)]
mod tests;
