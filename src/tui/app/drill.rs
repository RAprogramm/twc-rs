// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Drill-in views listing the resources contained in a project.

use super::ResourceTab;

/// A single row inside a drill-in view (a resource contained in a parent).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrillItem {
    /// The resource category, for opening the full details view.
    pub tab:    ResourceTab,
    /// The resource id within its category.
    pub id:     String,
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
        self.drill_retried = None;
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
    ///
    /// The project-resources endpoint occasionally flaps, returning a response
    /// with whole collections missing. A response that strictly shrinks the
    /// cached contents is therefore not applied immediately: it triggers one
    /// silent re-fetch, and only a confirming second answer replaces the data
    /// (so real deletions still show up).
    pub fn apply_drill(&mut self, id: i32, view: DrillView) {
        if self.drill_retried != Some(id)
            && let Some(cached) = self.drill_cache.get(&id)
            && looks_like_flap(cached, &view)
        {
            self.drill_retried = Some(id);
            self.drill_request = Some((ResourceTab::Projects, id, view.title));
            return;
        }
        self.drill_retried = None;
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
        self.detail_open = false;
    }

    /// Opens the full details view for the drill item under the highlight:
    /// points the detail renderer at that resource in its own category.
    /// Returns false when the resource is not loaded (yet).
    pub fn open_drill_item_detail(&mut self) -> bool {
        let Some(item) = self
            .drill
            .as_ref()
            .and_then(|d| d.items.get(d.selected))
            .cloned()
        else {
            return false;
        };
        let index = match item.tab {
            ResourceTab::Servers => position(&self.servers, |s| s.id.to_string() == item.id),
            ResourceTab::Databases => position(&self.databases, |d| d.id.to_string() == item.id),
            ResourceTab::S3 => position(&self.s3_storages, |s| s.id.to_string() == item.id),
            ResourceTab::Kubernetes => {
                position(&self.k8s_clusters, |c| c.id.to_string() == item.id)
            }
            ResourceTab::Balancers => position(&self.balancers, |b| b.id.to_string() == item.id),
            ResourceTab::DedicatedServers => {
                position(&self.dedicated_servers, |d| d.id.to_string() == item.id)
            }
            ResourceTab::Apps => position(&self.apps, |a| a.id.to_string() == item.id),
            _ => None
        };
        let Some(index) = index else {
            return false;
        };
        self.active_tab = item.tab;
        self.filter.clear();
        self.filter_editing = false;
        self.selected = index;
        self.detail_scroll = 0;
        self.detail_selected = 0;
        self.detail_open = true;
        if let Ok(id) = item.id.parse::<i32>() {
            self.detail_fetch = Some((item.tab, id));
        }
        true
    }

    /// Opens the interactive details panel for the resource selected in the
    /// current service list: the same view a project drill opens, with the
    /// action buttons on top and the deep sections fetched in the background.
    pub fn open_selected_detail(&mut self) -> bool {
        let Some((id, _)) = self.selected_resource() else {
            return false;
        };
        self.detail_scroll = 0;
        self.detail_selected = 0;
        self.detail_open = true;
        if let Ok(id) = id.parse::<i32>() {
            self.detail_fetch = Some((self.active_tab, id));
        }
        true
    }

    /// Takes the pending detail action for the loop to execute.
    #[must_use]
    pub const fn take_detail_action(
        &mut self
    ) -> Option<(i32, crate::tui::widgets::details::DetailAction)> {
        self.detail_action.take()
    }

    /// Takes the pending deep-detail fetch for the loop to run in the
    /// background.
    #[must_use]
    pub const fn take_detail_fetch(&mut self) -> Option<(ResourceTab, i32)> {
        self.detail_fetch.take()
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

/// Index of the first element matching the predicate.
fn position<T>(items: &[T], pred: impl Fn(&T) -> bool) -> Option<usize> {
    items.iter().position(pred)
}

/// True when `fresh` is a strict subset of `cached` — the signature of the
/// project-resources endpoint transiently dropping a collection rather than a
/// real deletion (which would change ids, not just shrink the set).
fn looks_like_flap(cached: &DrillView, fresh: &DrillView) -> bool {
    fresh.items.len() < cached.items.len()
        && fresh
            .items
            .iter()
            .all(|f| cached.items.iter().any(|c| c.tab == f.tab && c.id == f.id))
}
