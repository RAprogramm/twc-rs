// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Resource list widget — shows selected resources in a scrollable list.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState}
};
use rust_i18n::t;

use crate::tui::{
    app::{App, ResourceTab},
    themes::Palette
};

/// Maps a server status (the API enum's debug name) to a display icon,
/// color, and human-readable label.
///
/// The API reports states like `On`, `Off`, `Installing`, `Rebooting` —
/// not `Running`/`Stopped` — so the dashboard translates them here.
#[must_use]
pub fn server_status_view(status: &str, palette: &Palette) -> (&'static str, Color, String) {
    match status {
        "On" => (
            "\u{25B6}",
            palette.success,
            t!("resource_list.status_running").to_string()
        ),
        "Off" => (
            "\u{25CB}",
            palette.error,
            t!("resource_list.status_stopped").to_string()
        ),
        "Removed" | "Blocked" => ("\u{25CB}", palette.error, status.to_lowercase()),
        "TurningOn" => (
            "\u{25D0}",
            palette.warning,
            t!("resource_list.status_starting").to_string()
        ),
        "TurningOff" | "HardTurningOff" => (
            "\u{25D0}",
            palette.warning,
            t!("resource_list.status_stopping").to_string()
        ),
        "Rebooting" | "HardRebooting" => (
            "\u{25D0}",
            palette.warning,
            t!("resource_list.status_rebooting").to_string()
        ),
        "Installing" | "SoftwareInstall" | "Reinstalling" => (
            "\u{25D0}",
            palette.warning,
            t!("resource_list.status_installing").to_string()
        ),
        "Removing" => (
            "\u{25CC}",
            palette.error,
            t!("resource_list.status_removing").to_string()
        ),
        other => ("\u{25D0}", palette.warning, other.to_lowercase())
    }
}

/// Maps an arbitrary resource status string to a display color and a
/// human-readable lowercase label.
///
/// Works on the Debug names of the SDK's per-resource status enums
/// (`Started`, `NoPaid`, `Failure`, ...), grouping them into healthy,
/// failed, and transitional states; unknown states render as transitional.
#[must_use]
pub fn status_view(status: &str, palette: &Palette) -> (Color, String) {
    let label = status.to_lowercase();
    let color = match label.as_str() {
        "on" | "started" | "active" | "running" | "ok" | "ready" | "created" | "deployed"
        | "delivered" | "attached" | "available" | "success" | "finished" | "normal" => {
            palette.success
        }
        "off" | "stopped" | "error" | "failure" | "failed" | "blocked" | "removed" | "damaged"
        | "disabled" | "nopaid" | "startuperror" | "cancelled" | "expired" => palette.error,
        _ => palette.warning
    };
    (color, label)
}

/// Computes the integral used-disk percentage of a container registry,
/// treating a zero-sized disk as fully free.
fn registry_used_percent(registry: &crate::tui::app::RegistrySummary) -> i64 {
    if registry.disk_size <= 0 {
        0
    } else {
        registry.disk_used * 100 / registry.disk_size
    }
}

