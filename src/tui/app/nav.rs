// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Sidebar navigation model: a vertical list of Projects and Services on the
//! left, with the selected entry's resources shown in the content pane.

use super::{App, Pane, ResourceTab};

/// What a sidebar entry points at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavKind {
    /// The create hub: cards for creating every resource type.
    Create,
    /// A project entry, carrying the project's list index.
    Project(usize),
    /// A service-category entry, carrying its resource tab.
    Service(ResourceTab),
    /// The settings panel.
    Settings
}

/// A single entry in the sidebar.
#[derive(Debug, Clone)]
pub struct NavItem {
    /// The target this entry opens.
    pub kind:  NavKind,
    /// The Nerd Font glyph shown before the label.
    pub glyph: &'static str,
    /// The entry's display label.
    pub label: String,
    /// The count badge shown right-aligned, when the entry has one.
    pub count: Option<usize>
}

impl App {
    /// The resource categories shown as service entries, in display order.
    pub(crate) const fn service_tabs() -> [ResourceTab; 18] {
        [
            ResourceTab::Servers,
            ResourceTab::Databases,
            ResourceTab::S3,
            ResourceTab::Kubernetes,
            ResourceTab::Balancers,
            ResourceTab::Registry,
            ResourceTab::Domains,
            ResourceTab::Firewall,
            ResourceTab::FloatingIps,
            ResourceTab::Images,
            ResourceTab::NetworkDrives,
            ResourceTab::Vpc,
            ResourceTab::DedicatedServers,
            ResourceTab::Mail,
            ResourceTab::Apps,
            ResourceTab::AiAgents,
            ResourceTab::KnowledgeBases,
            ResourceTab::SshKeys
        ]
    }

    /// True when the tab's service header offers a Create button, so the
    /// content pane may land on it; header-less tabs (e.g. Finances) never
    /// park the cursor on an invisible chip.
    pub(crate) fn tab_has_create(tab: ResourceTab) -> bool {
        !crate::tui::widgets::service_header::texts(tab).1.is_empty()
    }

    /// All sidebar entries: projects, then services (optionally hiding empty
    /// ones), the account finances entry, then the settings entry.
    #[must_use]
    pub fn nav_items(&self) -> Vec<NavItem> {
        use crate::tui::widgets::sidebar::tab_icon;

        let mut items: Vec<NavItem> = vec![NavItem {
            kind:  NavKind::Create,
            glyph: "\u{f067}",
            label: rust_i18n::t!("sidebar.create").into_owned(),
            count: None
        }];
        items.extend(self.projects.iter().enumerate().map(|(i, p)| NavItem {
            kind:  NavKind::Project(i),
            glyph: tab_icon(ResourceTab::Projects),
            label: p.name.clone(),
            count: Some(usize::try_from(p.resource_count()).unwrap_or(0))
        }));
        items.extend(
            Self::service_tabs()
                .into_iter()
                .filter(|tab| !self.hide_empty_tabs || self.tab_count(*tab) > 0)
                .map(|tab| NavItem {
                    kind:  NavKind::Service(tab),
                    glyph: tab_icon(tab),
                    label: tab.display_name().into_owned(),
                    count: Some(self.tab_count(tab))
                })
        );
        items.push(NavItem {
            kind:  NavKind::Service(ResourceTab::Finances),
            glyph: tab_icon(ResourceTab::Finances),
            label: ResourceTab::Finances.display_name().into_owned(),
            count: None
        });
        items.push(NavItem {
            kind:  NavKind::Settings,
            glyph: "\u{f013}",
            label: rust_i18n::t!("sidebar.settings").into_owned(),
            count: None
        });
        items
    }

    /// The kind of the currently selected sidebar entry.
    #[must_use]
    pub fn nav_current(&self) -> Option<NavKind> {
        self.nav_items()
            .get(self.nav_selected)
            .map(|i| i.kind.clone())
    }

    /// The longest sidebar label in characters, for sizing the sidebar.
    #[must_use]
    pub fn nav_longest_label(&self) -> usize {
        self.nav_items()
            .iter()
            .map(|i| i.label.chars().count())
            .max()
            .unwrap_or(0)
    }

    /// Moves the sidebar selection one entry up.
    pub fn nav_up(&mut self) {
        if self.nav_selected > 0 {
            self.nav_selected -= 1;
            self.nav_changed();
        }
    }

    /// Moves the sidebar selection one entry down.
    pub fn nav_down(&mut self) {
        if self.nav_selected + 1 < self.nav_items().len() {
            self.nav_selected += 1;
            self.nav_changed();
        }
    }

    /// Applies a sidebar selection change: the content pane switches to the
    /// newly selected entry immediately (service grids are local data; a
    /// project shows its per-type counts until opened).
    fn nav_changed(&mut self) {
        self.close_drill();
        self.selected = 0;
        self.filter.clear();
        self.filter_editing = false;
        match self.nav_current() {
            Some(NavKind::Service(tab)) => self.active_tab = tab,
            Some(NavKind::Project(index)) => {
                self.active_tab = ResourceTab::Projects;
                self.select_project_drill(index);
            }
            Some(NavKind::Settings | NavKind::Create) | None => {}
        }
    }

    /// Opens the selected entry: simply focuses the content pane — whatever
    /// the sidebar selection already raised there (service grid, cached
    /// project contents or its counts preview) stays put.
    pub fn nav_open(&mut self) {
        match self.nav_current() {
            Some(NavKind::Service(tab)) => {
                self.active_tab = tab;
                self.content_on_create = Self::tab_has_create(tab) && self.tab_count(tab) == 0;
                self.pane = Pane::Content;
            }
            Some(NavKind::Project(index)) => {
                self.active_tab = ResourceTab::Projects;
                if self.drill.is_none() && self.drill_fetching_id.is_none() {
                    self.select_project_drill(index);
                }
                self.pane = Pane::Content;
            }
            Some(NavKind::Settings | NavKind::Create) => self.pane = Pane::Content,
            None => {}
        }
    }

    /// Returns focus to the sidebar, closing any opened project contents.
    pub fn focus_sidebar(&mut self) {
        self.pane = Pane::Sidebar;
        self.content_on_create = false;
        self.close_drill();
        self.filter.clear();
        self.filter_editing = false;
    }
}
