// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! System metrics widget — renders CPU, RAM, and network usage as ASCII bar
//! charts.

use std::fmt::Write;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph}
};

use crate::tui::app::App;

/// Renders system resource usage as ASCII bar charts.
///
/// # Overview
///
/// Displays four metrics — CPU usage (%), RAM usage (%), network in (MB/s),
/// and network out (MB/s) — as horizontal bars using Unicode block characters.
/// Shows "No data" when the corresponding history is empty.
///
/// # Bar characters
///
/// | Fill | Char | Code |
/// |------|------|------|
/// | Full | █ | U+2588 |
/// | 3/4  | ▓ | U+2593 |
/// | 1/2  | ▒ | U+2592 |
/// | 1/4  | ░ | U+2591 |
/// | Empty| ░ | U+2591 |
///
/// # Examples
///
/// ```ignore
/// use twc_rs::tui::widgets::stats::StatsWidget;
///
/// let widget = StatsWidget::new(true);
/// assert_eq!(widget.id(), "stats");
/// ```
pub struct StatsWidget {
    enabled: bool
}

impl StatsWidget {
    /// Creates a new stats widget with the given enabled state.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether the widget is initially visible.
    pub const fn new(enabled: bool) -> Self {
        Self {
            enabled
        }
    }

    /// Returns the Unicode block character for a given fill fraction.
    ///
    /// # Arguments
    ///
    /// * `frac` - Fraction of the character to fill (0.0 to 1.0).
    /// * `full` - Character for fully filled blocks.
    /// * `empty` - Character for empty blocks.
    ///
    /// # Returns
    ///
    /// The appropriate block character for the given fill fraction.
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn bar_char(frac: f64, full: char, empty: char) -> char {
        let idx = (frac * 4.0) as usize;
        match idx {
            4 => full,
            3 => '\u{2593}',
            2 => '\u{2592}',
            1 => '\u{2591}',
            _ => empty
        }
    }

    /// Builds the bar string for a given percentage.
    ///
    /// # Arguments
    ///
    /// * `percentage` - Fill percentage (0.0 to 100.0).
    /// * `width` - Total bar width in characters.
    /// * `full` - Character for fully filled blocks.
    /// * `empty` - Character for empty blocks.
    ///
    /// # Returns
    ///
    /// A string of exactly `width` characters representing the bar.
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss
    )]
    fn build_bar(percentage: f64, width: usize, full: char, empty: char) -> String {
        let fill_pct = percentage / 100.0;
        let filled = fill_pct * (width as f64);
        let full_blocks = filled as usize;
        let remainder = filled - (full_blocks as f64);
        let half_block = Self::bar_char(remainder, full, empty);
        let empty_blocks = width - full_blocks - 1;

        let mut bar = String::with_capacity(width);
        for _ in 0..full_blocks {
            let _ = write!(bar, "{full}");
        }
        let _ = write!(bar, "{half_block}");
        for _ in 0..empty_blocks {
            let _ = write!(bar, "{empty}");
        }
        bar
    }

    /// Builds the styled lines for the stats panel.
    ///
    /// # Arguments
    ///
    /// * `app` - The application state with metric histories.
    /// * `palette` - The theme color palette.
    ///
    /// # Returns
    ///
    /// A vector of `Line` structs representing the rendered content.
    fn build_lines(app: &App, palette: crate::tui::themes::Palette) -> Vec<Line<'static>> {
        let bar_width: usize = 16;

        let cpu_line = match app.cpu_history.back() {
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            Some(&val) => {
                let pct = val.round() as u64;
                let bar = Self::build_bar(val, bar_width, '\u{2588}', '\u{2591}');
                Line::from(Span::styled(
                    format!("CPU Usage  {bar} {pct}%"),
                    Style::default().fg(palette.accent)
                ))
            }
            None => Line::from(Span::styled(
                "CPU Usage       No data",
                Style::default().fg(palette.dim)
            ))
        };

        let ram_line = match app.ram_history.back() {
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            Some(&val) => {
                let pct = val.round() as u64;
                let bar = Self::build_bar(val, bar_width, '\u{2588}', '\u{2591}');
                Line::from(Span::styled(
                    format!("RAM Usage  {bar} {pct}%"),
                    Style::default().fg(palette.accent)
                ))
            }
            None => Line::from(Span::styled(
                "RAM Usage       No data",
                Style::default().fg(palette.dim)
            ))
        };

        let net_in_line = match app.net_in_history.back() {
            #[expect(clippy::cast_precision_loss)]
            Some(&val) => {
                let max_val = app.net_in_history.iter().copied().max().unwrap_or(1);
                let pct = (val as f64 / max_val as f64) * 100.0;
                let mb = (val as f64 / 1_048_576.0).round();
                let bar = Self::build_bar(pct, bar_width, '\u{2588}', '\u{2591}');
                Line::from(Span::styled(
                    format!("Net In    {bar} {mb} MB/s"),
                    Style::default().fg(palette.accent)
                ))
            }
            None => Line::from(Span::styled(
                "Net In          No data",
                Style::default().fg(palette.dim)
            ))
        };

        let net_out_line = match app.net_out_history.back() {
            #[expect(clippy::cast_precision_loss)]
            Some(&val) => {
                let max_val = app.net_out_history.iter().copied().max().unwrap_or(1);
                let pct = (val as f64 / max_val as f64) * 100.0;
                let mb = (val as f64 / 1_048_576.0).round();
                let bar = Self::build_bar(pct, bar_width, '\u{2588}', '\u{2591}');
                Line::from(Span::styled(
                    format!("Net Out   {bar} {mb} MB/s"),
                    Style::default().fg(palette.accent)
                ))
            }
            None => Line::from(Span::styled(
                "Net Out         No data",
                Style::default().fg(palette.dim)
            ))
        };

        vec![cpu_line, ram_line, net_in_line, net_out_line]
    }
}

impl crate::tui::widgets::Widget for StatsWidget {
    fn id(&self) -> &'static str {
        "stats"
    }

    fn name(&self) -> &'static str {
        "Stats"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        let palette = app.theme.palette();
        let lines = Self::build_lines(app, palette);

        let paragraph = Paragraph::new(lines).block(
            Block::default().borders(Borders::ALL).title(Line::from(Span::styled(
                " Stats ",
                Style::default()
                    .fg(palette.title)
                    .add_modifier(Modifier::BOLD)
            )))
        );

        frame.render_widget(paragraph, area);
    }
}
