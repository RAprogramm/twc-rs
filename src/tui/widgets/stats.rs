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
use rust_i18n::t;

use crate::tui::{app::App, themes::Palette};

/// Number of baseline samples rendered when a metric history is empty.
const BASELINE_SAMPLES: usize = 8;

/// A single metric row prepared for rendering.
struct MetricRow {
    /// Short label such as `CPU` or `Net↓`.
    label:       String,
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
    const fn pct_to_u64(value: f64) -> u64 {
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
    fn bytes_to_mb(bytes: f64) -> f64 {
        bytes / 1_048_576.0
    }

    /// Converts a byte sample to a non-negative integer for the sparkline,
    /// which renders `u64` data points.
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    const fn bytes_to_spark(value: f64) -> u64 {
        value.max(0.0).round() as u64
    }

    /// Builds a percentage metric row from a history window.
    fn percent_row(
        label: impl Into<String>,
        history: &VecDeque<f64>,
        palette: Palette
    ) -> MetricRow {
        let label = label.into();
        history.back().map_or_else(
            || MetricRow {
                label:       label.clone(),
                value:       "\u{2014}".to_owned(),
                value_color: palette.dim,
                data:        vec![0; BASELINE_SAMPLES],
                max:         100,
                spark_color: palette.dim
            },
            |&last| {
                let color = Self::level_color(last, palette);
                MetricRow {
                    label:       label.clone(),
                    value:       format!("{}%", Self::pct_to_u64(last)),
                    value_color: color,
                    data:        history.iter().map(|&v| Self::pct_to_u64(v)).collect(),
                    max:         100,
                    spark_color: color
                }
            }
        )
    }

    /// Builds a network throughput metric row from a history window.
    fn net_row(
        label: impl Into<String>,
        history: &VecDeque<f64>,
        color: Color,
        palette: Palette
    ) -> MetricRow {
        let label = label.into();
        history.back().map_or_else(
            || MetricRow {
                label:       label.clone(),
                value:       "\u{2014}".to_owned(),
                value_color: palette.dim,
                data:        vec![0; BASELINE_SAMPLES],
                max:         1,
                spark_color: palette.dim
            },
            |&last| {
                let data: Vec<u64> = history.iter().map(|&v| Self::bytes_to_spark(v)).collect();
                let max = data.iter().copied().max().unwrap_or(1).max(1);
                MetricRow {
                    label: label.clone(),
                    value: format!("{:.1}M", Self::bytes_to_mb(last)),
                    value_color: color,
                    data,
                    max,
                    spark_color: color
                }
            }
        )
    }

    /// Assembles a metric row for every series that has data, so resources that
    /// expose only some metrics (servers report no live RAM) show just those.
    fn metric_rows(app: &App, palette: Palette) -> Vec<MetricRow> {
        let mut rows = Vec::with_capacity(4);
        if !app.cpu_history.is_empty() {
            rows.push(Self::percent_row(
                t!("stats.cpu").to_string(),
                &app.cpu_history,
                palette
            ));
        }
        if !app.ram_history.is_empty() {
            rows.push(Self::percent_row(
                t!("stats.ram").to_string(),
                &app.ram_history,
                palette
            ));
        }
        if !app.net_in_history.is_empty() {
            rows.push(Self::net_row(
                t!("stats.net_in").to_string(),
                &app.net_in_history,
                palette.accent,
                palette
            ));
        }
        if !app.net_out_history.is_empty() {
            rows.push(Self::net_row(
                t!("stats.net_out").to_string(),
                &app.net_out_history,
                palette.warning,
                palette
            ));
        }
        rows
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

        let title = app.stats_subject.as_ref().map_or_else(
            || format!(" {} ", t!("stats.title")),
            |name| format!(" {} — {name} ", t!("stats.title"))
        );
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(palette.border))
            .title(Line::from(Span::styled(
                title,
                Style::default()
                    .fg(palette.title)
                    .add_modifier(Modifier::BOLD)
            )));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if app.stats_subject.is_none() {
            let hint = Paragraph::new(t!("stats.subject_hint").to_string())
                .style(Style::default().fg(palette.dim))
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(hint, inner);
            return;
        }

        let rows = Self::metric_rows(app, palette);
        if rows.is_empty() {
            return;
        }
        let constraints: Vec<Constraint> = rows.iter().map(|_| Constraint::Length(1)).collect();
        let row_areas = Layout::vertical(constraints).spacing(1).split(inner);

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
        assert!((StatsWidget::bytes_to_mb(1_048_576.0) - 1.0).abs() < f64::EPSILON);
        assert!((StatsWidget::bytes_to_mb(0.0) - 0.0).abs() < f64::EPSILON);
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
        let history: VecDeque<f64> = [0.0, 1_048_576.0, 2_097_152.0].into_iter().collect();
        let row = StatsWidget::net_row("Net\u{2193}", &history, p.accent, p);
        assert_eq!(row.value, "2.0M");
        assert_eq!(row.value_color, p.accent);
        assert_eq!(row.max, 2_097_152);
    }

    #[test]
    fn metric_rows_returns_a_row_per_populated_series() {
        let p = palette();
        let mut app = App::new(5);
        app.push_cpu(10.0);
        app.push_ram(20.0);
        app.push_net_in(100.0);
        app.push_net_out(200.0);
        let rows = StatsWidget::metric_rows(&app, p);
        assert_eq!(rows[0].label, "CPU");
        assert_eq!(rows[1].label, "RAM");
        assert_eq!(rows[2].label, "Net\u{2193}");
        assert_eq!(rows[3].label, "Net\u{2191}");
    }

    #[test]
    fn metric_rows_empty_without_data() {
        let p = palette();
        let app = App::new(5);
        assert!(StatsWidget::metric_rows(&app, p).is_empty());
    }
}
