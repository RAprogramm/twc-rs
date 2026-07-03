// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Live statistics state: polling requests and sparkline histories.

use std::time::{Duration, Instant};

use super::ResourceTab;

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

impl super::App {
    /// Interval between live statistics refreshes for the selected resource.
    const STATS_REFRESH: Duration = Duration::from_secs(30);

    /// Polls the selected resource for live statistics.
    ///
    /// Returns a [`StatsRequest`] when the selected resource changed or the
    /// current series is older than [`Self::STATS_REFRESH`], keeping the
    /// sparklines live. Returns `None` while the data is fresh; clears the
    /// metrics panel when the selection moved to a resource without live
    /// statistics.
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
        let fresh = self
            .stats_requested
            .is_some_and(|at| at.elapsed() < Self::STATS_REFRESH);
        if target_id == self.stats_loaded_for && fresh {
            return None;
        }
        self.stats_loaded_for = target_id;

        if let Some((id, name)) = target {
            self.stats_requested = Some(Instant::now());
            Some(StatsRequest {
                tab: self.active_tab,
                id,
                name
            })
        } else {
            self.stats_requested = None;
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
}
