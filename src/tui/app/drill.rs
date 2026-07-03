// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Drill-in views listing the resources contained in a project.

use super::ResourceTab;

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

impl super::App {
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
            && let Ok(id) = id.parse::<i32>()
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
}
