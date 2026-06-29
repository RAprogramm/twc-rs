// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Resource tabs widget — renders horizontal resource category tabs.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Tabs}
};

use crate::tui::app::{App, ResourceTab};

/// Renders the resource category tabs as a horizontal bar.
///
/// # Overview
///
/// Displays all resource tabs (Servers, Databases, S3, etc.) in a single
/// line using ratatui's `Tabs` widget. The active tab is highlighted with
/// the theme's `tab_active` color and marked with a `▶` indicator.
pub struct ResourceTabsWidget {
    enabled: bool
}

impl ResourceTabsWidget {
    /// Creates a new resource tabs widget with enabled state.
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

impl crate::tui::widgets::Widget for ResourceTabsWidget {
    fn id(&self) -> &'static str {
        "resource_tabs"
    }

    fn name(&self) -> &'static str {
        "Resource Tabs"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        let palette = app.theme.palette();
        let tab_names: Vec<Line> = ResourceTab::names()
            .iter()
            .map(|name| {
                if *name == ResourceTab::names()[app.active_tab.index()] {
                    Line::from(Span::styled(
                        format!("▶ {name}"),
                        Style::default()
                            .fg(palette.tab_active)
                            .add_modifier(Modifier::BOLD)
                    ))
                } else {
                    Line::from(Span::raw(*name))
                }
            })
            .collect();

        let tab_widget = Tabs::new(tab_names)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(palette.border))
            )
            .select(app.active_tab.index())
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
