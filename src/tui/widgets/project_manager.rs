// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Project Manager widget — renders filterable project tabs.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Tabs}
};

use crate::tui::app::{App, ProjectSummary};

/// A single filter tab in the Project Manager.
///
/// # Overview
///
/// Each tab defines a display label (icon + name) and a filter string
/// that determines which projects are shown when the tab is active.
#[derive(Debug, Clone)]
pub struct ProjectTab {
    /// Display name shown alongside the icon.
    pub name:   String,
    /// Unicode icon character rendered before the name.
    pub icon:   char,
    /// Filter value used to select projects ("all", "production", etc.).
    pub filter: String
}

impl ProjectTab {
    /// Creates a new project tab.
    ///
    /// # Arguments
    ///
    /// * `name` - Display name for the tab.
    /// * `icon` - Unicode icon character.
    /// * `filter` - Filter string for project selection.
    ///
    /// # Examples
    ///
    /// ```
    /// use twc_rs::tui::widgets::project_manager::ProjectTab;
    ///
    /// let tab = ProjectTab::new("production", '\u{1F525}', "production");
    /// assert_eq!(tab.name, "production");
    /// assert_eq!(tab.icon, '\u{1F525}');
    /// assert_eq!(tab.filter, "production");
    /// ```
    pub fn new(name: &str, icon: char, filter: &str) -> Self {
        Self {
            name: name.to_string(),
            icon,
            filter: filter.to_string()
        }
    }

    /// Returns the full label rendered in the tab bar.
    ///
    /// # Returns
    ///
    /// A string combining the icon and name, e.g. "🔥 production".
    pub fn label(&self) -> String {
        format!("{} {}", self.icon, self.name)
    }
}

/// Default tabs initialized for the Project Manager.
///
/// # Returns
///
/// A vector containing All, production, staging, and backup tabs.
fn default_tabs() -> Vec<ProjectTab> {
    vec![
        ProjectTab::new("All", '\u{1F4CA}', "all"),
        ProjectTab::new("production", '\u{1F525}', "production"),
        ProjectTab::new("staging", '\u{1F9EA}', "staging"),
        ProjectTab::new("backup", '\u{1F4BE}', "backup"),
    ]
}

/// State for the project filter tabs.
///
/// # Overview
///
/// Holds the list of tabs and tracks which one is currently active.
/// The active tab determines which subset of projects is displayed.
#[derive(Debug, Clone)]
pub struct ProjectManager {
    /// All available filter tabs.
    pub tabs:       Vec<ProjectTab>,
    /// Index of the currently active tab.
    pub active_tab: usize
}

impl ProjectManager {
    /// Creates a new `ProjectManager` with default tabs.
    ///
    /// # Examples
    ///
    /// ```
    /// use twc_rs::tui::widgets::project_manager::ProjectManager;
    ///
    /// let pm = ProjectManager::new();
    /// assert_eq!(pm.tabs.len(), 4);
    /// assert_eq!(pm.active_tab, 0);
    /// ```
    pub fn new() -> Self {
        Self {
            tabs:       default_tabs(),
            active_tab: 0
        }
    }

    /// Cycles to the next tab, wrapping around.
    #[allow(dead_code)]
    pub const fn next_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        self.active_tab = (self.active_tab + 1) % self.tabs.len();
    }

    /// Cycles to the previous tab, wrapping around.
    #[allow(dead_code)]
    pub fn previous_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        self.active_tab = self
            .active_tab
            .checked_sub(1)
            .unwrap_or_else(|| self.tabs.len().saturating_sub(1));
    }

    /// Returns the filter string for the active tab.
    ///
    /// # Returns
    ///
    /// The filter string, or `"all"` if no tabs exist.
    #[allow(dead_code)]
    pub fn active_filter(&self) -> &str {
        self.tabs
            .get(self.active_tab)
            .map_or("all", |t| t.filter.as_str())
    }

    /// Returns the filtered list of projects for the active tab.
    ///
    /// # Arguments
    ///
    /// * `projects` - The full list of projects.
    ///
    /// # Returns
    ///
    /// All projects when the active filter is `"all"`, otherwise a
    /// vector containing only projects whose name contains the filter
    /// string (case-insensitive).
    #[allow(dead_code)]
    pub fn filtered_projects(&self, projects: &[ProjectSummary]) -> Vec<ProjectSummary> {
        let filter = self.active_filter();
        if filter == "all" {
            projects.to_vec()
        } else {
            projects
                .iter()
                .filter(|p| p.name.to_lowercase().contains(&filter.to_lowercase()))
                .cloned()
                .collect()
        }
    }
}

impl Default for ProjectManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Renders the project filter tabs as a horizontal bar.
///
/// # Overview
///
/// Displays all tabs in a single line using ratatui's `Tabs` widget.
/// The active tab is highlighted with the theme's `tab_active` color.
pub struct ProjectManagerWidget {
    enabled: bool
}

