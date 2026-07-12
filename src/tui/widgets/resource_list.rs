// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Resource list panel — renders the active tab's resources through the shared
//! [`card_grid`], and exposes the status-to-color helpers reused elsewhere.

use ratatui::{Frame, layout::Rect, style::Color};
use rust_i18n::t;

use crate::tui::{
    app::App,
    themes::Palette,
    widgets::{card_grid, resource_cards}
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

/// Renders the resource card grid into `area` with a given border color.
pub fn render(frame: &mut Frame, area: Rect, app: &App, border_color: Color) {
    let palette = app.theme.palette();

    let all = resource_cards::build(app, &palette);
    let indices = app.filtered_indices();
    let cards: Vec<card_grid::GridCard> = indices
        .iter()
        .filter_map(|&i| all.get(i).cloned())
        .collect();

    let tab_name = app.active_tab.display_name();
    let title = if app.filter_active() {
        let cursor = if app.filter_editing { "\u{2588}" } else { "" };
        format!(" {tab_name} ({})  /{}{cursor} ", cards.len(), app.filter)
    } else {
        format!(" {tab_name} ({}) ", cards.len())
    };

    let cols = card_grid::columns(
        area.width.saturating_sub(2),
        card_grid::longest_title(&cards)
    );
    let selected = if app.pane == crate::tui::app::Pane::Content {
        app.selected
    } else {
        usize::MAX
    };
    let empty = t!("ui.drill_empty");
    card_grid::render(
        frame,
        area,
        &title,
        &cards,
        selected,
        cols,
        empty.as_ref(),
        border_color,
        &palette
    );
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
    use ratatui::{Terminal, backend::TestBackend};

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
