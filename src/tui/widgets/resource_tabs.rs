// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Resource tabs widget — renders a modern horizontal resource category bar.
//!
//! Each tab is drawn as a padded pill with an icon, a display name and a live
//! item count badge. The active tab becomes a filled pill (inverted accent
//! background). When the tabs overflow the available width the bar scrolls to
//! keep the active tab visible, showing `‹`/`›` chevrons for hidden tabs and a
//! `i/N` position indicator pinned to the right edge.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph}
};

use crate::tui::app::{App, ResourceTab};

/// Per-tab icon glyphs, indexed by [`ResourceTab::index`].
///
/// Monochrome, geometric BMP glyphs are used on purpose: they render at a
/// stable single-cell width across terminals, so the layout math stays exact
/// (unlike full-color emoji which are frequently double-width).
const TAB_ICONS: [char; 20] = [
    '\u{25A3}', // Servers ▣
    '\u{25A4}', // Databases ▤
    '\u{25A6}', // S3 ▦
    '\u{2638}', // Kubernetes ☸
    '\u{25A7}', // Projects ▧
    '\u{25D1}', // Balancers ◑
    '\u{25A9}', // Registry ▩
    '\u{25C9}', // Domains ◉
    '\u{25A8}', // Firewall ▨
    '\u{25C8}', // FloatingIps ◈
    '\u{25A5}', // Images ▥
    '\u{25E2}', // NetworkDrives ◢
    '\u{2756}', // Vpc ❖
    '\u{25A0}', // DedicatedServers ■
    '\u{25E7}', // Mail ◧
    '\u{2726}', // Apps ✦
    '\u{25C6}', // AiAgents ◆
    '\u{2605}', // KnowledgeBases ★
    '\u{25C7}', // SshKeys ◇
    '\u{25CE}'  // Finances ◎
];

/// Renders the resource category tabs as a modern, scrolling pill bar.
///
/// # Overview
///
/// Displays resource tabs (Servers, Databases, S3, …) on a single line. Each
/// tab shows ` <icon> <Name> <count> `; the active tab is rendered as a filled
/// pill. Off-screen tabs are signalled with `‹`/`›` chevrons and a `i/N`
/// position indicator is pinned to the right edge.
pub struct ResourceTabsWidget {
    enabled: bool
}

impl ResourceTabsWidget {
    /// Creates a new resource tabs widget with enabled state.
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

    /// Collects the live item count for every tab, ordered by tab index.
    fn tab_counts(app: &App) -> [usize; 20] {
        [
            app.servers.len(),
            app.databases.len(),
            app.s3_storages.len(),
            app.k8s_clusters.len(),
            app.projects.len(),
            app.balancers.len(),
            app.registries.len(),
            app.domains.len(),
            app.firewalls.len(),
            app.floating_ips.len(),
            app.images.len(),
            app.network_drives.len(),
            app.vpcs.len(),
            app.dedicated_servers.len(),
            app.mails.len(),
            app.apps.len(),
            app.ai_agents.len(),
            app.knowledge_bases.len(),
            app.ssh_keys.len(),
            app.finances.len()
        ]
    }

    /// Computes the visible tab window `[start, end)` for a given budget.
    ///
    /// The active tab is always included; the window then grows outward
    /// (right first, then left) while the accumulated tab widths fit within
    /// `budget`. This keeps the active tab on screen and surrounds it with as
    /// many neighbours as fit.
    ///
    /// # Arguments
    ///
    /// * `widths` - Rendered cell width of each tab pill.
    /// * `active` - Index of the active tab.
    /// * `budget` - Cells available for tab pills (chevrons/counter excluded).
    fn visible_window(widths: &[u16], active: usize, budget: u16) -> (usize, usize) {
        if widths.is_empty() {
            return (0, 0);
        }

        let active = active.min(widths.len() - 1);
        let mut start = active;
        let mut end = active + 1;
        let mut used = widths[active];

        if used > budget {
            return (start, end);
        }

        loop {
            let grow_right = end < widths.len() && used.saturating_add(widths[end]) <= budget;
            if grow_right {
                used += widths[end];
                end += 1;
            }

            let grow_left = start > 0 && used.saturating_add(widths[start - 1]) <= budget;
            if grow_left {
                start -= 1;
                used += widths[start];
            }

            if !grow_right && !grow_left {
                break;
            }
        }

        (start, end)
    }