impl ProjectManagerWidget {
    /// Creates a new project manager widget with enabled state.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether the widget is initially visible.
    pub const fn new(enabled: bool) -> Self {
        Self {
            enabled
        }
    }
}

impl crate::tui::widgets::Widget for ProjectManagerWidget {
    fn id(&self) -> &'static str {
        "project_manager"
    }

    fn name(&self) -> &'static str {
        "Project Manager"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        let palette = app.theme.palette();
        let labels = app.project_manager.tabs.iter().map(|tab| {
            let label = tab.label();
            Line::from(label)
        });

        let tab_widget = Tabs::new(labels)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .style(Style::default().fg(palette.border))
            )
            .select(Some(app.project_manager.active_tab))
            .divider(Span::raw("  "))
            .style(Style::default().fg(palette.tab_inactive))
            .highlight_style(
                Style::default()
                    .fg(palette.tab_active)
                    .add_modifier(Modifier::BOLD)
            );

        frame.render_widget(tab_widget, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::{app::ProjectSummary, widgets::Widget};

    fn sample_projects() -> Vec<ProjectSummary> {
        vec![
            ProjectSummary {
                id:           1,
                name:         "production-api".to_string(),
                server_count: 5
            },
            ProjectSummary {
                id:           2,
                name:         "staging-web".to_string(),
                server_count: 2
            },
            ProjectSummary {
                id:           3,
                name:         "backup-service".to_string(),
                server_count: 1
            },
            ProjectSummary {
                id:           4,
                name:         "production-db".to_string(),
                server_count: 3
            },
        ]
    }

    #[test]
    fn project_tab_new_creates_correct_tab() {
        let tab = ProjectTab::new("test", '🧪', "test");
        assert_eq!(tab.name, "test");
        assert_eq!(tab.icon, '\u{1F9EA}');
        assert_eq!(tab.filter, "test");
    }

    #[test]
    fn project_tab_label_includes_icon() {
        let tab = ProjectTab::new("prod", '\u{1F525}', "production");
        assert_eq!(tab.label(), "\u{1F525} prod");
    }

    #[test]
    fn project_tab_label_all() {
        let tab = ProjectTab::new("All", '\u{1F4CA}', "all");
        assert_eq!(tab.label(), "\u{1F4CA} All");
    }

    #[test]
    fn default_tabs_has_four_entries() {
        let tabs = default_tabs();
        assert_eq!(tabs.len(), 4);
    }

    #[test]
    fn default_tabs_first_is_all() {
        let tabs = default_tabs();
        assert_eq!(tabs[0].filter, "all");
    }

    #[test]
    fn default_tabs_has_production() {
        let tabs = default_tabs();
        let prod = tabs.iter().find(|t| t.filter == "production");
        assert!(prod.is_some());
    }

    #[test]
    fn default_tabs_has_staging() {
        let tabs = default_tabs();
        let staging = tabs.iter().find(|t| t.filter == "staging");
        assert!(staging.is_some());
    }

    #[test]
    fn default_tabs_has_backup() {
        let tabs = default_tabs();
        let backup = tabs.iter().find(|t| t.filter == "backup");
        assert!(backup.is_some());
    }

    #[test]
    fn project_manager_new_has_default_tabs() {
        let pm = ProjectManager::new();
        assert_eq!(pm.tabs.len(), 4);
        assert_eq!(pm.active_tab, 0);
    }

    #[test]
    fn project_manager_next_tab_cycles() {
        let mut pm = ProjectManager::new();
        pm.next_tab();
        assert_eq!(pm.active_tab, 1);
        pm.next_tab();
        assert_eq!(pm.active_tab, 2);
        pm.next_tab();
        assert_eq!(pm.active_tab, 3);
        pm.next_tab();
        assert_eq!(pm.active_tab, 0);
    }

    #[test]
    fn project_manager_previous_tab_wraps() {
        let mut pm = ProjectManager::new();
        pm.previous_tab();
        assert_eq!(pm.active_tab, 3);
    }

    #[test]
    fn project_manager_active_filter_returns_filter() {
        let mut pm = ProjectManager::new();
        assert_eq!(pm.active_filter(), "all");
        pm.active_tab = 1;
        assert_eq!(pm.active_filter(), "production");
        pm.active_tab = 2;
        assert_eq!(pm.active_filter(), "staging");
    }

    #[test]
    fn project_manager_filtered_projects_all_returns_all() {
        let pm = ProjectManager::new();
        let projects = sample_projects();
        let filtered = pm.filtered_projects(&projects);
        assert_eq!(filtered.len(), 4);
    }

    #[test]
    fn project_manager_filtered_projects_production_matches() {
        let mut pm = ProjectManager::new();
        pm.active_tab = 1;
        let projects = sample_projects();
        let filtered = pm.filtered_projects(&projects);
        assert_eq!(filtered.len(), 2);
        assert!(filtered[0].name.contains("production"));
        assert!(filtered[1].name.contains("production"));
    }

    #[test]
    fn project_manager_filtered_projects_staging_matches() {
        let mut pm = ProjectManager::new();
        pm.active_tab = 2;
        let projects = sample_projects();
        let filtered = pm.filtered_projects(&projects);
        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].name.contains("staging"));
    }

    #[test]
    fn project_manager_filtered_projects_no_match_returns_empty() {
        let mut pm = ProjectManager::new();
        pm.active_tab = 3;
        let projects = sample_projects();
        // "backup" filter matches "backup-service" in sample data,
        // so we test with a custom ProjectManager using a non-matching filter.
        let pm_custom = ProjectManager {
            tabs:       vec![ProjectTab::new("custom", '\u{2753}', "xyznonexistent")],
            active_tab: 0
        };
        let filtered = pm_custom.filtered_projects(&projects);
        assert!(filtered.is_empty());
    }

    #[test]
    fn project_manager_filtered_projects_empty_input() {
        let pm = ProjectManager::new();
        let projects: Vec<ProjectSummary> = vec![];
        let filtered = pm.filtered_projects(&projects);
        assert!(filtered.is_empty());
    }

    #[test]
    fn project_manager_widget_id() {
        let widget = ProjectManagerWidget::new(true);
        assert_eq!(widget.id(), "project_manager");
    }

    #[test]
    fn project_manager_widget_name() {
        let widget = ProjectManagerWidget::new(true);
        assert_eq!(widget.name(), "Project Manager");
    }

    #[test]
    fn project_manager_widget_enabled() {
        let widget = ProjectManagerWidget::new(true);
        assert!(widget.enabled());
    }

    #[test]
    fn project_manager_widget_disabled() {
        let widget = ProjectManagerWidget::new(false);
        assert!(!widget.enabled());
    }

    #[test]
    fn project_manager_widget_toggle() {
        let mut widget = ProjectManagerWidget::new(true);
        widget.toggle();
        assert!(!widget.enabled());
        widget.toggle();
        assert!(widget.enabled());
    }

    #[test]
    fn project_manager_widget_trait_object_is_send() {
        let widget: Box<dyn Widget + Send> = Box::new(ProjectManagerWidget::new(true));
        assert_eq!(widget.id(), "project_manager");
    }

    #[test]
    #[allow(clippy::redundant_clone)]
    fn project_manager_clone_preserves_state() {
        let mut pm = ProjectManager::new();
        pm.active_tab = 2;
        let pm_clone = pm.clone();
        assert_eq!(pm_clone.active_tab, 2);
        assert_eq!(pm_clone.tabs.len(), 4);
    }

    #[test]
    #[allow(clippy::redundant_clone)]
    fn project_tab_clone_preserves_state() {
        let tab = ProjectTab::new("test", '🧪', "test");
        let tab_clone = tab.clone();
        assert_eq!(tab_clone.name, "test");
        assert_eq!(tab_clone.icon, '\u{1F9EA}');
        assert_eq!(tab_clone.filter, "test");
    }

    #[test]
    fn project_manager_empty_tabs_next_does_not_panic() {
        let mut pm = ProjectManager {
            tabs:       vec![],
            active_tab: 0
        };
        pm.next_tab();
        assert_eq!(pm.active_tab, 0);
    }

    #[test]
    fn project_manager_empty_tabs_previous_does_not_panic() {
        let mut pm = ProjectManager {
            tabs:       vec![],
            active_tab: 0
        };
        pm.previous_tab();
        assert_eq!(pm.active_tab, 0);
    }

    #[test]
    fn project_manager_empty_tabs_active_filter_returns_all() {
        let pm = ProjectManager {
            tabs:       vec![],
            active_tab: 0
        };
        assert_eq!(pm.active_filter(), "all");
    }

    #[test]
    fn default_tabs_default_icons() {
        let tabs = default_tabs();
        assert_eq!(tabs[0].icon, '\u{1F4CA}');
        assert_eq!(tabs[1].icon, '\u{1F525}');
        assert_eq!(tabs[2].icon, '\u{1F9EA}');
        assert_eq!(tabs[3].icon, '\u{1F4BE}');
    }

    #[test]
    fn filtered_projects_case_insensitive() {
        let mut pm = ProjectManager::new();
        pm.active_tab = 1;
        let projects = vec![ProjectSummary {
            id:           1,
            name:         "PRODUCTION-api".to_string(),
            server_count: 1
        }];
        let filtered = pm.filtered_projects(&projects);
        assert_eq!(filtered.len(), 1);
    }
}
