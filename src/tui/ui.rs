// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Main draw function — composes all widgets into the terminal layout.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph}
};

use crate::tui::{
    app::App,
    themes::Palette,
    widgets::{
        Widget, account::AccountWidget, help::HelpWidget, project_manager::ProjectManagerWidget,
        resource_tabs::ResourceTabsWidget
    }
};

/// Renders the full dashboard into the given frame area using the widget
/// system.
///
/// # Overview
///
/// Composes the layout into five sections: header (Account widget), resource
/// tabs, project tabs, content (`ResourceList` and `Details` side by side),
/// and status bar. When help is requested, the Help widget is rendered as an
/// overlay covering the entire frame.
///
/// # Arguments
///
/// * `frame` - The render frame.
/// * `app` - The application state.
pub fn draw(frame: &mut Frame, app: &App) {
    let size = frame.area();
    let palette = app.theme.palette();

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),    // Header (Account widget)
            Constraint::Length(2),    // Resource tabs
            Constraint::Length(2),    // Project tabs
            Constraint::Min(10),      // Content
            Constraint::Length(3)     // Status bar
        ])
        .split(size);

    let account_widget = AccountWidget::new(true);
    account_widget.render(frame, main_chunks[0], app);

    let rt_widget = ResourceTabsWidget::new(true);
    rt_widget.render(frame, main_chunks[1], app);

    let pm_widget = ProjectManagerWidget::new(true);
    pm_widget.render(frame, main_chunks[2], app);

    render_content(frame, main_chunks[3], app, &palette);
    render_status_bar(frame, main_chunks[4], app, &palette);

    if app.show_help {
        let help_widget = HelpWidget::new();
        help_widget.render(frame, size, app);
    }
}

/// Renders the content area with resource list and details side by side.
///
/// # Arguments
///
/// * `frame` - The render frame.
/// * `area` - The content area rectangle.
/// * `app` - The application state.
/// * `_palette` - The theme palette (reserved for future use).
fn render_content(frame: &mut Frame, area: Rect, app: &App, _palette: &Palette) {
    if app.is_loading {
        let spinner = crate::tui::widgets::spinner::current_frame();
        let text = Line::from(vec![spinner, Span::raw(" Loading resources...")]);
        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(Line::from(" Loading ")))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    app.widgets
        .render_side_by_side(frame, chunks[0], chunks[1], app);
}

/// Renders the status bar with keyboard shortcuts and status messages.
///
/// # Arguments
///
/// * `frame` - The render frame.
/// * `area` - The status bar area rectangle.
/// * `app` - The application state.
/// * `palette` - The theme color palette.
fn render_status_bar(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let focus_str = match app.focus {
        crate::tui::app::Focus::ResourceList => "ResourceList",
        crate::tui::app::Focus::Details => "Details",
        crate::tui::app::Focus::ResourceTabs => "ResourceTabs",
        crate::tui::app::Focus::ProjectTabs => "ProjectTabs"
    };
    let left = format!("focus: {}  k/j up/down  h/l switch  Enter detail  Esc close  Q quit", focus_str);
    let right = match (&app.error_message, &app.status_message) {
        (Some(err), _) => err.clone(),
        (_, Some(msg)) => msg.clone(),
        _ => String::new()
    };
    let color = if app.error_message.is_some() {
        palette.error
    } else {
        palette.success
    };
    let line = Line::from(vec![
        Span::styled(left, Style::default().fg(palette.dim)),
        Span::raw("  "),
        Span::styled(right, Style::default().fg(color)),
    ]);
    let paragraph = Paragraph::new(line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette.border))
    );
    frame.render_widget(paragraph, area);
}
