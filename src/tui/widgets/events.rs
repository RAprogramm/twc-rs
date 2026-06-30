// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Events widget — a live log of actions, refreshes, and load errors.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem}
};

use crate::tui::app::{App, LogLevel};

/// Renders the event log panel, newest entries at the bottom.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let palette = app.theme.palette();
    let capacity = usize::from(area.height.saturating_sub(2)).max(1);
    let skip = app.logs.len().saturating_sub(capacity);

    let items: Vec<ListItem> = app
        .logs
        .iter()
        .skip(skip)
        .map(|entry| {
            let (icon, color) = match entry.level {
                LogLevel::Info => ("\u{2022}", palette.dim),
                LogLevel::Success => ("\u{2713}", palette.success),
                LogLevel::Warn => ("\u{25B2}", palette.warning),
                LogLevel::Error => ("\u{2717}", palette.error)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{icon} "), Style::default().fg(color)),
                Span::styled(entry.text.clone(), Style::default().fg(palette.fg)),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(palette.border))
            .title(Line::from(Span::styled(
                " Events ",
                Style::default()
                    .fg(palette.title)
                    .add_modifier(Modifier::BOLD)
            )))
    );
    frame.render_widget(list, area);
}

/// Widget wrapper for the event log panel.
pub struct EventsWidget {
    enabled: bool
}

impl EventsWidget {
    /// Creates a new events widget with the given enabled state.
    #[must_use]
    pub const fn new(enabled: bool) -> Self {
        Self {
            enabled
        }
    }
}

impl crate::tui::widgets::Widget for EventsWidget {
    fn id(&self) -> &'static str {
        "events"
    }

    fn name(&self) -> &'static str {
        "Events"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        render(frame, area, app);
    }
}
