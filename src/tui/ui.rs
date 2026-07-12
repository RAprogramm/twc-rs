// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Main draw function — composes all widgets into the terminal layout.

mod overlays;
mod status_bar;

use overlays::{render_action_menu, render_confirm, render_create_form};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect}
};
use rust_i18n::t;
use status_bar::render_status_bar;

use crate::tui::{
    app::{App, DrillView, NavKind, Pane},
    themes::Palette,
    widgets::{Widget, account::AccountWidget, help::HelpWidget}
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

    let mut constraints = Vec::with_capacity(3);
    if show_account {
        constraints.push(Constraint::Length(3));
    }
    constraints.push(Constraint::Min(8));
    constraints.push(Constraint::Length(3));

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(size);

    let mut idx = 0;
    if show_account {
        AccountWidget::new(true).render(frame, rows[idx], app);
        idx += 1;
    }

    let sidebar_w = crate::tui::widgets::sidebar::width_for(size.width, app.nav_longest_label());
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(sidebar_w), Constraint::Min(10)])
        .split(rows[idx]);
    idx += 1;

    crate::tui::widgets::sidebar::render(frame, body[0], app, &palette);

    if show_events {
        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(8), Constraint::Length(7)])
            .split(body[1]);
        render_content(frame, right[0], app, &palette);
        crate::tui::widgets::events::render(frame, right[1], app);
    } else {
        render_content(frame, body[1], app, &palette);
    }

    render_status_bar(frame, rows[idx], app, &palette);

    draw_modals(frame, size, app, &palette);
}

/// Renders the floating overlays (help, menus, dialogs, palette) shared by all
/// views.
fn draw_modals(frame: &mut Frame, size: Rect, app: &App, palette: &Palette) {
    if app.show_help {
        HelpWidget::new().render(frame, size, app);
    }

    if app.action_menu_open() {
        render_action_menu(frame, size, app, palette);
    }

    if app.awaiting_confirm() {
        render_confirm(frame, size, app, palette);
    }

    if let Some(form) = app.create_form.as_ref() {
        render_create_form(frame, size, form, palette);
    }

    if let Some(cp) = app.palette.as_ref() {
        crate::tui::command_palette::render(frame, size, palette, cp);
    }
}

/// Renders the content pane: an opened project's contents, a highlighted
/// project's local preview, or the selected service's card grid.
fn render_content(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let border = if app.pane == Pane::Content {
        palette.accent
    } else {
        palette.border
    };

    if let Some(view) = app.drill_view() {
        let selected = if app.pane == Pane::Content {
            view.selected
        } else {
            usize::MAX
        };
        render_drill(frame, area, app, view, selected, border, palette);
        return;
    }

    if app.is_loading {
        crate::tui::widgets::skeleton::render(
            frame,
            area,
            palette,
            &t!("ui.skeleton_resources"),
            8,
            app.anim_tick
        );
        return;
    }

    if let Some(NavKind::Project(index)) = app.nav_current()
        && let Some(project) = app.projects.get(index)
    {
        render_project_preview(frame, area, app, project, border, palette);
        return;
    }

    crate::tui::widgets::resource_list::render(frame, area, app, border);
}

/// Renders a highlighted (not yet opened) project: its per-type resource
/// counts as cards, built from already loaded data — no fetch needed.
fn render_project_preview(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    project: &crate::tui::app::ProjectSummary,
    border: ratatui::style::Color,
    palette: &Palette
) {
    let cards = crate::tui::widgets::resource_cards::project_preview(project);
    let title = format!(" {} ({}) ", project.name, project.resource_count());
    let selected = if app.pane == Pane::Content {
        app.content_selected()
    } else {
        usize::MAX
    };
    render_project_panel(frame, area, app, &title, &cards, selected, border, palette);
}

/// Renders a bordered project panel: the Projects product header with its
/// Create button on top, then the given cards.
#[allow(clippy::too_many_arguments)]
fn render_project_panel(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    title: &str,
    cards: &[crate::tui::widgets::card_grid::GridCard],
    selected: usize,
    border: ratatui::style::Color,
    palette: &Palette
) {
    use ratatui::{
        style::{Modifier, Style},
        text::{Line, Span},
        widgets::{Block, BorderType, Borders, Paragraph}
    };

    use crate::tui::{app::ResourceTab, widgets::card_grid};

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .title(Line::from(Span::styled(
            title.to_string(),
            Style::default()
                .fg(palette.title)
                .add_modifier(Modifier::BOLD)
        )));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 || inner.width < 6 {
        return;
    }

    let focused = app.pane == Pane::Content;
    let header_h = crate::tui::widgets::service_header::height(ResourceTab::Projects, inner.width)
        .min(inner.height.saturating_sub(1));
    if header_h >= 3 {
        crate::tui::widgets::service_header::render(
            frame,
            Rect::new(inner.x, inner.y, inner.width, header_h),
            ResourceTab::Projects,
            focused && app.content_on_create,
            palette
        );
    }

    let grid = Rect::new(
        inner.x,
        inner.y + header_h,
        inner.width,
        inner.height.saturating_sub(header_h)
    );
    if cards.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                t!("ui.drill_empty").to_string(),
                Style::default().fg(palette.dim)
            ))),
            grid
        );
        return;
    }
    let cols = card_grid::columns(grid.width, card_grid::longest_title(cards));
    let shown = if app.content_on_create {
        usize::MAX
    } else {
        selected
    };
    card_grid::render_grid_in(frame, grid, cards, shown, cols, palette);
}

/// Renders the drill-in view: a project's contents as the shared card grid,
/// under the Projects product header.
fn render_drill(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    view: &DrillView,
    selected: usize,
    border: ratatui::style::Color,
    palette: &Palette
) {
    use crate::tui::widgets::card_grid::GridCard;

    let cards: Vec<GridCard> = view
        .items
        .iter()
        .map(|item| {
            let mut card = GridCard::new(item.name.clone());
            if !item.detail.is_empty() {
                let (color, _) =
                    crate::tui::widgets::resource_list::status_view(&item.detail, palette);
                card = card.status(color, item.detail.clone());
            }
            card.meta(item.kind.clone())
        })
        .collect();

    let title = format!(" {} ({}) ", view.title, view.items.len());
    render_project_panel(frame, area, app, &title, &cards, selected, border, palette);
}
