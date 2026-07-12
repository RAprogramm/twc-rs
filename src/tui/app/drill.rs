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
    /// Opens the given project's contents panel immediately and queues a
    /// background fetch. Cached contents show at once (stale-while-revalidate);
    /// only a never-opened project shows the loading skeleton.
    pub fn open_project_drill(&mut self, index: usize) {
        let Some(project) = self.projects.get(index) else {
            return;
        };
        if let Some(cached) = self.drill_cache.get(&project.id) {
            self.drill = Some(cached.clone());
            self.drill_loading = false;
        } else {
            self.drill = Some(DrillView {
                title:    project.name.clone(),
                items:    Vec::new(),
                selected: 0
            });
            self.drill_loading = true;
        }
        self.drill_fetching_id = Some(project.id);
        self.drill_request = Some((ResourceTab::Projects, project.id, project.name.clone()));
    }

    /// Takes the pending drill request for the loop to fetch.
    #[must_use]
    pub const fn take_drill_request(&mut self) -> Option<(ResourceTab, i32, String)> {
        self.drill_request.take()
    }

    /// Applies freshly fetched drill contents: updates the cache, and swaps
    /// the open panel in place when it still shows the same project (keeping
    /// the selection where possible).
    pub fn open_drill(&mut self, view: DrillView) {
        if let Some(id) = self.drill_fetching_id.take() {
            self.drill_cache.insert(id, view.clone());
        }
        match self.drill.as_mut() {
            Some(current) if current.title == view.title => {
                current.selected = current.selected.min(view.items.len().saturating_sub(1));
                current.items = view.items;
            }
            Some(_) => {}
            None => {}
        }
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
