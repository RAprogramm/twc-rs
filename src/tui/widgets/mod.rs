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

impl WidgetRegistry {
    /// Creates a new empty widget registry.
    ///
    /// # Examples
    ///
    /// ```
    /// use twc_rs::tui::widgets::WidgetRegistry;
    ///
    /// let registry = WidgetRegistry::new();
    /// assert_eq!(registry.enabled_widgets().len(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            widgets: Vec::new()
        }
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
    ///
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

    /// Returns all enabled widgets.
    ///
    /// # Returns
    ///
    /// A vector of references to enabled widgets.
    pub fn enabled_widgets(&self) -> Vec<&(dyn Widget + Send)> {
        self.widgets
            .iter()
            .filter(|w| w.enabled())
            .map(std::convert::AsRef::as_ref)
            .collect()
    }

    /// Renders all enabled widgets, splitting the area evenly.
    ///
    /// # Arguments
    ///
    /// * `frame` - The render frame.
    /// * `area` - The total area to split among widgets.
    /// * `app` - The application state.
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    pub fn render_all(&self, frame: &mut Frame, area: Rect, app: &App) {
        let enabled = self.enabled_widgets();
        let count = enabled.len();
        if count == 0 {
            return;
        }

        let rows = (count as f64).sqrt().ceil() as usize;
        let cols = count.div_ceil(rows);

        let h = area.height / rows as u16;
        let w = area.width / cols as u16;

        let mut idx = 0usize;
        for r in 0..rows {
            for c in 0..cols {
                if idx < count {
                    let x = area.x + (c as u16) * w;
                    let y = area.y + (r as u16) * h;
                    let inner = Rect {
                        x,
                        y,
                        width: w.saturating_sub(1),
                        height: h.saturating_sub(1)
                    };
                    enabled[idx].render(frame, inner, app);
                }
                idx += 1;
            }
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

pub mod details;
pub mod resource_list;
pub mod spinner;