/// Renders the resource list panel with a given border color.
///
/// # Arguments
///
/// * `frame` - The render frame.
/// * `area` - The area to render in.
/// * `app` - The application state.
/// * `border_color` - Color for the panel border.
// JUSTIFY: Large match expression covering all resource types.
#[allow(clippy::too_many_lines)]
pub fn render(frame: &mut Frame, area: Rect, app: &App, border_color: Color) {
    let palette = app.theme.palette();

    let items: Vec<ListItem> = match app.active_tab {
        ResourceTab::Servers => app
            .servers
            .iter()
            .map(|s| {
                let (icon, status_color, label) = server_status_view(&s.status, &palette);
                let line = Line::from(vec![
                    Span::raw(format!("{icon} ")),
                    Span::styled(&s.name, Style::default().fg(palette.fg)),
                    Span::raw("  "),
                    Span::styled(format!("[{label}]"), Style::default().fg(status_color)),
                ]);
                ListItem::new(line)
            })
            .collect(),
        ResourceTab::Databases => app
            .databases
            .iter()
            .map(|d| {
                let line = Line::from(vec![
                    Span::raw("\u{25CF} "),
                    Span::styled(&d.name, Style::default().fg(palette.fg)),
                    Span::raw("  "),
                    Span::styled(
                        format!("[{}]", d.engine),
                        Style::default().fg(palette.accent)
                    ),
                ]);
                ListItem::new(line)
            })
            .collect(),
        ResourceTab::S3 => app
            .s3_storages
            .iter()
            .map(|s| {
                let line = Line::from(vec![
                    Span::raw("\u{1F4E6} "),
                    Span::styled(&s.name, Style::default().fg(palette.fg)),
                    Span::raw("  "),
                    Span::styled(&s.region, Style::default().fg(palette.warning)),
                ]);
                ListItem::new(line)
            })
            .collect(),
        ResourceTab::Kubernetes => app
            .k8s_clusters
            .iter()
            .map(|c| {
                let line = Line::from(vec![
                    Span::raw("\u{2638} "),
                    Span::styled(&c.name, Style::default().fg(palette.fg)),
                    Span::raw("  "),
                    Span::styled(
                        format!("[v{}]", c.version),
                        Style::default().fg(palette.accent)
                    ),
                ]);
                ListItem::new(line)
            })
            .collect(),
        ResourceTab::Projects => app
            .projects
            .iter()
            .map(|p| {
                let line = Line::from(vec![
                    Span::raw("\u{1F4C1} "),
                    Span::styled(&p.name, Style::default().fg(palette.fg)),
                    Span::raw("  "),
                    Span::styled(
                        format!(
                            "[{}]",
                            t!("resource_list.count_servers", n => p.server_count)
                        ),
                        Style::default().fg(palette.dim)
                    ),
                ]);
                ListItem::new(line)
            })
            .collect(),
        ResourceTab::Balancers => app
            .balancers
            .iter()
            .map(|b| {
                let line = Line::from(vec![
                    Span::raw("\u{2696} "),
                    Span::styled(&b.name, Style::default().fg(palette.fg)),
                    Span::raw("  "),
                    Span::styled(&b.ip, Style::default().fg(palette.warning)),
                ]);
                ListItem::new(line)
            })
            .collect(),
        ResourceTab::Registry => app
            .registries
            .iter()
            .map(|r| {
                let line = Line::from(vec![
                    Span::raw("\u{1F433} "),
                    Span::styled(&r.name, Style::default().fg(palette.fg)),
                    Span::raw("  "),
                    Span::styled(
                        format!(
                            "[{}]",
                            t!("resource_list.disk_used", pct => registry_used_percent(r))
                        ),
                        Style::default().fg(palette.accent)
                    ),
                ]);
                ListItem::new(line)
            })
            .collect(),
        ResourceTab::Domains => app
            .domains
            .iter()
            .map(|d| {
                let (status_color, label) = status_view(&d.status, &palette);
                let line = Line::from(vec![
                    Span::raw("\u{1F310} "),
                    Span::styled(&d.name, Style::default().fg(palette.fg)),
                    Span::raw("  "),
                    Span::styled(format!("[{label}]"), Style::default().fg(status_color)),
                ]);
                ListItem::new(line)
            })
            .collect(),
        ResourceTab::Firewall => app
            .firewalls
            .iter()
            .map(|f| {
                let line = Line::from(vec![
                    Span::raw("\u{1F6E1} "),
                    Span::styled(&f.name, Style::default().fg(palette.fg)),
                    Span::raw("  "),
                    Span::styled(
                        format!("[{}]", f.policy),
                        Style::default().fg(palette.accent)
                    ),
                ]);
                ListItem::new(line)
            })
            .collect(),
        ResourceTab::FloatingIps => app
            .floating_ips
            .iter()
            .map(|f| {
                let line = Line::from(vec![
                    Span::raw("\u{1F500} "),
                    Span::styled(&f.ip, Style::default().fg(palette.warning)),
                    Span::raw("  "),
                    Span::styled(&f.server_name, Style::default().fg(palette.fg)),
                ]);
                ListItem::new(line)
            })
            .collect(),
        ResourceTab::Images => app
            .images
            .iter()
            .map(|i| {
                let line = Line::from(vec![
                    Span::raw("\u{1F5BC} "),
                    Span::styled(&i.name, Style::default().fg(palette.fg)),
                    Span::raw("  "),
                    Span::styled(
                        format!("[{} MB]", i.size_mb),
                        Style::default().fg(palette.dim)
                    ),
                ]);
                ListItem::new(line)
            })
            .collect(),
        ResourceTab::NetworkDrives => app
            .network_drives
            .iter()
            .map(|n| {
                let line = Line::from(vec![
                    Span::raw("\u{1F4BE} "),
                    Span::styled(&n.name, Style::default().fg(palette.fg)),
                    Span::raw("  "),
                    Span::styled(
                        format!("[{} GB]", n.size_gb),
                        Style::default().fg(palette.accent)
                    ),
                ]);
                ListItem::new(line)
            })
            .collect(),
        ResourceTab::Vpc => app
            .vpcs
            .iter()
            .map(|v| {
                let line = Line::from(vec![
                    Span::raw("\u{1F517} "),
                    Span::styled(&v.name, Style::default().fg(palette.fg)),
                    Span::raw("  "),
                    Span::styled(v.subnet.clone(), Style::default().fg(palette.warning)),
                ]);
                ListItem::new(line)
            })
            .collect(),
        ResourceTab::DedicatedServers => app
            .dedicated_servers
            .iter()
            .map(|d| {
                let (status_color, label) = status_view(&d.status, &palette);
                let line = Line::from(vec![
                    Span::raw("\u{1F5A5} "),
                    Span::styled(&d.name, Style::default().fg(palette.fg)),
                    Span::raw("  "),
                    Span::styled(format!("[{label}]"), Style::default().fg(status_color)),
                ]);
                ListItem::new(line)
            })
            .collect(),
        ResourceTab::Mail => app
            .mails
            .iter()
            .map(|m| {
                let line = Line::from(vec![
                    Span::raw("\u{1F4E7} "),
                    Span::styled(&m.name, Style::default().fg(palette.fg)),
                    Span::raw("  "),
                    Span::styled(m.owner.clone(), Style::default().fg(palette.dim)),
                ]);
                ListItem::new(line)
            })
            .collect(),
        ResourceTab::Apps => app
            .apps
            .iter()
            .map(|a| {
                let line = Line::from(vec![
                    Span::raw("\u{1F680} "),
                    Span::styled(&a.name, Style::default().fg(palette.fg)),
                    Span::raw("  "),
                    Span::styled(a.location.clone(), Style::default().fg(palette.warning)),
                ]);
                ListItem::new(line)
            })
            .collect(),
        ResourceTab::AiAgents => app
            .ai_agents
            .iter()
            .map(|a| {
                let line = Line::from(vec![
                    Span::raw("\u{1F916} "),
                    Span::styled(&a.name, Style::default().fg(palette.fg)),
                    Span::raw("  "),
                    Span::styled(
                        format!("[{}/{}]", a.tokens_used, a.tokens_total),
                        Style::default().fg(palette.warning)
                    ),
                ]);
                ListItem::new(line)
            })
            .collect(),
        ResourceTab::KnowledgeBases => app
            .knowledge_bases
            .iter()
            .map(|k| {
                let line = Line::from(vec![
                    Span::raw("\u{1F4DA} "),
                    Span::styled(&k.name, Style::default().fg(palette.fg)),
                    Span::raw("  "),
                    Span::styled(
                        format!(
                            "[{}]",
                            t!("resource_list.count_docs", n => k.document_count)
                        ),
                        Style::default().fg(palette.dim)
                    ),
                ]);
                ListItem::new(line)
            })
            .collect(),
        ResourceTab::SshKeys => app
            .ssh_keys
            .iter()
            .map(|k| {
                let line = Line::from(vec![
                    Span::raw("\u{1F511} "),
                    Span::styled(k, Style::default().fg(palette.fg)),
                ]);
                ListItem::new(line)
            })
            .collect(),
        ResourceTab::Finances => app
            .finances
            .iter()
            .map(|f| {
                let line = Line::from(vec![
                    Span::raw("\u{1F4B0} "),
                    Span::styled(f, Style::default().fg(palette.fg)),
                ]);
                ListItem::new(line)
            })
            .collect()
    };

    let indices = app.filtered_indices();
    let items: Vec<ListItem> = indices
        .iter()
        .filter_map(|&i| items.get(i).cloned())
        .collect();

    let tab_name = app.active_tab.display_name();
    let title = if app.filter_active() {
        let cursor = if app.filter_editing { "\u{2588}" } else { "" };
        format!(" {tab_name} ({})  /{}{cursor} ", items.len(), app.filter)
    } else {
        format!(" {tab_name} ({}) ", items.len())
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color))
                .title(Line::from(Span::styled(
                    title,
                    Style::default()
                        .fg(palette.title)
                        .add_modifier(Modifier::BOLD)
                )))
        )
        .highlight_style(
            Style::default()
                .fg(palette.bg)
                .bg(palette.accent)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol("\u{2503} ");

    let mut state = ListState::default();
    state.select(Some(app.selected));
    frame.render_stateful_widget(list, area, &mut state);
}

