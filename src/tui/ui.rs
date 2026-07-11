// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Main draw function — composes all widgets into the terminal layout.

mod overlays;
mod status_bar;

use overlays::{render_action_menu, render_confirm, render_create_form};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState}
};
use rust_i18n::t;
use status_bar::render_status_bar;

use crate::tui::{
    app::{App, DrillView, Focus},
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

    let show_account = app.is_widget_enabled("account") && size.height >= 16;
    let show_events = app.is_widget_enabled("events") && size.height >= 24;

    let mut constraints = Vec::with_capacity(5);
    if show_account {
        constraints.push(Constraint::Length(3));
    }
    constraints.push(Constraint::Length(2));
    constraints.push(Constraint::Min(8));
    if show_events {
        constraints.push(Constraint::Length(7));
    }
    constraints.push(Constraint::Length(3));

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(size);

    let mut idx = 0;
    if show_account {
        AccountWidget::new(true).render(frame, main_chunks[idx], app);
        idx += 1;
    }

    ResourceTabsWidget::new(true).render(frame, main_chunks[idx], app);
    idx += 1;

    render_content(frame, main_chunks[idx], app, &palette);
    idx += 1;

    if show_events {
        crate::tui::widgets::events::render(frame, main_chunks[idx], app);
        idx += 1;
    }

    render_status_bar(frame, main_chunks[idx], app, &palette);

    if app.show_help {
        HelpWidget::new().render(frame, size, app);
    }

    if app.action_menu_open() {
        render_action_menu(frame, size, app, &palette);
    }

    if app.awaiting_confirm() {
        render_confirm(frame, size, app, &palette);
    }

    if let Some(form) = app.create_form.as_ref() {
        render_create_form(frame, size, form, &palette);
    }

    if let Some(cp) = app.palette.as_ref() {
        crate::tui::command_palette::render(frame, size, &palette, cp);
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
    if let Some(view) = app.drill_view() {
        render_drill(frame, area, view, palette);
        return;
    }

    let list_pct = app.list_width_pct.clamp(20, 70);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(list_pct),
            Constraint::Percentage(100 - list_pct)
        ])
        .split(area);

    if app.is_loading {
        crate::tui::widgets::skeleton::render(
            frame,
            chunks[0],
            palette,
            &t!("ui.skeleton_resources"),
            8,
            app.anim_tick
        );
        crate::tui::widgets::skeleton::render(
            frame,
            chunks[1],
            palette,
            &t!("ui.skeleton_details"),
            6,
            app.anim_tick
        );
        return;
    }

    let border_for = |target: Focus| {
        if app.focus != target {
            palette.border
        } else if app.focus_active {
            palette.success
        } else {
            palette.accent
        }
    };
    let list_border_color = border_for(Focus::ResourceList);
    let detail_border_color = border_for(Focus::Details);

    if area.width < 56 {
        crate::tui::widgets::resource_list::render(frame, area, app, list_border_color);
        return;
    }

    let wide = area.width >= 100;
    let show_stats = wide && app.is_widget_enabled("stats");
    let show_token = wide && app.is_widget_enabled("token_info");

    crate::tui::widgets::resource_list::render(frame, chunks[0], app, list_border_color);

    if show_stats || show_token {
        let right = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
            .split(chunks[1]);
        crate::tui::widgets::details::render(frame, right[0], app, detail_border_color);
        render_info_column(frame, right[1], app, show_stats, show_token);
    } else {
        crate::tui::widgets::details::render(frame, chunks[1], app, detail_border_color);
    }
}

/// Renders the optional right-hand info column with the Stats and Token Info
/// widgets, stacked according to which are enabled.
fn render_info_column(frame: &mut Frame, area: Rect, app: &App, stats: bool, token: bool) {
    use crate::tui::widgets::{Widget, stats::StatsWidget, token_info::TokenInfoWidget};

    match (stats, token) {
        (true, true) => {
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(area);
            StatsWidget::new(true).render(frame, rows[0], app);
            TokenInfoWidget::new(true).render(frame, rows[1], app);
        }
        (true, false) => StatsWidget::new(true).render(frame, area, app),
        (false, true) => TokenInfoWidget::new(true).render(frame, area, app),
        (false, false) => {}
    }
}

/// Renders the drill-in view listing the resources contained in a parent.
fn render_drill(frame: &mut Frame, area: Rect, view: &DrillView, palette: &Palette) {
    let items: Vec<ListItem> = if view.items.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            t!("ui.drill_empty").to_string(),
            Style::default().fg(palette.dim)
        )))]
    } else {
        view.items
            .iter()
            .map(|item| {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{:<11}", item.kind),
                        Style::default().fg(palette.dim)
                    ),
                    Span::styled(
                        item.name.clone(),
                        Style::default().fg(palette.fg).add_modifier(Modifier::BOLD)
                    ),
                    Span::raw("   "),
                    Span::styled(item.detail.clone(), Style::default().fg(palette.accent)),
                ]))
            })
            .collect()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(palette.accent))
                .title(Line::from(Span::styled(
                    t!("ui.drill_title", title => view.title).to_string(),
                    Style::default()
                        .fg(palette.title)
                        .add_modifier(Modifier::BOLD)
                )))
        )
        .highlight_style(
            Style::default()
                .fg(palette.bg)
                .bg(palette.accent)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol("\u{2503} ");

    let mut state = ListState::default();
    state.select(Some(view.selected));
    frame.render_stateful_widget(list, area, &mut state);
}