    /// Builds the styled tab bar line with pills, chevrons and a position
    /// badge.
    ///
    /// # Arguments
    ///
    /// * `app` - The application state.
    /// * `width` - Available width for the tab bar.
    ///
    /// # Returns
    ///
    /// A `Line` containing the styled tab bar.
    fn build_tab_bar(app: &App, width: u16) -> Line<'static> {
        let names = ResourceTab::names();
        let len = names.len();
        let active_idx = app.active_tab.index();
        let palette = app.theme.palette();
        let counts = Self::tab_counts(app);

        let labels: Vec<(String, String, u16)> = (0..len)
            .map(|i| {
                let head = format!(" {} {} ", TAB_ICONS[i], names[i]);
                let tail = format!("{} ", counts[i]);
                let w =
                    u16::try_from(head.chars().count() + tail.chars().count()).unwrap_or(u16::MAX);
                (head, tail, w)
            })
            .collect();
        let widths: Vec<u16> = labels.iter().map(|l| l.2).collect();

        let position = format!(" {}/{} ", active_idx + 1, len);
        let position_w = u16::try_from(position.chars().count()).unwrap_or(0);

        let reserve = position_w.saturating_add(4);
        let budget = width.saturating_sub(reserve);
        let (start, end) = Self::visible_window(&widths, active_idx, budget);

        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut used: u16 = 0;

        if start > 0 {
            spans.push(Span::styled("\u{2039} ", Style::default().fg(palette.dim)));
            used = used.saturating_add(2);
        }

        for (i, (head, tail, w)) in labels.into_iter().enumerate().take(end).skip(start) {
            if i == active_idx {
                let pill = Style::default()
                    .fg(palette.bg)
                    .bg(palette.tab_active)
                    .add_modifier(Modifier::BOLD);
                spans.push(Span::styled(head, pill));
                spans.push(Span::styled(tail, pill));
            } else {
                spans.push(Span::styled(
                    head,
                    Style::default().fg(palette.tab_inactive)
                ));
                spans.push(Span::styled(tail, Style::default().fg(palette.dim)));
            }
            used = used.saturating_add(w);
        }

        if end < len {
            spans.push(Span::styled(" \u{203A}", Style::default().fg(palette.dim)));
            used = used.saturating_add(2);
        }

        let consumed = used.saturating_add(position_w);
        if width > consumed {
            spans.push(Span::raw(" ".repeat(usize::from(width - consumed))));
        }
        spans.push(Span::styled(
            position,
            Style::default()
                .fg(palette.header)
                .add_modifier(Modifier::BOLD)
        ));

        Line::from(spans)
    }
}

impl crate::tui::widgets::Widget for ResourceTabsWidget {
    fn id(&self) -> &'static str {
        "resource_tabs"
    }

    fn name(&self) -> &'static str {
        "Resource Tabs"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        let palette = app.theme.palette();
        let border_color = if app.focus == crate::tui::app::Focus::ResourceTabs {
            palette.accent
        } else {
            palette.border
        };
        let tab_bar = Self::build_tab_bar(app, area.width);

        let paragraph = Paragraph::new(tab_bar).block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(border_color))
        );

        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_tabs_fit_when_budget_is_large() {
        let widths = [10u16, 10, 10, 10];
        let (start, end) = ResourceTabsWidget::visible_window(&widths, 0, 1000);
        assert_eq!((start, end), (0, 4));
    }

    #[test]
    fn active_tab_is_always_inside_window() {
        let widths = [12u16; 20];
        let budget = 30;
        for active in 0..widths.len() {
            let (start, end) = ResourceTabsWidget::visible_window(&widths, active, budget);
            assert!(
                start <= active && active < end,
                "active {active} not in window"
            );
        }
    }

    #[test]
    fn window_never_exceeds_budget() {
        let widths = [12u16; 20];
        let budget = 40;
        let (start, end) = ResourceTabsWidget::visible_window(&widths, 7, budget);
        let total: u16 = widths[start..end].iter().sum();
        assert!(total <= budget);
    }

    #[test]
    fn oversized_active_tab_still_renders_alone() {
        let widths = [5u16, 100, 5];
        let (start, end) = ResourceTabsWidget::visible_window(&widths, 1, 10);
        assert_eq!((start, end), (1, 2));
    }

    #[test]
    fn empty_widths_yield_empty_window() {
        let widths: [u16; 0] = [];
        assert_eq!(ResourceTabsWidget::visible_window(&widths, 0, 50), (0, 0));
    }

    #[test]
    fn icon_table_covers_every_tab() {
        assert_eq!(TAB_ICONS.len(), ResourceTab::names().len());
    }
}
