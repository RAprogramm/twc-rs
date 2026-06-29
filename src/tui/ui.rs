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
    app::{App, Focus, NavLevel},
    themes::Palette,
    widgets::{
        Widget, account::AccountWidget, help::HelpWidget, resource_tabs::ResourceTabsWidget
    }
};

/// Renders the full dashboard into the given frame area.
///
/// Composes the layout into four sections: header (Account widget), resource
/// tabs, content (`ResourceList` and `Details` side by side), and status bar.
/// When help is requested, the Help widget is rendered as an overlay.
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
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(10),
            Constraint::Length(3)
        ])
        .split(size);

    let account_widget = AccountWidget::new(true);
    account_widget.render(frame, main_chunks[0], app);

    let rt_widget = ResourceTabsWidget::new(true);
    rt_widget.render(frame, main_chunks[1], app);

    render_content(frame, main_chunks[2], app, &palette);
    render_status_bar(frame, main_chunks[3], app, &palette);

    if app.show_help {
        let help_widget = HelpWidget::new();
        help_widget.render(frame, size, app);
    }
}

/// Renders the content area with resource list and details side by side.
///
/// The focused panel receives a highlighted border using `palette.accent`.
/// Non-focused panels use `palette.border`.
///
/// # Arguments
///
/// * `frame` - The render frame.
/// * `area` - The content area rectangle.
/// * `app` - The application state.
/// * `palette` - The theme color palette.
fn render_content(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    if app.is_loading {
        let spinner = crate::tui::widgets::spinner::current_frame();
        let text = Line::from(vec![spinner, Span::raw(" Loading resources...")]);
        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(" Loading "))
            )
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    let list_border_color = if app.focus == Focus::ResourceList {
        palette.accent
    } else {
        palette.border
    };

    let detail_border_color = if app.focus == Focus::Details {
        palette.accent
    } else {
        palette.border
    };

    crate::tui::widgets::WidgetRegistry::render_side_by_side(
        frame,
        chunks[0],
        chunks[1],
        app,
        list_border_color,
        detail_border_color
    );
}

/// Renders the status bar with mode indicator and available keys.
///
/// # Arguments
///
/// * `frame` - The render frame.
/// * `area` - The status bar area rectangle.
/// * `app` - The application state.
/// * `palette` - The theme color palette.
fn render_status_bar(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let mode = match app.nav_level {
        NavLevel::Overview => "overview",
        NavLevel::Inner => "inner"
    };
    let focus = app.focus.label();
    let keys = match app.nav_level {
        NavLevel::Overview => match app.focus {
            Focus::ResourceTabs => "Tab cycle  hjkl navigate  Enter drill in  Q quit",
            Focus::ResourceList => "hjkl navigate  Enter drill in  ? help  Q quit",
            Focus::Details => "hjkl navigate  ? help  Q quit"
        },
        NavLevel::Inner => match app.focus {
            Focus::ResourceTabs => "k/j cycle tabs  Esc back",
            Focus::ResourceList => "k/j select  Enter details  Esc back",
            Focus::Details => "k/j scroll  Esc back"
        }
    };

    let left = format!("{mode}: {focus}  {keys}");
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
