// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Main draw function — composes all widgets into the terminal layout.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph}
};

use crate::tui::{
    app::{App, Focus, NavLevel, ResourceTab},
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

    if app.action_menu_open() {
        render_action_menu(frame, size, app, &palette);
    }

    if app.awaiting_confirm() {
        render_confirm(frame, size, app, &palette);
    }
}

/// Renders the context action menu for the selected server.
///
/// Lists the available actions with the highlighted one marked; destructive
/// actions are shown in the error color with a warning glyph.
fn render_action_menu(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let Some(menu) = app.action_menu() else {
        return;
    };

    let lines: Vec<Line> = menu
        .actions
        .iter()
        .enumerate()
        .map(|(idx, action)| {
            let selected = idx == menu.selected;
            let marker = if selected { "\u{25B6} " } else { "  " };
            let color = if action.is_destructive() {
                palette.error
            } else if selected {
                palette.accent
            } else {
                palette.fg
            };
            let mut style = Style::default().fg(color);
            if selected {
                style = style.add_modifier(Modifier::BOLD);
            }
            let mut spans = vec![
                Span::styled(marker, Style::default().fg(palette.accent)),
                Span::styled(format!("{:<10}", action.label()), style),
            ];
            if action.is_destructive() {
                spans.push(Span::styled(
                    "\u{26A0}",
                    Style::default().fg(palette.error)
                ));
            }
            Line::from(spans)
        })
        .collect();

    let kind = ResourceTab::names()
        .get(menu.tab.index())
        .copied()
        .unwrap_or("resource");
    let title = format!(
        " Actions: {kind} '{}' (id {}) ",
        menu.resource_name, menu.resource_id
    );
    let width = u16::try_from(title.len() + 4)
        .unwrap_or(40)
        .clamp(28, area.width.saturating_sub(4));
    let height = u16::try_from(menu.actions.len()).unwrap_or(5) + 2;
    let popup = Rect {
        x:      area.width.saturating_sub(width) / 2,
        y:      area.height.saturating_sub(height) / 2,
        width,
        height
    };

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(palette.accent))
                .title(Line::from(Span::styled(
                    title,
                    Style::default()
                        .fg(palette.title)
                        .add_modifier(Modifier::BOLD)
                )))
        )
        .alignment(Alignment::Left)
        .style(Style::default().bg(palette.bg));

    frame.render_widget(Clear, popup);
    frame.render_widget(paragraph, popup);
}

/// Renders the action confirmation modal centered on screen.
///
/// Shows the verb, target server, an irreversibility warning for
/// destructive actions, and the confirm/cancel keys.
fn render_confirm(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let Some(pending) = app.pending_action() else {
        return;
    };

    let accent = if pending.kind.is_destructive() {
        palette.error
    } else {
        palette.warning
    };

    let mut lines = vec![
        Line::from(Span::styled(
            format!(
                " {} '{}' (id {})?",
                pending.kind.label(),
                pending.resource_name,
                pending.resource_id
            ),
            Style::default()
                .fg(palette.fg)
                .add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
    ];
    if pending.kind.is_destructive() {
        lines.push(Line::from(Span::styled(
            " This action cannot be undone.",
            Style::default().fg(palette.error)
        )));
        lines.push(Line::from(""));
    }
    lines.push(Line::from(vec![
        Span::styled(
            " [y] ",
            Style::default().fg(accent).add_modifier(Modifier::BOLD)
        ),
        Span::styled("confirm    ", Style::default().fg(palette.dim)),
        Span::styled(
            "[n] ",
            Style::default()
                .fg(palette.fg)
                .add_modifier(Modifier::BOLD)
        ),
        Span::styled("cancel", Style::default().fg(palette.dim)),
    ]));

    let width = 54u16.min(area.width.saturating_sub(4));
    let height = u16::try_from(lines.len()).unwrap_or(4) + 2;
    let popup = Rect {
        x:      area.width.saturating_sub(width) / 2,
        y:      area.height.saturating_sub(height) / 2,
        width,
        height
    };

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(accent))
                .title(Line::from(Span::styled(
                    " Confirm ",
                    Style::default()
                        .fg(palette.title)
                        .add_modifier(Modifier::BOLD)
                )))
        )
        .alignment(Alignment::Left)
        .style(Style::default().bg(palette.bg));

    frame.render_widget(Clear, popup);
    frame.render_widget(paragraph, popup);
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
            Focus::ResourceList => "k/j select  Enter actions  Esc back",
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
