// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Main draw function — composes all widgets into the terminal layout.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph}
};
use rust_i18n::t;

use crate::tui::{
    app::{App, DrillView, Focus, ResourceTab},
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

    let show_account = app.is_widget_enabled("account");
    let show_events = app.is_widget_enabled("events");

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

    if let Some(cp) = app.palette.as_ref() {
        crate::tui::command_palette::render(frame, size, &palette, cp);
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
                Span::styled(format!("{:<10}", action.display_label()), style),
            ];
            if action.is_destructive() {
                spans.push(Span::styled("\u{26A0}", Style::default().fg(palette.error)));
            }
            Line::from(spans)
        })
        .collect();

    let kind_fallback = t!("ui.action_menu_kind_fallback");
    let kind = ResourceTab::names()
        .get(menu.tab.index())
        .copied()
        .map_or_else(|| kind_fallback.as_ref(), |k| k);
    let title = t!(
        "ui.action_menu_title",
        kind => kind,
        name => menu.resource_name,
        id => menu.resource_id
    )
    .to_string();
    let width = u16::try_from(title.len() + 4)
        .unwrap_or(40)
        .clamp(28, area.width.saturating_sub(4));
    let height = u16::try_from(menu.actions.len()).unwrap_or(5) + 2;
    let popup = Rect {
        x: area.width.saturating_sub(width) / 2,
        y: area.height.saturating_sub(height) / 2,
        width,
        height
    };

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
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
            t!(
                "ui.confirm_prompt",
                verb => pending.kind.display_label(),
                name => pending.resource_name,
                id => pending.resource_id
            )
            .to_string(),
            Style::default().fg(palette.fg).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
    ];
    if pending.kind.is_destructive() {
        lines.push(Line::from(Span::styled(
            t!("ui.confirm_irreversible").to_string(),
            Style::default().fg(palette.error)
        )));
        lines.push(Line::from(""));
    }
    lines.push(Line::from(vec![
        Span::styled(
            " [y] ",
            Style::default().fg(accent).add_modifier(Modifier::BOLD)
        ),
        Span::styled(
            format!("{}    ", t!("ui.confirm_yes")),
            Style::default().fg(palette.dim)
        ),
        Span::styled(
            "[n] ",
            Style::default().fg(palette.fg).add_modifier(Modifier::BOLD)
        ),
        Span::styled(
            t!("ui.confirm_no").to_string(),
            Style::default().fg(palette.dim)
        ),
    ]));

    let width = 54u16.min(area.width.saturating_sub(4));
    let height = u16::try_from(lines.len()).unwrap_or(4) + 2;
    let popup = Rect {
        x: area.width.saturating_sub(width) / 2,
        y: area.height.saturating_sub(height) / 2,
        width,
        height
    };

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(accent))
                .title(Line::from(Span::styled(
                    t!("ui.confirm_title").to_string(),
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

    let show_stats = app.is_widget_enabled("stats");
    let show_token = app.is_widget_enabled("token_info");

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

/// Renders the status bar with mode indicator and available keys.
///
/// # Arguments
///
/// * `frame` - The render frame.
/// * `area` - The status bar area rectangle.
/// * `app` - The application state.
/// * `palette` - The theme color palette.
fn render_status_bar(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let resources_fallback = t!("ui.status_resources_fallback");
    let tab = ResourceTab::names()
        .get(app.active_tab.index())
        .copied()
        .map_or_else(|| resources_fallback.as_ref(), |t| t);

    let key = |k: &'static str| {
        Span::styled(
            k,
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD)
        )
    };
    let lbl = |t: String| Span::styled(t, Style::default().fg(palette.dim));

    let mut spans = vec![
        Span::styled(
            format!(" {tab} "),
            Style::default()
                .fg(palette.bg)
                .bg(palette.tab_active)
                .add_modifier(Modifier::BOLD)
        ),
        Span::raw("  "),
        key("h/l"),
        lbl(format!(" {}   ", t!("ui.status_tabs"))),
        key("j/k"),
        lbl(format!(" {}   ", t!("ui.status_move"))),
        key("⏎"),
        lbl(format!(" {}   ", t!("ui.status_open"))),
        key("/"),
        lbl(format!(" {}   ", t!("ui.status_filter"))),
        key("^K"),
        lbl(format!(" {}   ", t!("ui.status_cmds"))),
        key("?"),
        lbl(format!(" {}   ", t!("ui.status_help"))),
        key("Q"),
        lbl(format!(" {}", t!("ui.status_quit"))),
    ];

    let message = match (&app.error_message, &app.status_message) {
        (Some(err), _) => Some((err.clone(), palette.error)),
        (_, Some(msg)) => Some((msg.clone(), palette.success)),
        _ => None
    };
    if let Some((text, color)) = message {
        spans.push(Span::raw("   "));
        spans.push(Span::styled("● ", Style::default().fg(color)));
        spans.push(Span::styled(text, Style::default().fg(color)));
    }

    let paragraph = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(palette.border))
    );
    frame.render_widget(paragraph, area);
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
