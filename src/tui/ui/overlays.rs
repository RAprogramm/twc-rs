// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Popup overlays: the create form, the action menu and the confirm dialog.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph}
};
use rust_i18n::t;

use crate::tui::{app::App, themes::Palette};

/// Renders the in-dashboard resource-creation form: a titled box with one
/// labelled input per field, the focused field highlighted, and a hint line.
pub(super) fn render_create_form(
    frame: &mut Frame,
    area: Rect,
    form: &crate::tui::app::CreateForm,
    palette: &Palette
) {
    let width = 56u16.min(area.width.saturating_sub(4));
    let height = u16::try_from(form.fields.len()).unwrap_or(2) + 4;
    let popup = Rect {
        x: (area.width.saturating_sub(width)) / 2,
        y: (area.height.saturating_sub(height)) / 2,
        width,
        height
    };
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(palette.accent))
        .title(Line::from(Span::styled(
            format!(" {} ", form.title),
            Style::default()
                .fg(palette.title)
                .add_modifier(Modifier::BOLD)
        )));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let mut constraints: Vec<Constraint> =
        form.fields.iter().map(|_| Constraint::Length(1)).collect();
    constraints.push(Constraint::Length(1));
    let rows = Layout::vertical(constraints).split(inner);

    for (i, field) in form.fields.iter().enumerate() {
        let focused = i == form.active;
        let marker = if focused { "▸ " } else { "  " };
        let label = format!("{marker}{}: ", field.label);
        let value_style = if focused {
            Style::default()
                .fg(palette.fg)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(palette.dim)
        };
        let cursor = if focused { "█" } else { "" };
        let line = Line::from(vec![
            Span::styled(label, Style::default().fg(palette.header)),
            Span::styled(format!("{}{cursor}", field.value), value_style),
        ]);
        if let Some(row) = rows.get(i) {
            frame.render_widget(Paragraph::new(line), *row);
        }
    }

    if let Some(hint_row) = rows.last() {
        let hint = Paragraph::new(Line::from(Span::styled(
            "Tab next · Enter create · Esc cancel",
            Style::default().fg(palette.dim)
        )));
        frame.render_widget(hint, *hint_row);
    }
}

/// Renders the context action menu for the selected server.
///
/// Lists the available actions with the highlighted one marked; destructive
/// actions are shown in the error color with a warning glyph.
pub(super) fn render_action_menu(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
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

    let kind = menu.tab.display_name();
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
pub(super) fn render_confirm(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
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
