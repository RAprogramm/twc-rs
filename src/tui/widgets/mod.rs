// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Composable widget renderers for the TUI dashboard.

use std::fmt;

use ratatui::{Frame, layout::Rect};

use super::app::App;

/// Trait for dashboard widgets.
///
/// # Overview
///
/// Every widget in the dashboard must implement this trait. It provides
/// identity, lifecycle, and rendering capabilities.
///
/// # Example
///
/// ```ignore
/// struct MyWidget {
///     enabled: bool,
/// }
///
/// impl Widget for MyWidget {
///     fn id(&self) -> &str { "my_widget" }
///     fn name(&self) -> &str { "My Widget" }
///     fn enabled(&self) -> bool { self.enabled }
///     fn toggle(&mut self) { self.enabled = !self.enabled; }
///     fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
///         // render logic
///     }
/// }
/// ```
#[allow(dead_code)]
pub trait Widget: Send {
    /// Returns the unique identifier for this widget.
    fn id(&self) -> &str;

    /// Returns the human-readable name of this widget.
    fn name(&self) -> &str;

    /// Returns whether this widget is currently enabled.
    fn enabled(&self) -> bool;

    /// Toggles the enabled state of this widget.
    fn toggle(&mut self);

    /// Renders the widget into the given frame area.
    fn render(&self, frame: &mut Frame, area: Rect, app: &App);
}

/// Registry that owns and manages all dashboard widgets.
///
/// # Overview
///
/// The registry stores widgets as trait objects and provides lookup,
/// toggling, and batch rendering capabilities.
pub struct WidgetRegistry {
    widgets: Vec<Box<dyn Widget + Send + 'static>>
}

#[allow(dead_code)]
impl WidgetRegistry {
    /// Creates a new widget registry pre-populated with the default
    /// dashboard widgets.
    ///
    /// # Examples
    ///
    /// ```
    /// use twc_rs::tui::widgets::WidgetRegistry;
    ///
    /// let registry = WidgetRegistry::new();
    /// assert!(registry.get("account").is_some());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        let mut registry = Self {
            widgets: Vec::new()
        };
        registry.register(Box::new(account::AccountWidget::new(true)));
        registry.register(Box::new(resource_list::ResourceListWidget::new(true)));
        registry.register(Box::new(details::DetailsWidget::new(true)));
        registry.register(Box::new(stats::StatsWidget::new(true)));
        registry.register(Box::new(token_info::TokenInfoWidget::new(true)));
        registry.register(Box::new(events::EventsWidget::new(true)));
        registry.register(Box::new(help::HelpWidget::new()));
        registry
    }

    /// Registers a new widget in the registry.
    ///
    /// # Arguments
    ///
    /// * `widget` - The widget to register.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut registry = WidgetRegistry::new();
    /// registry.register(Box::new(resource_list::ResourceListWidget));
    /// ```
    pub fn register(&mut self, widget: Box<dyn Widget + Send>) {
        self.widgets.push(widget);
    }

    /// Returns a reference to the widget with the given id.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the widget to find.
    ///
    /// # Returns
    ///
    /// `Some(&dyn Widget)` if found, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some(w) = registry.get("resource_list") {
    ///     println!("Found widget: {}", w.name());
    /// }
    /// ```
    // JUSTIFY: Public API method for widget lookup.
    #[allow(dead_code)]
    pub fn get(&self, id: &str) -> Option<&(dyn Widget + Send)> {
        self.widgets
            .iter()
            .find(|w| w.id() == id)
            .map(std::convert::AsRef::as_ref)
    }

    /// Returns a mutable reference to the widget with the given id.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the widget to find.
    ///
    /// # Returns
    /// `Some(&mut dyn Widget)` if found, `None` otherwise.
    pub fn get_mut<'a>(&'a mut self, id: &str) -> Option<&'a mut dyn Widget> {
        for w in &mut self.widgets {
            if w.id() == id {
                return Some(w.as_mut());
            }
        }
        None
    }

    /// Toggles the enabled state of the widget with the given id.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the widget to toggle.
    pub fn toggle(&mut self, id: &str) {
        if let Some(w) = self.widgets.iter_mut().find(|w| w.id() == id) {
            w.toggle();
        }
    }
}

impl Default for WidgetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for WidgetRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WidgetRegistry")
            .field("widget_count", &self.widgets.len())
            .finish()
    }
}

pub mod account;
pub mod card_grid;
pub mod create_panel;
pub mod details;
pub mod events;
pub mod help;
pub mod resource_cards;
pub mod resource_list;
pub mod service_header;
pub mod settings_panel;
pub mod sidebar;
pub mod skeleton;
pub mod stats;
pub mod token_info;
