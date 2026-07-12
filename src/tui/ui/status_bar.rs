// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! The bottom status bar: a context chip naming where the user is, the key
//! hints that actually apply there, and the freshest message right-aligned.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph}
};
use rust_i18n::t;

use crate::tui::{
    app::{App, NavKind, Pane},
    themes::Palette
};

/// The name of the place the user currently is, for the left chip.
fn context_name(app: &App) -> String {
    if app.detail_open {
        return app
            .selected_resource()
            .map_or_else(|| app.active_tab.display_name().into_owned(), |(_, n)| n);
    }
    match app.nav_current() {
        Some(NavKind::Create) => t!("sidebar.create").into_owned(),
        Some(NavKind::Settings) => t!("sidebar.settings").into_owned(),
        Some(NavKind::Project(i)) => app
            .projects
            .get(i)
            .map_or_else(String::new, |p| p.name.clone()),
        Some(NavKind::Service(tab)) => tab.display_name().into_owned(),
        None => app.active_tab.display_name().into_owned()
    }
}

/// The `(key, action)` hints that are actually available right now.
fn hints(app: &App) -> Vec<(&'static str, String)> {
    if app.filter_editing {
        return vec![
            ("\u{23ce}", t!("ui.status_apply").into_owned()),
            ("Esc", t!("ui.status_clear").into_owned()),
        ];
    }
    if app.picker_open() {
        return vec![
            ("\u{2191}\u{2193}", t!("ui.status_nav").into_owned()),
            ("\u{23ce}", t!("ui.status_pick").into_owned()),
            ("Esc", t!("ui.status_back").into_owned()),
        ];
    }
    if app.detail_open {
        return vec![
            ("\u{2191}\u{2193}", t!("ui.status_scroll").into_owned()),
            ("Esc", t!("ui.status_back").into_owned()),
            ("Q", t!("ui.status_quit").into_owned()),
        ];
    }
    if app.pane == Pane::Sidebar {
        return vec![
            ("\u{2191}\u{2193}", t!("ui.status_nav").into_owned()),
            ("\u{23ce}", t!("ui.status_open").into_owned()),
            ("^K", t!("ui.status_cmds").into_owned()),
            ("?", t!("ui.status_help").into_owned()),
            ("Q", t!("ui.status_quit").into_owned()),
        ];
    }
    let mut hints = vec![(
        "\u{2190}\u{2192}\u{2191}\u{2193}",
        t!("ui.status_nav").into_owned()
    )];
    match app.nav_current() {
        Some(NavKind::Create) => {
            hints.push(("\u{23ce}", t!("ui.status_create_hint").into_owned()));
        }
        Some(NavKind::Settings) => {
            hints.push(("\u{23ce}", t!("ui.status_change").into_owned()));
        }
        Some(NavKind::Project(_)) => {
            hints.push(("\u{23ce}", t!("ui.status_details").into_owned()));
        }
        Some(NavKind::Service(_)) | None => {
            hints.push(("\u{23ce}", t!("ui.status_actions").into_owned()));
            hints.push(("/", t!("ui.status_filter").into_owned()));
        }
    }
    hints.push(("Esc", t!("ui.status_back").into_owned()));
    hints.push(("Q", t!("ui.status_quit").into_owned()));
    hints
}

/// Renders the status bar: context chip, live hints, right-aligned message
/// (an error wins over a status note; with neither, the data age is shown).
pub(super) fn render_status_bar(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let key = |k: &str| {
        Span::styled(
            k.to_string(),
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD)
        )
    };
    let lbl = |t: String| Span::styled(t, Style::default().fg(palette.dim));

    let mut spans = vec![
        Span::styled(
            format!(" {} ", context_name(app)),
            Style::default()
                .fg(palette.bg)
                .bg(palette.tab_active)
                .add_modifier(Modifier::BOLD)
        ),
        Span::raw("  "),
    ];
    for (k, label) in hints(app) {
        spans.push(key(k));
        spans.push(lbl(format!(" {label}   ")));
    }

    let (message, color) = match (&app.error_message, &app.status_message) {
        (Some(err), _) => (format!("\u{25CF} {err}"), palette.error),
        (_, Some(msg)) => (format!("\u{25CF} {msg}"), palette.success),
        _ => (
            t!("ui.status_updated", s => app.last_refresh.elapsed().as_secs()).into_owned(),
            palette.dim
        )
    };

    let used: usize = spans.iter().map(|s| s.content.chars().count()).sum();
    let inner_w = usize::from(area.width.saturating_sub(2));
    let msg_len = message.chars().count();
    if used + msg_len < inner_w {
        spans.push(Span::raw(" ".repeat(inner_w - used - msg_len)));
        spans.push(Span::styled(message, Style::default().fg(color)));
    } else if inner_w > used + 2 {
        let cut: String = message.chars().take(inner_w - used - 1).collect();
        spans.push(Span::raw(" "));
        spans.push(Span::styled(cut, Style::default().fg(color)));
    }

    let paragraph = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(palette.border))
    );
    frame.render_widget(paragraph, area);
}
