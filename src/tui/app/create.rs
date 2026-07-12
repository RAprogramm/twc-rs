// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! The create hub model: one card per creatable resource type.

use super::ResourceTab;

impl super::App {
    /// The resource types offered by the create hub, projects first.
    #[must_use]
    pub fn create_targets() -> Vec<ResourceTab> {
        let mut targets = vec![ResourceTab::Projects];
        targets.extend(Self::service_tabs());
        targets
    }

    /// Moves the create-hub selection one step on the card grid.
    pub fn create_move(&mut self, dir: super::FocusDir) {
        let len = Self::create_targets().len();
        self.create_selected =
            super::grid_step(self.create_selected, len, self.resource_cols, dir);
    }

    /// Opens the creation form for the selected target.
    pub fn create_activate(&mut self) {
        let targets = Self::create_targets();
        let Some(tab) = targets.get(self.create_selected.min(targets.len() - 1)) else {
            return;
        };
        self.active_tab = *tab;
        self.open_create_form();
    }
}
