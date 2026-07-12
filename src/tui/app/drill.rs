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
    /// Raises the given project's contents in the content pane the moment the
    /// sidebar selection lands on it: cached contents show instantly, an
    /// uncached project shows its counts preview, and a silent background
    /// fetch revalidates either way. Never blocks, never shows a skeleton.
    pub fn select_project_drill(&mut self, index: usize) {
        let Some(project) = self.projects.get(index) else {
            return;
        };
        self.drill = self.drill_cache.get(&project.id).cloned();
        self.drill_fetching_id = Some(project.id);
        self.drill_request = Some((ResourceTab::Projects, project.id, project.name.clone()));
    }

    /// Takes the pending drill request for the loop to fetch in the background.
    #[must_use]
    pub const fn take_drill_request(&mut self) -> Option<(ResourceTab, i32, String)> {
        self.drill_request.take()
    }

    /// Applies drill contents fetched for project `id`: refreshes the cache,
    /// and swaps the open pane in place when that project is still the one
    /// selected (keeping the highlighted row where possible).
    pub fn apply_drill(&mut self, id: i32, view: DrillView) {
        self.drill_cache.insert(id, view.clone());
        if self.drill_fetching_id != Some(id) {
            return;
        }
        let selected = self
            .drill
            .as_ref()
            .map_or(0, |d| d.selected)
            .min(view.items.len().saturating_sub(1));
        self.drill = Some(DrillView {
            selected,
            ..view
        });
    }

    /// Closes the drill-in view; the content pane reverts to the selected
    /// sidebar entry's default view.
    pub fn close_drill(&mut self) {
        self.drill = None;
        self.drill_fetching_id = None;
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
