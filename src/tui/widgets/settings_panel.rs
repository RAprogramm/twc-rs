// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! The settings panel: every dashboard setting as a `label → value` row,
//! adjustable in place.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph}
};
use rust_i18n::t;

use crate::tui::app::{App, Pane, SETTING_ROWS};

/// Renders the settings panel into `area`.
pub fn render(frame: &mut Frame, area: Rect, app: &App, border: ratatui::style::Color) {
    let palette = app.theme.palette();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .title(Line::from(Span::styled(
            format!(" {} ", t!("sidebar.settings")),
            Style::default()
                .fg(palette.title)
                .add_modifier(Modifier::BOLD)
        )));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width < 10 || inner.height < 2 {
        return;
    }

    let focused = app.pane == Pane::Content;
    let mut lines: Vec<Line> = Vec::with_capacity(SETTING_ROWS.len() * 2 + 2);
    lines.push(Line::from(Span::styled(
        t!("settings.hint").into_owned(),
        Style::default().fg(palette.dim)
    )));
    lines.push(Line::from(""));

    for (i, row) in SETTING_ROWS.iter().enumerate() {
        let selected = focused && i == app.settings_selected;
        let label = row.label().into_owned();
        let value = app.setting_value(*row);
        let width = usize::from(inner.width);
        let pad = width
            .saturating_sub(4 + label.chars().count() + value.chars().count())
            .max(1);

        let bar = if selected { "\u{258E}" } else { " " };
        let value_style = if selected {
            Style::default()
                .fg(palette.bg)
                .bg(palette.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(palette.accent)
        };
        lines.push(Line::from(vec![
            Span::styled(
                bar.to_string(),
                Style::default().fg(if selected { palette.accent } else { palette.bg })
            ),
            Span::styled(
                label,
                if selected {
                    Style::default().fg(palette.fg).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(palette.fg)
                }
            ),
            Span::raw(" ".repeat(pad)),
            Span::styled(format!(" {value} "), value_style),
        ]));
        lines.push(Line::from(""));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

#[cfg(test)]
mod tests {
    use ratatui::{Terminal, backend::TestBackend};

    use super::*;

    #[test]
    fn renders_all_rows_without_panic() {
        let mut app = App::new(5);
        app.pane = Pane::Content;
        app.settings_selected = 2;
        let palette = app.theme.palette();
        for (w, h) in [(1, 1), (30, 6), (80, 24), (200, 50)] {
            let backend = TestBackend::new(w, h);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|f| render(f, Rect::new(0, 0, w, h), &app, palette.accent))
                .unwrap();
        }
    }
}
