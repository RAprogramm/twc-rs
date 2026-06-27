// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Application state for the TUI dashboard.

use std::{
    collections::VecDeque,
    time::{Duration, Instant}
};

/// Summary of a single server for display in the dashboard.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ServerSummary {
    pub id:        i32,
    pub name:      String,
    pub status:    String,
    pub cpu_count: i32,
    pub ram_mb:    i32,
    pub disk_gb:   i32
}

/// Active tab in the dashboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Overview,
    Servers,
    Databases,
    Storage
}

impl Tab {
    /// Returns all tab names for display.
    pub fn names() -> &'static [&'static str] {
        &["Overview", "Servers", "Databases", "Storage"]
    }

    /// Cycles to the next tab.
    pub fn next(self) -> Self {
        match self {
            Self::Overview => Self::Servers,
            Self::Servers => Self::Databases,
            Self::Databases => Self::Storage,
            Self::Storage => Self::Overview
        }
    }

    /// Returns the index of this tab.
    pub fn index(self) -> usize {
        match self {
            Self::Overview => 0,
            Self::Servers => 1,
            Self::Databases => 2,
            Self::Storage => 3
        }
    }
}

/// Holds all runtime state for the TUI dashboard.
///
/// Updated by the event loop; read by the renderer.
pub struct App {
    pub servers:          Vec<ServerSummary>,
    pub selected:         usize,
    pub active_tab:       Tab,
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
            servers:          Vec::new(),
            selected:         0,
            active_tab:       Tab::Overview,
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

    /// Moves selection up in the server list.
    pub fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Moves selection down in the server list.
    pub fn select_next(&mut self) {
        if self.selected + 1 < self.servers.len() {
            self.selected += 1;
        }
    }

    /// Cycles to the next tab.
    pub fn next_tab(&mut self) {
        self.active_tab = self.active_tab.next();
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

    /// Updates state with new server data.
    pub fn update_servers(&mut self, servers: Vec<ServerSummary>) {
        self.servers = servers;
        if self.selected >= self.servers.len() && !self.servers.is_empty() {
            self.selected = self.servers.len() - 1;
        }
        self.last_refresh = Instant::now();
    }

    /// Appends a CPU sample to the history (rolling 60-point window).
    #[allow(dead_code)]
    pub fn push_cpu(&mut self, value: f64) {
        if self.cpu_history.len() >= 60 {
            self.cpu_history.pop_front();
        }
        self.cpu_history.push_back(value);
    }

    /// Appends a RAM sample to the history (rolling 60-point window).
    #[allow(dead_code)]
    pub fn push_ram(&mut self, value: f64) {
        if self.ram_history.len() >= 60 {
            self.ram_history.pop_front();
        }
        self.ram_history.push_back(value);
    }

    /// Appends a network-in sample to the history.
    #[allow(dead_code)]
    pub fn push_net_in(&mut self, value: u64) {
        if self.net_in_history.len() >= 60 {
            self.net_in_history.pop_front();
        }
        self.net_in_history.push_back(value);
    }

    /// Appends a network-out sample to the history.
    #[allow(dead_code)]
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