/// Widget wrapper for the resource list panel.
pub struct ResourceListWidget {
    enabled: bool
}

impl ResourceListWidget {
    /// Creates a new resource list widget with enabled state.
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
}

impl crate::tui::widgets::Widget for ResourceListWidget {
    fn id(&self) -> &'static str {
        "resource_list"
    }

    fn name(&self) -> &'static str {
        "Resource List"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        let border_color = if app.focus == crate::tui::app::Focus::ResourceList {
            app.theme.palette().accent
        } else {
            app.theme.palette().border
        };
        render(frame, area, app, border_color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::themes::Theme;

    #[test]
    fn server_status_on_is_running() {
        let palette = Theme::GruvboxDark.palette();
        let (icon, color, label) = server_status_view("On", &palette);
        assert_eq!(icon, "\u{25B6}");
        assert_eq!(color, palette.success);
        assert_eq!(label, "running");
    }

    #[test]
    fn server_status_off_is_stopped() {
        let palette = Theme::GruvboxDark.palette();
        let (icon, color, label) = server_status_view("Off", &palette);
        assert_eq!(icon, "\u{25CB}");
        assert_eq!(color, palette.error);
        assert_eq!(label, "stopped");
    }

    #[test]
    fn server_status_transitional_is_warning() {
        let palette = Theme::GruvboxDark.palette();
        let (_, color, label) = server_status_view("Rebooting", &palette);
        assert_eq!(color, palette.warning);
        assert_eq!(label, "rebooting");
    }

    #[test]
    fn server_status_unknown_falls_back_to_lowercased() {
        let palette = Theme::GruvboxDark.palette();
        let (_, color, label) = server_status_view("SomethingNew", &palette);
        assert_eq!(color, palette.warning);
        assert_eq!(label, "somethingnew");
    }
}
