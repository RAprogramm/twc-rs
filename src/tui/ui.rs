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
    text::{Line, Span}
};
use rust_i18n::t;
use status_bar::render_status_bar;

use crate::tui::{
    app::{App, DrillView},
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

    if app.view == crate::tui::app::DashboardView::Overview {
        draw_overview(frame, size, app, &palette, show_account);
        return draw_modals(frame, size, app, &palette);
    }

    let show_events = app.is_widget_enabled("events") && size.height >= 24;
    let show_tabs = !app.drill_open();

    let mut constraints = Vec::with_capacity(5);
    if show_account {
        constraints.push(Constraint::Length(3));
    }
    if show_tabs {
        constraints.push(Constraint::Length(2));
    }
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

    if show_tabs {
        ResourceTabsWidget::new(true).render(frame, main_chunks[idx], app);
        idx += 1;
    }

    render_content(frame, main_chunks[idx], app, &palette);
    idx += 1;

    if show_events {
        crate::tui::widgets::events::render(frame, main_chunks[idx], app);
        idx += 1;
    }

    render_status_bar(frame, main_chunks[idx], app, &palette);

    draw_modals(frame, size, app, &palette);
}

/// Renders the overview landing screen: account header, the card zones, and the
/// status bar.
fn draw_overview(frame: &mut Frame, size: Rect, app: &App, palette: &Palette, show_account: bool) {
    let mut constraints = Vec::with_capacity(4);
    if show_account {
        constraints.push(Constraint::Length(3));
    }
    if app.is_loading {
        constraints.push(Constraint::Length(1));
    }
    constraints.push(Constraint::Min(6));
    constraints.push(Constraint::Length(3));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(size);

    let mut idx = 0;
    if show_account {
        AccountWidget::new(true).render(frame, chunks[idx], app);
        idx += 1;
    }
    if app.is_loading {
        render_loading_banner(frame, chunks[idx], app, palette);
        idx += 1;
    }
    crate::tui::widgets::overview::render(frame, chunks[idx], app, *palette);
    idx += 1;
    render_status_bar(frame, chunks[idx], app, palette);
}

/// Spinner frames for the loading banner, advanced by the animation tick.
const SPINNER: [&str; 10] = [
    "\u{280B}", "\u{2819}", "\u{2839}", "\u{2838}", "\u{283C}", "\u{2834}", "\u{2826}",
    "\u{2827}", "\u{2807}", "\u{280F}"
];

/// Renders a one-line animated loading indicator so it is obvious the dashboard
/// is still fetching data.
fn render_loading_banner(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let frame_idx = usize::try_from(app.anim_tick % SPINNER.len() as u64).unwrap_or(0);
    let spinner = SPINNER[frame_idx];
    let line = Line::from(vec![
        Span::raw("  "),
        Span::styled(
            spinner,
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD)
        ),
        Span::raw(" "),
        Span::styled(
            t!("overview.loading").to_string(),
            Style::default().fg(palette.dim)
        ),
    ]);
    frame.render_widget(ratatui::widgets::Paragraph::new(line), area);
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
        if app.drill_loading {
            crate::tui::widgets::skeleton::render(
                frame,
                area,
                palette,
                &view.title,
                8,
                app.anim_tick
            );
        } else {
            render_drill(frame, area, view, palette);
        }
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

    crate::tui::widgets::resource_list::render(frame, area, app, palette.accent);
}

/// Renders the drill-in view: a project's contents as the shared card grid.
fn render_drill(frame: &mut Frame, area: Rect, view: &DrillView, palette: &Palette) {
    use crate::tui::widgets::card_grid::{self, GridCard};

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
    let cols = card_grid::columns(
        area.width.saturating_sub(2),
        card_grid::longest_title(&cards)
    );
    let empty = t!("ui.drill_empty");
    card_grid::render(
        frame,
        area,
        &title,
        &cards,
        view.selected,
        cols,
        empty.as_ref(),
        palette.accent,
        palette
    );
}
