// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! The create hub: one card per creatable resource type — icon, type name and
//! the create action label — opened with Enter.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders}
};
use rust_i18n::t;

use crate::tui::{
    app::{App, Pane},
    widgets::{
        card_grid::{self, GridCard},
        service_header, sidebar
    }
};

/// Builds one card per creatable resource type.
#[must_use]
pub fn cards() -> Vec<GridCard> {
    App::create_targets()
        .into_iter()
        .map(|tab| {
            let (_, create) = service_header::texts(tab);
            GridCard::new(tab.display_name().into_owned())
                .icon(sidebar::tab_icon(tab))
                .meta(create.into_owned())
        })
        .collect()
}

/// Renders the create hub into `area`.
pub fn render(frame: &mut Frame, area: Rect, app: &App, border: ratatui::style::Color) {
    let palette = app.theme.palette();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .title(Line::from(Span::styled(
            format!(" {} ", t!("sidebar.create")),
            Style::default()
                .fg(palette.title)
                .add_modifier(Modifier::BOLD)
        )));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width < 10 || inner.height < 3 {
        return;
    }

    let cards = cards();
    let cols = card_grid::columns(inner.width, card_grid::longest_title(&cards));
    let selected = if app.pane == Pane::Content {
        app.create_selected
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
    fn one_card_per_target_with_create_label() {
        let cards = cards();
        assert_eq!(cards.len(), App::create_targets().len());
        assert!(cards.iter().all(|c| !c.meta.is_empty()));
    }

    #[test]
    fn renders_across_sizes_without_panic() {
        let mut app = App::new(5);
        app.pane = Pane::Content;
        let palette = app.theme.palette();
        for (w, h) in [(1, 1), (30, 8), (80, 24), (200, 50)] {
            let backend = TestBackend::new(w, h);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|f| render(f, Rect::new(0, 0, w, h), &app, palette.accent))
                .unwrap();
        }
    }
}
