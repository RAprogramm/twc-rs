// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! System metrics widget — renders CPU, RAM, and network usage as live
//! sparklines.

use std::collections::VecDeque;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Sparkline}
};

use crate::tui::{app::App, themes::Palette};

/// Number of baseline samples rendered when a metric history is empty.
const BASELINE_SAMPLES: usize = 8;

/// A single metric row prepared for rendering.
struct MetricRow {
    /// Short label such as `CPU` or `Net↓`.
    label:       &'static str,
    /// Current value formatted for display, or `—` when no data is available.
    value:       String,
    /// Foreground color for the displayed value.
    value_color: Color,
    /// Sparkline samples, oldest first.
    data:        Vec<u64>,
    /// Upper bound used to scale the sparkline bars.
    max:         u64,
    /// Foreground color for the sparkline bars.
    spark_color: Color
}

/// Renders system resource usage as live sparklines.
///
/// # Overview
///
/// Displays four metrics — CPU usage (%), RAM usage (%), network in (MB/s),
/// and network out (MB/s) — each as a label, current value, and a
/// [`Sparkline`] of the rolling history window. Percentage metrics are colored
/// by level (green below 60%, warning below 85%, error otherwise). When a
/// history is empty, a flat baseline sparkline is shown with a dim `—` value
/// rather than any "no data" text.
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
    #[must_use]
    pub const fn new(enabled: bool) -> Self {
        Self {
            enabled
        }
    }

    /// Maps a percentage value to a level color.
    ///
    /// # Arguments
    ///
    /// * `value` - Percentage in the range 0.0 to 100.0.
    /// * `palette` - The active theme palette.
    ///
    /// # Returns
    ///
    /// `palette.success` below 60%, `palette.warning` below 85%, otherwise
    /// `palette.error`.
    fn level_color(value: f64, palette: Palette) -> Color {
        if value < 60.0 {
            palette.success
        } else if value < 85.0 {
            palette.warning
        } else {
            palette.error
        }
    }

    /// Converts a percentage to a clamped, rounded sparkline sample.
    ///
    /// # Arguments
    ///
    /// * `value` - Percentage that may fall outside the 0.0 to 100.0 range.
    ///
    /// # Returns
    ///
    /// The value clamped to `0..=100` and rounded to the nearest integer.
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn pct_to_u64(value: f64) -> u64 {
        value.clamp(0.0, 100.0).round() as u64
    }

    /// Converts a byte count to mebibytes for display.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Raw byte count for the sampling interval.
    ///
    /// # Returns
    ///
    /// The value expressed in mebibytes (MiB).
    #[expect(clippy::cast_precision_loss)]
    fn bytes_to_mb(bytes: u64) -> f64 {
        bytes as f64 / 1_048_576.0
    }

    /// Builds a percentage metric row from a history window.
    fn percent_row(label: &'static str, history: &VecDeque<f64>, palette: Palette) -> MetricRow {
        history.back().map_or_else(
            || MetricRow {
                label,
                value: "\u{2014}".to_owned(),
                value_color: palette.dim,
                data: vec![0; BASELINE_SAMPLES],
                max: 100,
                spark_color: palette.dim
            },
            |&last| {
                let color = Self::level_color(last, palette);
                MetricRow {
                    label,
                    value: format!("{}%", Self::pct_to_u64(last)),
                    value_color: color,
                    data: history.iter().map(|&v| Self::pct_to_u64(v)).collect(),
                    max: 100,
                    spark_color: color
                }
            }
        )
    }

    /// Builds a network throughput metric row from a history window.
    fn net_row(
        label: &'static str,
        history: &VecDeque<u64>,
        color: Color,
        palette: Palette
    ) -> MetricRow {
        history.back().map_or_else(
            || MetricRow {
                label,
                value: "\u{2014}".to_owned(),
                value_color: palette.dim,
                data: vec![0; BASELINE_SAMPLES],
                max: 1,
                spark_color: palette.dim
            },
            |&last| {
                let data: Vec<u64> = history.iter().copied().collect();
                let max = data.iter().copied().max().unwrap_or(1).max(1);
                MetricRow {
                    label,
                    value: format!("{:.1}M", Self::bytes_to_mb(last)),
                    value_color: color,
                    data,
                    max,
                    spark_color: color
                }
            }
        )
    }

    /// Assembles all four metric rows for the current application state.
    fn metric_rows(app: &App, palette: Palette) -> [MetricRow; 4] {
        [
            Self::percent_row("CPU", &app.cpu_history, palette),
            Self::percent_row("RAM", &app.ram_history, palette),
            Self::net_row("Net\u{2193}", &app.net_in_history, palette.accent, palette),
            Self::net_row(
                "Net\u{2191}",
                &app.net_out_history,
                palette.warning,
                palette
            )
        ]
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

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(palette.border))
            .title(Line::from(Span::styled(
                " Stats ",
                Style::default()
                    .fg(palette.title)
                    .add_modifier(Modifier::BOLD)
            )));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let rows = Self::metric_rows(app, palette);
        let row_areas = Layout::vertical([Constraint::Fill(1); 4]).split(inner);

        for (row, &row_area) in rows.iter().zip(row_areas.iter()) {
            let cols =
                Layout::horizontal([Constraint::Length(11), Constraint::Min(0)]).split(row_area);

            let label = Paragraph::new(Line::from(vec![
                Span::styled(
                    format!("{} ", row.label),
                    Style::default().fg(palette.header)
                ),
                Span::styled(
                    row.value.clone(),
                    Style::default()
                        .fg(row.value_color)
                        .add_modifier(Modifier::BOLD)
                ),
            ]));
            frame.render_widget(label, cols[0]);

            let sparkline = Sparkline::default()
                .data(row.data.clone())
                .max(row.max)
                .style(Style::default().fg(row.spark_color));
            frame.render_widget(sparkline, cols[1]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::themes::Theme;

    fn palette() -> Palette {
        Theme::default().palette()
    }

    #[test]
    fn level_color_low_is_success() {
        let p = palette();
        assert_eq!(StatsWidget::level_color(0.0, p), p.success);
        assert_eq!(StatsWidget::level_color(59.9, p), p.success);
    }

    #[test]
    fn level_color_mid_is_warning() {
        let p = palette();
        assert_eq!(StatsWidget::level_color(60.0, p), p.warning);
        assert_eq!(StatsWidget::level_color(84.9, p), p.warning);
    }

    #[test]
    fn level_color_high_is_error() {
        let p = palette();
        assert_eq!(StatsWidget::level_color(85.0, p), p.error);
        assert_eq!(StatsWidget::level_color(100.0, p), p.error);
    }

    #[test]
    fn pct_to_u64_clamps_and_rounds() {
        assert_eq!(StatsWidget::pct_to_u64(-5.0), 0);
        assert_eq!(StatsWidget::pct_to_u64(0.4), 0);
        assert_eq!(StatsWidget::pct_to_u64(0.5), 1);
        assert_eq!(StatsWidget::pct_to_u64(73.6), 74);
        assert_eq!(StatsWidget::pct_to_u64(150.0), 100);
    }

    #[test]
    fn bytes_to_mb_converts_mebibytes() {
        assert!((StatsWidget::bytes_to_mb(1_048_576) - 1.0).abs() < f64::EPSILON);
        assert!((StatsWidget::bytes_to_mb(0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn percent_row_empty_shows_dim_dash_baseline() {
        let p = palette();
        let history = VecDeque::new();
        let row = StatsWidget::percent_row("CPU", &history, p);
        assert_eq!(row.value, "\u{2014}");
        assert_eq!(row.value_color, p.dim);
        assert_eq!(row.data.len(), BASELINE_SAMPLES);
        assert!(row.data.iter().all(|&v| v == 0));
        assert_eq!(row.max, 100);
    }

    #[test]
    fn percent_row_with_data_uses_level_color() {
        let p = palette();
        let history: VecDeque<f64> = [10.0, 50.0, 90.0].into_iter().collect();
        let row = StatsWidget::percent_row("CPU", &history, p);
        assert_eq!(row.value, "90%");
        assert_eq!(row.value_color, p.error);
        assert_eq!(row.data, vec![10, 50, 90]);
    }

    #[test]
    fn net_row_empty_shows_dim_dash_baseline() {
        let p = palette();
        let history = VecDeque::new();
        let row = StatsWidget::net_row("Net\u{2193}", &history, p.accent, p);
        assert_eq!(row.value, "\u{2014}");
        assert_eq!(row.value_color, p.dim);
        assert_eq!(row.data.len(), BASELINE_SAMPLES);
        assert_eq!(row.max, 1);
    }

    #[test]
    fn net_row_with_data_scales_to_max() {
        let p = palette();
        let history: VecDeque<u64> = [0, 1_048_576, 2_097_152].into_iter().collect();
        let row = StatsWidget::net_row("Net\u{2193}", &history, p.accent, p);
        assert_eq!(row.value, "2.0M");
        assert_eq!(row.value_color, p.accent);
        assert_eq!(row.max, 2_097_152);
    }

    #[test]
    fn metric_rows_returns_four_labels() {
        let p = palette();
        let app = App::new(5);
        let rows = StatsWidget::metric_rows(&app, p);
        assert_eq!(rows[0].label, "CPU");
        assert_eq!(rows[1].label, "RAM");
        assert_eq!(rows[2].label, "Net\u{2193}");
        assert_eq!(rows[3].label, "Net\u{2191}");
    }
}
