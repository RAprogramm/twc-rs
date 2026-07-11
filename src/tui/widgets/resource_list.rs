// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Resource list widget — shows the active tab's resources as a responsive
//! grid of data cards that flex to fill the whole content area.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph}
};
use rust_i18n::t;

use crate::tui::{
    app::{App, ResourceTab},
    themes::Palette,
    widgets::overview
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

/// A single resource rendered as a card: a title, an optional colored status
/// badge and a secondary metric line.
struct Card {
    title:  String,
    status: Option<(Color, String)>,
    meta:   String
}

/// Builds the card view-models for every item on the active tab, in list order.
// JUSTIFY: Large match expression covering all resource types.
#[allow(clippy::too_many_lines)]
fn build_cards(app: &App, palette: &Palette) -> Vec<Card> {
    match app.active_tab {
        ResourceTab::Servers => app
            .servers
            .iter()
            .map(|s| {
                let (_, color, label) = server_status_view(&s.status, palette);
                Card {
                    title:  s.name.clone(),
                    status: Some((color, label)),
                    meta:   format!("{}c · {} MB · {}", s.cpu, s.ram_mb, s.location)
                }
            })
            .collect(),
        ResourceTab::Databases => app
            .databases
            .iter()
            .map(|d| {
                let (color, label) = status_view(&d.status, palette);
                Card {
                    title:  d.name.clone(),
                    status: Some((color, label)),
                    meta:   format!("{} · {} MB", d.engine, d.size_mb)
                }
            })
            .collect(),
        ResourceTab::S3 => app
            .s3_storages
            .iter()
            .map(|s| Card {
                title:  s.name.clone(),
                status: None,
                meta:   format!("{} · {} obj", s.region, s.object_count)
            })
            .collect(),
        ResourceTab::Kubernetes => app
            .k8s_clusters
            .iter()
            .map(|c| {
                let (color, label) = status_view(&c.status, palette);
                Card {
                    title:  c.name.clone(),
                    status: Some((color, label)),
                    meta:   format!("v{} · {}c · {} MB", c.version, c.cpu, c.ram_mb)
                }
            })
            .collect(),
        ResourceTab::Projects => app
            .projects
            .iter()
            .map(|p| Card {
                title:  p.name.clone(),
                status: None,
                meta:   t!("resource_list.count_resources", n => p.resource_count()).to_string()
            })
            .collect(),
        ResourceTab::Balancers => app
            .balancers
            .iter()
            .map(|b| {
                let (color, label) = status_view(&b.status, palette);
                Card {
                    title:  b.name.clone(),
                    status: Some((color, label)),
                    meta:   format!("{} · {}", b.ip, b.location)
                }
            })
            .collect(),
        ResourceTab::Registry => app
            .registries
            .iter()
            .map(|r| Card {
                title:  r.name.clone(),
                status: None,
                meta:   t!("resource_list.disk_used", pct => registry_used_percent(r)).to_string()
            })
            .collect(),
        ResourceTab::Domains => app
            .domains
            .iter()
            .map(|d| {
                let (color, label) = status_view(&d.status, palette);
                let prolong = if d.auto_prolong {
                    t!("resource_list.auto_prolong")
                } else {
                    t!("resource_list.manual_prolong")
                };
                Card {
                    title:  d.name.clone(),
                    status: Some((color, label)),
                    meta:   prolong.to_string()
                }
            })
            .collect(),
        ResourceTab::Firewall => app
            .firewalls
            .iter()
            .map(|f| Card {
                title:  f.name.clone(),
                status: None,
                meta:   f.policy.clone()
            })
            .collect(),
        ResourceTab::FloatingIps => app
            .floating_ips
            .iter()
            .map(|f| {
                let (color, label) = status_view(&f.status, palette);
                Card {
                    title:  f.ip.clone(),
                    status: Some((color, label)),
                    meta:   f.server_name.clone()
                }
            })
            .collect(),
        ResourceTab::Images => app
            .images
            .iter()
            .map(|i| {
                let (color, label) = status_view(&i.status, palette);
                Card {
                    title:  i.name.clone(),
                    status: Some((color, label)),
                    meta:   format!("{} MB", i.size_mb)
                }
            })
            .collect(),
        ResourceTab::NetworkDrives => app
            .network_drives
            .iter()
            .map(|n| {
                let (color, label) = status_view(&n.status, palette);
                Card {
                    title:  n.name.clone(),
                    status: Some((color, label)),
                    meta:   format!("{} GB", n.size_gb)
                }
            })
            .collect(),
        ResourceTab::Vpc => app
            .vpcs
            .iter()
            .map(|v| Card {
                title:  v.name.clone(),
                status: None,
                meta:   format!("{} · {}", v.subnet, v.location)
            })
            .collect(),
        ResourceTab::DedicatedServers => app
            .dedicated_servers
            .iter()
            .map(|d| {
                let (color, label) = status_view(&d.status, palette);
                Card {
                    title:  d.name.clone(),
                    status: Some((color, label)),
                    meta:   format!("{} · {}", d.cpu, d.ram)
                }
            })
            .collect(),
        ResourceTab::Mail => app
            .mails
            .iter()
            .map(|m| Card {
                title:  m.name.clone(),
                status: None,
                meta:   m.owner.clone()
            })
            .collect(),
        ResourceTab::Apps => app
            .apps
            .iter()
            .map(|a| {
                let (color, label) = status_view(&a.status, palette);
                Card {
                    title:  a.name.clone(),
                    status: Some((color, label)),
                    meta:   format!("{} · {}", a.location, a.framework)
                }
            })
            .collect(),
        ResourceTab::AiAgents => app
            .ai_agents
            .iter()
            .map(|a| {
                let (color, label) = status_view(&a.status, palette);
                Card {
                    title:  a.name.clone(),
                    status: Some((color, label)),
                    meta:   format!("{}/{} tok", a.tokens_used, a.tokens_total)
                }
            })
            .collect(),
        ResourceTab::KnowledgeBases => app
            .knowledge_bases
            .iter()
            .map(|k| {
                let (color, label) = status_view(&k.status, palette);
                Card {
                    title:  k.name.clone(),
                    status: Some((color, label)),
                    meta:   t!("resource_list.count_docs", n => k.document_count).to_string()
                }
            })
            .collect(),
        ResourceTab::SshKeys => app
            .ssh_keys
            .iter()
            .map(|k| Card {
                title:  k.clone(),
                status: None,
                meta:   String::new()
            })
            .collect(),
        ResourceTab::Finances => app
            .finances
            .iter()
            .map(|f| Card {
                title:  f.clone(),
                status: None,
                meta:   String::new()
            })
            .collect()
    }
}

/// Renders the resource card grid into `area` with a given border color.
///
/// # Arguments
///
/// * `frame` - The render frame.
/// * `area` - The area to render in.
/// * `app` - The application state.
/// * `border_color` - Color for the outer panel border.
pub fn render(frame: &mut Frame, area: Rect, app: &App, border_color: Color) {
    let palette = app.theme.palette();

    let all = build_cards(app, &palette);
    let indices = app.filtered_indices();
    let cards: Vec<&Card> = indices.iter().filter_map(|&i| all.get(i)).collect();

    let tab_name = app.active_tab.display_name();
    let title = if app.filter_active() {
        let cursor = if app.filter_editing { "\u{2588}" } else { "" };
        format!(" {tab_name} ({})  /{}{cursor} ", cards.len(), app.filter)
    } else {
        format!(" {tab_name} ({}) ", cards.len())
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Line::from(Span::styled(
            title,
            Style::default()
                .fg(palette.title)
                .add_modifier(Modifier::BOLD)
        )));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if cards.is_empty() {
        let empty = Paragraph::new(Line::from(Span::styled(
            t!("ui.drill_empty").to_string(),
            Style::default().fg(palette.dim)
        )));
        frame.render_widget(empty, inner);
        return;
    }

    render_grid(frame, inner, &cards, app, &palette);
}

/// Total height of a card cell (border included) when the grid must scroll.
const CARD_H: u16 = 5;
/// Vertical gap between card rows, in rows.
const VGAP: u16 = 1;
/// Horizontal gap between card cells, in cells.
const HGAP: u16 = 2;

fn render_grid(frame: &mut Frame, inner: Rect, cards: &[&Card], app: &App, palette: &Palette) {
    if inner.height < 3 || inner.width < 6 {
        return;
    }
    let icon = overview::tab_icon(app.active_tab);
    let cols = overview::columns_for(inner.width);
    let rows_total = cards.len().div_ceil(cols);
    let rows_fit = usize::from((inner.height + VGAP) / (CARD_H + VGAP)).max(1);

    let selected_row = app.selected / cols;
    let first_row = if rows_total <= rows_fit || selected_row < rows_fit {
        0
    } else {
        selected_row + 1 - rows_fit
    };

    let stretch = rows_total <= rows_fit;
    let visible_rows = if stretch {
        rows_total
    } else {
        rows_fit.min(rows_total - first_row)
    };

    let row_constraints: Vec<Constraint> = if stretch {
        vec![Constraint::Fill(1); visible_rows]
    } else {
        vec![Constraint::Length(CARD_H); visible_rows]
    };
    let row_areas = Layout::vertical(row_constraints).spacing(VGAP).split(inner);

    for (ri, row_area) in row_areas.iter().enumerate() {
        let grid_row = first_row + ri;
        let cells = Layout::horizontal(vec![Constraint::Fill(1); cols])
            .spacing(HGAP)
            .split(*row_area);
        for (ci, cell) in cells.iter().enumerate() {
            let idx = grid_row * cols + ci;
            let Some(card) = cards.get(idx) else {
                break;
            };
            render_card(frame, *cell, card, icon, idx == app.selected, palette);
        }
    }
}

fn render_card(
    frame: &mut Frame,
    rect: Rect,
    card: &Card,
    icon: &str,
    selected: bool,
    palette: &Palette
) {
    if rect.height < 3 || rect.width < 6 {
        return;
    }
    let border = if selected {
        palette.accent
    } else {
        palette.border
    };
    let inner_w = usize::from(rect.width.saturating_sub(4));

    let title_line = Line::from(vec![
        Span::styled(format!("{icon}  "), Style::default().fg(palette.accent)),
        Span::styled(
            truncate(&card.title, inner_w.saturating_sub(3)),
            Style::default().fg(palette.fg).add_modifier(Modifier::BOLD)
        ),
    ]);

    let mut lines = vec![title_line];
    if let Some((color, label)) = &card.status {
        lines.push(Line::from(Span::styled(
            format!("\u{25CF} {}", truncate(label, inner_w.saturating_sub(2))),
            Style::default().fg(*color)
        )));
    }
    if !card.meta.is_empty() {
        lines.push(Line::from(Span::styled(
            truncate(&card.meta, inner_w),
            Style::default().fg(palette.dim)
        )));
    }

    let filled = lines.len();
    let capacity = usize::from(rect.height.saturating_sub(2));
    for _ in filled..capacity {
        lines.push(Line::from(""));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border));
    frame.render_widget(Paragraph::new(lines).block(block), rect);
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('\u{2026}');
    out
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

    fn app_with_servers(n: i32) -> App {
        use crate::tui::app::ServerSummary;
        let mut app = App::new(5);
        app.servers = (0..n)
            .map(|i| ServerSummary {
                id:       i,
                name:     format!("srv-{i}"),
                status:   "On".to_string(),
                cpu:      2,
                ram_mb:   4096,
                disk_gb:  40,
                ip:       "1.2.3.4".to_string(),
                location: "ru-1".to_string()
            })
            .collect();
        app
    }

    fn draw_at(app: &App, w: u16, h: u16) {
        use ratatui::{Terminal, backend::TestBackend};
        let backend = TestBackend::new(w.max(1), h.max(1));
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                render(
                    frame,
                    Rect::new(0, 0, w, h),
                    app,
                    app.theme.palette().accent
                )
            })
            .unwrap();
    }

    #[test]
    fn grid_renders_across_sizes_without_panic() {
        let app = app_with_servers(12);
        for (w, h) in [(0, 0), (1, 1), (3, 3), (20, 6), (80, 24), (200, 60)] {
            draw_at(&app, w, h);
        }
    }

    #[test]
    fn grid_renders_empty_and_single_item() {
        draw_at(&app_with_servers(0), 80, 24);
        draw_at(&app_with_servers(1), 80, 24);
    }

    #[test]
    fn grid_renders_with_selection_scrolled_to_end() {
        let mut app = app_with_servers(40);
        app.resource_cols = 3;
        app.selected = 39;
        draw_at(&app, 60, 12);
    }
}
