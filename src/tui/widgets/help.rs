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
    /// * `widget_names` - Names of all registered widgets with their enabled
    ///   state.
    ///
    /// # Returns
    ///
    /// A vector of styled `Line` values representing the full help content.
    fn build_lines(
        palette: crate::tui::themes::Palette,
        widget_names: &[(String, bool)]
    ) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        lines.push(Line::from(Span::styled(
            t!("help.title").to_string(),
            Style::default()
                .fg(palette.title)
                .add_modifier(Modifier::BOLD)
        )));

        lines.push(Line::from(Span::styled(
            " ".repeat(60),
            Style::default().fg(palette.border)
        )));

        let shortcuts = [
            ("h/l", t!("help.shortcut_tab")),
            ("j/k", t!("help.shortcut_move")),
            ("g/G", t!("help.shortcut_jump")),
            ("Enter", t!("help.shortcut_enter")),
            ("n", t!("help.shortcut_new")),
            ("/", t!("help.shortcut_filter")),
            ("Ctrl+K", t!("help.shortcut_palette")),
            ("p", t!("help.shortcut_profile")),
            ("r", t!("help.shortcut_refresh")),
            ("Esc", t!("help.shortcut_esc")),
            ("?", t!("help.shortcut_toggle_help")),
            ("Q", t!("help.shortcut_quit"))
        ];

        for (key, desc) in shortcuts {
            lines.push(Self::make_shortcut_line(key, desc.as_ref(), palette));
        }

        lines.push(Line::from(Span::styled(
            " ".repeat(60),
            Style::default().fg(palette.border)
        )));

        lines.push(Line::from(Span::styled(
            t!("help.widgets_header").to_string(),
            Style::default()
                .fg(palette.title)
                .add_modifier(Modifier::BOLD)
        )));

        for (name, enabled) in widget_names {
            let status = if *enabled {
                Line::from(vec![
                    Span::styled("[x] ", Style::default().fg(palette.success)),
                    Span::styled(name.clone(), Style::default().fg(palette.fg)),
                ])
            } else {
                Line::from(vec![
                    Span::styled("[ ] ", Style::default().fg(palette.dim)),
                    Span::styled(name.clone(), Style::default().fg(palette.dim)),
                ])
            };
            lines.push(status);
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
        let key_len = u16::try_from(key.len()).unwrap_or(0);
        let padding = 8u16.saturating_sub(key_len);
        let pad_str = " ".repeat(padding as usize);

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

    /// Collects widget names and enabled states from the registry.
    ///
    /// # Arguments
    ///
    /// * `widgets` - The widget registry to inspect.
    ///
    /// # Returns
    ///
    /// A vector of `(name, enabled)` tuples for all registered widgets.
    fn collect_widget_status(
        widgets: &crate::tui::widgets::WidgetRegistry
    ) -> Vec<(String, bool)> {
        let mut status = Vec::new();

        let all_widgets: Vec<&(dyn crate::tui::widgets::Widget + Send)> = widgets
            .widgets
            .iter()
            .map(std::convert::AsRef::as_ref)
            .collect();

        for w in all_widgets {
            status.push((w.name().to_string(), w.enabled()));
        }

        status
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
        let widget_names = Self::collect_widget_status(&app.widgets);
        let content = Self::build_lines(palette, &widget_names);

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(palette.accent))
            )
            .alignment(Alignment::Left)
            .style(Style::default().bg(palette.bg));

        let popup_area = Rect {
            x:      area.width / 4,
            y:      area.height / 4,
            width:  area.width / 2,
            height: area.height / 2
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
        let lines = HelpWidget::build_lines(palette, &[]);

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
    }

    #[test]
    fn build_lines_contains_widget_section() {
        let palette = Theme::GruvboxDark.palette();
        let widgets = vec![
            ("Account".to_string(), true),
            ("Resources".to_string(), false),
        ];
        let lines = HelpWidget::build_lines(palette, &widgets);

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

        assert!(text.contains("[x]"));
        assert!(text.contains("[ ]"));
        assert!(text.contains("Account"));
        assert!(text.contains("Resources"));
    }

    #[test]
    fn build_lines_empty_widgets_no_panic() {
        let palette = Theme::GruvboxDark.palette();
        let lines = HelpWidget::build_lines(palette, &[]);
        assert!(!lines.is_empty());
    }

    #[test]
    fn make_shortcut_line_produces_correct_format() {
        let palette = Theme::CatppuccinMocha.palette();
        let line = HelpWidget::make_shortcut_line("q", "Quit", palette);

        assert_eq!(line.spans.len(), 3);
        assert!(line.spans[0].content.contains('q'));
    }

    #[test]
    fn make_shortcut_line_handles_long_keys() {
        let palette = Theme::GruvboxLight.palette();
        let line = HelpWidget::make_shortcut_line("Tab", "Cycle tabs", palette);

        assert_eq!(line.spans.len(), 3);
    }

    #[test]
    fn collect_widget_status_returns_widget_info() {
        let registry = crate::tui::widgets::WidgetRegistry::new();
        let status = HelpWidget::collect_widget_status(&registry);

        assert!(!status.is_empty());
        assert!(status.iter().any(|(name, _)| name == "Account"));
    }

    #[test]
    fn collect_widget_status_preserves_enabled_state() {
        let registry = crate::tui::widgets::WidgetRegistry::new();
        let status = HelpWidget::collect_widget_status(&registry);

        for (_, enabled) in &status {
            assert!(*enabled);
        }
    }

    #[test]
    fn widget_trait_object_is_send() {
        let widget: Box<dyn Widget + Send> = Box::new(HelpWidget::new());
        assert_eq!(widget.id(), "help");
    }

    #[test]
    fn build_lines_with_disabled_widgets() {
        let palette = Theme::GruvboxDark.palette();
        let widgets = vec![("Account".to_string(), true), ("Stats".to_string(), false)];
        let lines = HelpWidget::build_lines(palette, &widgets);

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

        let account_idx = text.find("Account").expect("account entry present");
        let stats_idx = text.find("Stats").expect("stats entry present");
        assert!(account_idx < stats_idx);
        assert!(text.contains("[x]"));
        assert!(text.contains("[ ]"));
    }

    #[test]
    fn build_lines_palette_applied() {
        let palette = Theme::CatppuccinLatte.palette();
        let lines = HelpWidget::build_lines(palette, &[]);

        let title_line = &lines[0];
        let span = title_line.spans.first().expect("title span");
        assert!(matches!(span.style.fg, Some(Color::Rgb(183, 108, 7))));
    }
}
