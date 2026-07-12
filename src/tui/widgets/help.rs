// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Help widget — displays keyboard shortcuts and widget toggle status.

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph}
};
use rust_i18n::t;

use crate::tui::app::App;

/// Renders the help overlay with keyboard shortcuts and widget toggles.
///
/// # Overview
///
/// Displays a centered popup containing all keyboard shortcuts,
/// a separator, and the current enabled/disabled status of every
/// registered dashboard widget.
///
/// The widget is always considered "enabled" because it is an
/// overlay — it does not participate in the grid layout.
///
/// # Example
///
/// ```ignore
/// let mut registry = WidgetRegistry::new();
/// registry.register(Box::new(help::HelpWidget::new()));
/// ```
pub struct HelpWidget;

impl HelpWidget {
    /// Creates a new help widget.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Builds the help text lines for the overlay.
    ///
    /// # Arguments
    ///
    /// * `palette` - The theme color palette.
    ///
    /// # Returns
    ///
    /// A vector of styled `Line` values representing the full help content.
    fn build_lines(palette: crate::tui::themes::Palette) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        lines.push(Line::from(Span::styled(
            t!("help.title").to_string(),
            Style::default()
                .fg(palette.title)
                .add_modifier(Modifier::BOLD)
        )));
        lines.push(Line::from(""));

        let shortcuts = [
            ("\u{2191}\u{2193} k j", t!("help.shortcut_move")),
            ("\u{2190}\u{2192} h l", t!("help.shortcut_cols")),
            ("\u{21e5}/\u{21e4}", t!("help.shortcut_tab")),
            ("Enter", t!("help.shortcut_enter")),
            ("Esc", t!("help.shortcut_esc")),
            ("y/c", t!("help.shortcut_copy")),
            ("g/G", t!("help.shortcut_jump")),
            ("n", t!("help.shortcut_new")),
            ("/", t!("help.shortcut_filter")),
            ("Ctrl+K", t!("help.shortcut_palette")),
            ("p", t!("help.shortcut_profile")),
            ("r", t!("help.shortcut_refresh")),
            ("?", t!("help.shortcut_toggle_help")),
            ("Q", t!("help.shortcut_quit"))
        ];

        for (key, desc) in shortcuts {
            lines.push(Self::make_shortcut_line(key, desc.as_ref(), palette));
        }

        lines
    }

    /// Formats a single shortcut line with key and description.
    ///
    /// # Arguments
    ///
    /// * `key` - The keyboard key or key combination.
    /// * `desc` - Human-readable description of the action.
    /// * `palette` - The theme color palette.
    ///
    /// # Returns
    ///
    /// A styled `Line` with the key in bold accent color and description in
    /// dim.
    fn make_shortcut_line(
        key: &str,
        desc: &str,
        palette: crate::tui::themes::Palette
    ) -> Line<'static> {
        let key_len = key.chars().count();
        let pad_str = " ".repeat(8usize.saturating_sub(key_len).max(2));

        Line::from(vec![
            Span::styled(
                key.to_string(),
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD)
            ),
            Span::raw(pad_str),
            Span::styled(desc.to_string(), Style::default().fg(palette.dim)),
        ])
    }

    /// Renders the help overlay centered in the given area.
    ///
    /// # Arguments
    ///
    /// * `frame` - The render frame.
    /// * `area` - The total terminal area.
    /// * `app` - The application state containing widgets and theme.
    fn render_overlay(frame: &mut Frame, area: Rect, app: &App) {
        let palette = app.theme.palette();
        let content = Self::build_lines(palette);

        let width = content
            .iter()
            .map(Line::width)
            .max()
            .unwrap_or(20)
            .saturating_add(4);
        let width = u16::try_from(width)
            .unwrap_or(u16::MAX)
            .min(area.width.saturating_sub(2));
        let height = u16::try_from(content.len() + 2)
            .unwrap_or(u16::MAX)
            .min(area.height.saturating_sub(2));

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(palette.accent))
            )
            .alignment(Alignment::Left)
            .style(Style::default().bg(palette.bg));

        let popup_area = Rect {
            x: area.width.saturating_sub(width) / 2,
            y: area.height.saturating_sub(height) / 2,
            width,
            height
        };

        frame.render_widget(Clear, popup_area);
        frame.render_widget(paragraph, popup_area);
    }
}

impl crate::tui::widgets::Widget for HelpWidget {
    fn id(&self) -> &'static str {
        "help"
    }

    fn name(&self) -> &'static str {
        "Help"
    }

    fn enabled(&self) -> bool {
        true
    }

    fn toggle(&mut self) {}

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        Self::render_overlay(frame, area, app);
    }
}

impl Default for HelpWidget {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use ratatui::style::Color;

    use super::*;
    use crate::tui::{themes::Theme, widgets::Widget};

    #[test]
    fn widget_id_is_help() {
        let widget = HelpWidget::new();
        assert_eq!(widget.id(), "help");
    }

    #[test]
    fn widget_name_is_help() {
        let widget = HelpWidget::new();
        assert_eq!(widget.name(), "Help");
    }

    #[test]
    fn widget_is_always_enabled() {
        let widget = HelpWidget::new();
        assert!(widget.enabled());
    }

    #[test]
    fn widget_toggle_does_nothing() {
        let mut widget = HelpWidget::new();
        widget.toggle();
        assert!(widget.enabled());
    }

    #[test]
    fn default_is_new() {
        let widget = HelpWidget;
        assert_eq!(widget.id(), "help");
    }

    #[test]
    fn build_lines_contains_shortcuts() {
        let palette = Theme::GruvboxDark.palette();
        let lines = HelpWidget::build_lines(palette);

        let text: String = lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
            .join(" ");

        assert!(text.contains("Quit"));
        assert!(text.contains("Refresh now"));
        assert!(text.contains("Toggle this help"));
        assert!(text.contains("Copy"));
        assert!(text.contains("details"));
    }

    #[test]
    fn make_shortcut_line_produces_correct_format() {
        let palette = Theme::CatppuccinMocha.palette();
        let line = HelpWidget::make_shortcut_line("q", "Quit", palette);

        assert_eq!(line.spans.len(), 3);
        assert!(line.spans[0].content.contains('q'));
    }

    #[test]
    fn overlay_fits_all_shortcuts_on_a_small_terminal() {
        use ratatui::{Terminal, backend::TestBackend, layout::Rect};

        let mut app = crate::tui::app::App::new(5);
        app.show_help = true;
        let (w, h) = (80u16, 24u16);
        let backend = TestBackend::new(w, h);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| HelpWidget::render_overlay(f, Rect::new(0, 0, w, h), &app))
            .unwrap();
        let buf = terminal.backend().buffer().clone();
        let mut text = String::new();
        for y in 0..h {
            for x in 0..w {
                text.push_str(buf[(x, y)].symbol());
            }
        }
        assert!(text.contains("Quit"), "last shortcut must be visible");
        assert!(
            text.contains("press a button"),
            "descriptions must not be clipped"
        );
    }

    #[test]
    fn build_lines_palette_applied() {
        let palette = Theme::CatppuccinLatte.palette();
        let lines = HelpWidget::build_lines(palette);

        let title_line = &lines[0];
        let span = title_line.spans.first().expect("title span");
        assert!(matches!(span.style.fg, Some(Color::Rgb(183, 108, 7))));
    }
}
