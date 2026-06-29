// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Resource tabs widget — renders horizontal resource category tabs with scrolling.

use std::fmt::Write;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph}
};

use crate::tui::app::{App, ResourceTab};

/// Renders the resource category tabs as a horizontal bar with dynamic scrolling.
///
/// # Overview
///
/// Displays resource tabs (Servers, Databases, S3, etc.) in a single line.
/// When tabs don't fit, shows `◀`/`▶` indicators and scrolls to keep the
/// active tab visible. The active tab is highlighted with a `▶` indicator.
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

    /// Builds the tab bar text with dynamic scrolling.
    ///
    /// # Arguments
    ///
    /// * `app` - The application state.
    /// * `width` - Available width for the tab bar.
    ///
    /// # Returns
    ///
    /// A `Line` containing the styled tab bar text.
    fn build_tab_bar(app: &App, width: u16) -> Line<'static> {
        let names = ResourceTab::names();
        let active_idx = app.active_tab.index();
        let tab_count = names.len();

        // Calculate how many tabs can fit (each tab needs at least 4 chars + 2 for divider)
        let min_tab_width = 4u16; // Minimum width for a tab name
        let divider_width = 2u16; // Space between tabs
        let _scroll_indicator_width = 2u16; // Width of ◀ or ▶

        let available_width = width.saturating_sub(2); // Subtract border width
        let max_tabs = (available_width + divider_width) / (min_tab_width + divider_width);

        if max_tabs >= tab_count as u16 {
            // All tabs fit, show them all
            let mut spans = Vec::new();
            for (i, name) in names.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::raw("  "));
                }
                if i == active_idx {
                    spans.push(Span::styled(
                        format!("▶ {name}"),
                        Style::default()
                            .fg(crate::tui::themes::Theme::GruvboxDark.palette().tab_active)
                            .add_modifier(Modifier::BOLD)
                    ));
                } else {
                    spans.push(Span::raw(*name));
                }
            }
            return Line::from(spans);
        }

        // Calculate scroll offset to keep active tab visible
        let max_tabs = max_tabs as usize;
        let mut start = 0;
        let mut end = max_tabs;

        // Keep active tab in the middle third of visible area
        let target_pos = max_tabs / 3;
        if active_idx < start + target_pos {
            start = 0;
        } else if active_idx > end - target_pos - 1 {
            start = tab_count - max_tabs;
        } else {
            start = active_idx - target_pos;
        }
        end = start + max_tabs;

        let mut text = String::new();
        let mut is_first = true;

        // Add left scroll indicator if needed
        if start > 0 {
            text.push('◀');
            is_first = false;
        }

        for (i, name) in names.iter().enumerate().skip(start) {
            if !is_first {
                text.push_str("  ");
            }
            is_first = false;

            if i == active_idx {
                write!(text, "▶ {name}").unwrap();
            } else {
                write!(text, "{name}").unwrap();
            }
        }

        // Add right scroll indicator if needed
        if end < tab_count {
            text.push('▶');
        }

        Line::from(Span::raw(text))
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
        let tab_bar = Self::build_tab_bar(app, area.width);

        let paragraph = Paragraph::new(tab_bar).block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(palette.border))
        );

        frame.render_widget(paragraph, area);
    }
}
