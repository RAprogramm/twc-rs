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
///
/// While the sidebar owns focus this is a presentation: an overview of the
/// platform, the full list of creatable products with one-line descriptions,
/// and the official-docs note about API tokens. After Enter the pane switches
/// to the creation cards.
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

    if app.pane == Pane::Sidebar {
        render_presentation(frame, inner, &palette);
        return;
    }

    let cards = cards();
    let cols = card_grid::columns(inner.width, card_grid::longest_title(&cards));
    card_grid::render_grid_in(frame, inner, &cards, app.create_selected, cols, &palette);
}

/// Renders the hover presentation, redistributing its content to the space
/// it actually has: with room to spare every product gets its one-line
/// description; when the panel shrinks (e.g. the events log is on) the
/// product list reflows into name-only columns so the API-token note and the
/// hint always stay visible.
fn render_presentation(frame: &mut Frame, inner: Rect, palette: &crate::tui::themes::Palette) {
    use ratatui::widgets::{Paragraph, Wrap};

    let targets = App::create_targets();
    let intro = t!("create.presentation_intro").into_owned();
    let token_text = t!("create.token_text").into_owned();
    let wrap_rows = |text: &str| {
        (text.chars().count() / usize::from(inner.width.saturating_sub(4).max(1)) + 1).min(4)
    };
    let intro_rows = wrap_rows(&intro);
    let token_rows = wrap_rows(&token_text) + 1;
    let chrome = intro_rows + 1 + token_rows + 2;
    let product_rows = usize::from(inner.height).saturating_sub(chrome).max(1);
    let full = targets.len() < product_rows;

    let mut lines: Vec<Line> = vec![Line::from(Span::styled(
        intro,
        Style::default().fg(palette.fg)
    ))];
    lines.push(Line::from(""));

    if full {
        for tab in &targets {
            let (desc, _) = service_header::texts(*tab);
            lines.push(Line::from(vec![
                Span::styled(
                    format!(" {} ", sidebar::tab_icon(*tab)),
                    Style::default().fg(palette.accent)
                ),
                Span::styled(
                    format!("{:<18}", tab.display_name()),
                    Style::default().fg(palette.fg).add_modifier(Modifier::BOLD)
                ),
                Span::styled(desc.into_owned(), Style::default().fg(palette.dim)),
            ]));
        }
    } else {
        let rows = product_rows.max(1);
        let cols = targets.len().div_ceil(rows);
        let col_w = targets
            .iter()
            .map(|t| t.display_name().chars().count())
            .max()
            .unwrap_or(12)
            + 2;
        for r in 0..rows.min(targets.len()) {
            let mut spans: Vec<Span> = Vec::new();
            for c in 0..cols {
                let Some(tab) = targets.get(c * rows + r) else {
                    continue;
                };
                spans.push(Span::styled(
                    format!(" {} ", sidebar::tab_icon(*tab)),
                    Style::default().fg(palette.accent)
                ));
                spans.push(Span::styled(
                    format!("{:<col_w$}", tab.display_name()),
                    Style::default().fg(palette.fg)
                ));
            }
            lines.push(Line::from(spans));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        t!("create.token_title").into_owned(),
        Style::default()
            .fg(palette.header)
            .add_modifier(Modifier::BOLD)
    )));
    lines.push(Line::from(Span::styled(
        token_text,
        Style::default().fg(palette.dim)
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        t!("create.presentation_hint").into_owned(),
        Style::default().fg(palette.accent)
    )));

    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap {
            trim: false
        }),
        inner
    );
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
    fn presentation_reflows_to_keep_token_note_visible() {
        let app = App::new(5);
        let palette = app.theme.palette();
        for h in [16u16, 40] {
            let w = 120u16;
            let backend = TestBackend::new(w, h);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|f| render(f, Rect::new(0, 0, w, h), &app, palette.accent))
                .unwrap();
            let buf = terminal.backend().buffer().clone();
            let mut text = String::new();
            for y in 0..h {
                for x in 0..w {
                    text.push_str(buf[(x, y)].symbol());
                }
                text.push(' ');
            }
            assert!(
                text.contains("About API tokens"),
                "token note must stay visible at height {h}"
            );
            assert!(text.contains("Kubernetes"), "products visible at {h}");
        }
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
