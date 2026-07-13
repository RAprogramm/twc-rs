// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! The create hub model: one card per creatable resource type.

use super::ResourceTab;

impl super::App {
    /// The resource types offered by the create hub, projects first.
    /// Account-level sections (SSH keys) are managed from their own tab,
    /// not created from the hub.
    #[must_use]
    pub fn create_targets() -> Vec<ResourceTab> {
        let mut targets = vec![ResourceTab::Projects];
        targets.extend(
            Self::service_tabs()
                .into_iter()
                .filter(|tab| !matches!(tab, ResourceTab::SshKeys))
        );
        targets
    }

    /// Moves the create-hub selection one step on the card grid.
    pub fn create_move(&mut self, dir: super::FocusDir) {
        let len = Self::create_targets().len();
        self.create_selected =
            super::grid_step(self.create_selected, len, self.resource_cols, dir);
    }

    /// Activates the selected target: opens the creation form when one
    /// exists, otherwise a guide popup describing the product and how to get
    /// one today.
    pub fn create_activate(&mut self) {
        let targets = Self::create_targets();
        let Some(tab) = targets.get(self.create_selected.min(targets.len() - 1)) else {
            return;
        };
        self.active_tab = *tab;
        if matches!(tab, ResourceTab::Projects) {
            self.open_create_form();
            return;
        }
        let (desc, _) = crate::tui::widgets::service_header::texts(*tab);
        self.info_popup = Some((
            tab.display_name().into_owned(),
            format!("{desc}\n\n{}", rust_i18n::t!("create.instruction"))
        ));
    }

    /// Returns true while the info popup is open.
    #[must_use]
    pub const fn info_popup_open(&self) -> bool {
        self.info_popup.is_some()
    }

    /// Closes the info popup.
    pub fn info_popup_close(&mut self) {
        self.info_popup = None;
    }
}
