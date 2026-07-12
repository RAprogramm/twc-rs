// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! The settings panel: setting cards on the shared card grid — icon, name and
//! current state — toggled or picked with Enter.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders}
};
use rust_i18n::t;

use crate::tui::{
    app::{App, Pane, SETTING_ROWS},
    widgets::card_grid::{self, GridCard}
};

/// Builds one card per setting: toggles get a colored on/off badge, value
/// settings show their current value.
#[must_use]
pub fn cards(app: &App) -> Vec<GridCard> {
    let palette = app.theme.palette();
    SETTING_ROWS
        .iter()
        .map(|row| {
            let card = GridCard::new(row.label().into_owned()).icon(row.icon());
            match app.setting_enabled(*row) {
                Some(true) => card.status(palette.success, t!("settings.on").into_owned()),
                Some(false) => card.status(palette.dim, t!("settings.off").into_owned()),
                None => card.meta(app.setting_value(*row))
            }
        })
        .collect()
}

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
    if inner.width < 10 || inner.height < 3 {
        return;
    }

    let cards = cards(app);
    let cols = card_grid::columns(inner.width, card_grid::longest_title(&cards));
    let selected = if app.pane == Pane::Content {
        app.settings_selected
    } else {
        usize::MAX
    };
    card_grid::render_grid_in(frame, inner, &cards, selected, cols, &palette);
}

#[cfg(test)]
mod tests {
    use ratatui::{Terminal, backend::TestBackend};

    use super::*;

    #[test]
    fn renders_across_sizes_without_panic() {
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

    #[test]
    fn toggle_cards_have_badges_value_cards_have_meta() {
        let app = App::new(5);
        let cards = cards(&app);
        assert_eq!(cards.len(), SETTING_ROWS.len());
        assert!(cards[0].status.is_none());
        assert!(!cards[0].meta.is_empty());
        assert!(cards[2].status.is_some());
    }
}
