// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! List navigation, filtering and tab switching.

use super::ResourceTab;

impl super::App {
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

    /// On the first data load, moves off an empty active tab onto the first
    /// tab that actually has items, so the dashboard never opens on a blank
    /// list. Runs once; later manual tab changes are left untouched.
    pub fn select_initial_tab(&mut self) {
        if self.initial_tab_set {
            return;
        }
        self.initial_tab_set = true;
        if self.tab_count(self.active_tab) > 0 {
            return;
        }
        if let Some(tab) = ResourceTab::ALL
            .into_iter()
            .find(|t| self.tab_count(*t) > 0)
        {
            self.active_tab = tab;
            self.reset_after_tab_change();
        }
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
        self.detail_scroll = 0;
    }

    /// The widgets that can hold focus, left-to-right in the content row.
    fn focus_ring(&self) -> Vec<super::Focus> {
        use super::Focus;
        let mut ring = vec![Focus::ResourceList, Focus::Details];
        if self.is_widget_enabled("stats") {
            ring.push(Focus::Stats);
        }
        ring
    }

    /// Moves focus to the next widget in the ring, leaving any active widget.
    pub fn focus_next(&mut self) {
        let ring = self.focus_ring();
        let pos = ring.iter().position(|f| *f == self.focus).unwrap_or(0);
        self.focus = ring[(pos + 1) % ring.len()];
        self.focus_active = false;
    }

    /// Moves focus to the previous widget in the ring, leaving any active
    /// widget.
    pub fn focus_previous(&mut self) {
        let ring = self.focus_ring();
        let pos = ring.iter().position(|f| *f == self.focus).unwrap_or(0);
        self.focus = ring[(pos + ring.len() - 1) % ring.len()];
        self.focus_active = false;
    }

    /// Activates the focused widget so its own keys (select, scroll) apply.
    pub const fn activate_focus(&mut self) {
        self.focus_active = true;
    }

    /// Leaves the active widget, returning to widget-to-widget navigation.
    pub const fn deactivate_focus(&mut self) {
        self.focus_active = false;
    }

    /// Scrolls the details panel down by one line.
    pub const fn detail_scroll_down(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_add(1);
    }

    /// Scrolls the details panel up by one line.
    pub const fn detail_scroll_up(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_sub(1);
    }

    pub(super) fn clamp_selection(&mut self) {
        let len = self.current_list_len();
        if len == 0 {
            self.selected = 0;
        } else if self.selected >= len {
            self.selected = len - 1;
        }
    }

    /// Returns the `(id, name)` of the selected item on the active tab,
    /// for tabs whose resources are addressable by a numeric id.
    #[must_use]
    pub fn selected_resource(&self) -> Option<(String, String)> {
        let real = *self.filtered_indices().get(self.selected)?;
        match self.active_tab {
            ResourceTab::Servers => self
                .servers
                .get(real)
                .map(|s| (s.id.to_string(), s.name.clone())),
            ResourceTab::Databases => self
                .databases
                .get(real)
                .map(|d| (d.id.to_string(), d.name.clone())),
            ResourceTab::S3 => self
                .s3_storages
                .get(real)
                .map(|s| (s.id.to_string(), s.name.clone())),
            ResourceTab::Kubernetes => self
                .k8s_clusters
                .get(real)
                .map(|c| (c.id.to_string(), c.name.clone())),
            ResourceTab::Balancers => self
                .balancers
                .get(real)
                .map(|b| (b.id.to_string(), b.name.clone())),
            ResourceTab::Registry => self
                .registries
                .get(real)
                .map(|r| (r.id.to_string(), r.name.clone())),
            ResourceTab::Projects => self
                .projects
                .get(real)
                .map(|p| (p.id.to_string(), p.name.clone())),
            ResourceTab::DedicatedServers => self
                .dedicated_servers
                .get(real)
                .map(|d| (d.id.to_string(), d.name.clone())),
            ResourceTab::AiAgents => self
                .ai_agents
                .get(real)
                .map(|a| (a.id.to_string(), a.name.clone())),
            ResourceTab::Apps => self
                .apps
                .get(real)
                .map(|a| (a.id.to_string(), a.name.clone())),
            ResourceTab::KnowledgeBases => self
                .knowledge_bases
                .get(real)
                .map(|k| (k.id.to_string(), k.name.clone())),
            ResourceTab::Domains => self
                .domains
                .get(real)
                .map(|d| (d.id.to_string(), d.name.clone())),
            ResourceTab::Firewall => self
                .firewalls
                .get(real)
                .map(|f| (f.id.clone(), f.name.clone())),
            ResourceTab::FloatingIps => self
                .floating_ips
                .get(real)
                .map(|f| (f.id.clone(), f.ip.clone())),
            ResourceTab::Images => self
                .images
                .get(real)
                .map(|i| (i.id.clone(), i.name.clone())),
            ResourceTab::NetworkDrives => self
                .network_drives
                .get(real)
                .map(|n| (n.id.clone(), n.name.clone())),
            ResourceTab::Vpc => self.vpcs.get(real).map(|v| (v.id.clone(), v.name.clone())),
            _ => None
        }
    }
}
