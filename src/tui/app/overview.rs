// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! The overview landing screen: Projects and Services zones as cards.

use super::{App, DashboardView, FocusDir, ResourceTab};

/// What a card on the overview screen points at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverviewKind {
    /// A project card, carrying the project's list index.
    Project(usize),
    /// A service-category card, carrying its resource tab.
    Service(ResourceTab)
}

/// A single card shown on the overview screen.
#[derive(Debug, Clone)]
pub struct OverviewCard {
    /// The target this card opens.
    pub kind:  OverviewKind,
    /// The tab whose icon represents this card.
    pub icon:  ResourceTab,
    /// The card's display label.
    pub label: String,
    /// The count shown large on the card.
    pub count: usize
}

impl App {
    /// The resource categories shown as service cards, in display order.
    fn service_tabs() -> [ResourceTab; 17] {
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
            ResourceTab::KnowledgeBases
        ]
    }

    /// The project cards, one per project, counting its resources.
    #[must_use]
    pub fn overview_project_cards(&self) -> Vec<OverviewCard> {
        self.projects
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let count = p.server_count
                    + p.database_count
                    + p.bucket_count
                    + p.cluster_count
                    + p.balancer_count
                    + p.dedicated_count
                    + p.app_count;
                OverviewCard {
                    kind:  OverviewKind::Project(i),
                    icon:  ResourceTab::Projects,
                    label: p.name.clone(),
                    count: usize::try_from(count).unwrap_or(0)
                }
            })
            .collect()
    }

    /// The service cards, one per category, showing its loaded count.
    #[must_use]
    pub fn overview_service_cards(&self) -> Vec<OverviewCard> {
        Self::service_tabs()
            .into_iter()
            .map(|tab| OverviewCard {
                kind:  OverviewKind::Service(tab),
                icon:  tab,
                label: tab.display_name().into_owned(),
                count: self.tab_count(tab)
            })
            .collect()
    }

    /// All overview cards, projects first then services, in navigation order.
    #[must_use]
    pub fn overview_cards(&self) -> Vec<OverviewCard> {
        let mut cards = self.overview_project_cards();
        cards.extend(self.overview_service_cards());
        cards
    }

    /// Moves the overview selection on the card grid in the given direction.
    pub fn move_overview(&mut self, dir: FocusDir) {
        let len = self.overview_cards().len();
        if len == 0 {
            self.overview_selected = 0;
            return;
        }
        let cols = self.overview_cols.max(1);
        let cur = self.overview_selected.min(len - 1);
        let next = match dir {
            FocusDir::Left => cur.saturating_sub(1),
            FocusDir::Right => (cur + 1).min(len - 1),
            FocusDir::Up => cur.saturating_sub(cols),
            FocusDir::Down => (cur + cols).min(len - 1)
        };
        self.overview_selected = next;
    }

    /// Opens the selected overview card: a service card switches to its
    /// resource list, a project card opens that project's resources.
    pub fn enter_overview(&mut self) {
        let cards = self.overview_cards();
        let Some(card) = cards.get(self.overview_selected) else {
            return;
        };
        match card.kind {
            OverviewKind::Service(tab) => {
                self.active_tab = tab;
                self.selected = 0;
            }
            OverviewKind::Project(index) => {
                self.active_tab = ResourceTab::Projects;
                self.selected = index;
            }
        }
        self.view = DashboardView::Resources;
        self.focus = super::Focus::ResourceList;
        self.focus_active = false;
        self.filter.clear();
        self.filter_editing = false;
        self.detail_scroll = 0;
    }

    /// Returns to the overview landing screen from the resource view.
    pub fn show_overview(&mut self) {
        self.view = DashboardView::Overview;
        self.focus_active = false;
    }
}
