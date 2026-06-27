// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! CPU / RAM / Disk gauge widgets.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Gauge}
};

/// Renders a single gauge bar.
fn render_gauge(frame: &mut Frame, area: Rect, label: &str, ratio: f64, detail: &str) {
    let color = if ratio > 0.8 {
        Color::Red
    } else if ratio > 0.6 {
        Color::Yellow
    } else {
        Color::Green
    };

    let ratio_clamped = ratio.clamp(0.0, 1.0);
    let title = format!("{label}  {detail}");
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
        )
        .gauge_style(Style::default().fg(color).bg(Color::Black))
        .ratio(ratio_clamped)
        .label(title);

    frame.render_widget(gauge, area);
}

/// Renders CPU, RAM, and Disk gauges in the given area.
///
/// The area is split into 3 equal horizontal sections.
pub fn render(
    frame: &mut Frame,
    area: Rect,
    cpu_ratio: f64,
    ram_ratio: f64,
    ram_used_gb: f64,
    ram_total_gb: f64,
    disk_ratio: f64,
    disk_used_gb: f64,
    disk_total_gb: f64
) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Percentage(33),
            ratatui::layout::Constraint::Percentage(33),
            ratatui::layout::Constraint::Percentage(34)
        ])
        .split(area);

    let cpu_pct = format!("{:.0}%", cpu_ratio * 100.0);
    render_gauge(frame, chunks[0], "CPU", cpu_ratio, &cpu_pct);

    let ram_detail = format!("{:.1}/{:.1} GB", ram_used_gb, ram_total_gb);
    render_gauge(frame, chunks[1], "RAM", ram_ratio, &ram_detail);

    let disk_detail = format!("{:.0}/{:.0} GB", disk_used_gb, disk_total_gb);
    render_gauge(frame, chunks[2], "Disk", disk_ratio, &disk_detail);
}
