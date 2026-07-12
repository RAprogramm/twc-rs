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
    /// Opens the given project's contents panel immediately (empty, marked
    /// loading, so no intermediate screen flashes) and queues the fetch.
    pub fn open_project_drill(&mut self, index: usize) {
        let Some(project) = self.projects.get(index) else {
            return;
        };
        self.drill = Some(DrillView {
            title:    project.name.clone(),
            items:    Vec::new(),
            selected: 0
        });
        self.drill_loading = true;
        self.drill_request = Some((ResourceTab::Projects, project.id, project.name.clone()));
    }

    /// Takes the pending drill request for the loop to fetch.
    #[must_use]
    pub const fn take_drill_request(&mut self) -> Option<(ResourceTab, i32, String)> {
        self.drill_request.take()
    }

    /// Opens the drill-in view with fetched contents.
    pub fn open_drill(&mut self, view: DrillView) {
        self.drill = Some(view);
        self.drill_loading = false;
    }

    /// Closes the drill-in view; the content pane reverts to the selected
    /// sidebar entry's default view.
    pub fn close_drill(&mut self) {
        self.drill = None;
        self.drill_loading = false;
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
}
