// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Details widget — shows information about the selected resource.

mod resources;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph}
};
use resources::{
    render_ai_agent_details, render_app_details, render_balancer_details, render_database_details,
    render_dedicated_details, render_domain_details, render_firewall_details,
    render_floating_ip_details, render_image_details, render_k8s_details,
    render_knowledge_details, render_mail_details, render_network_drive_details,
    render_project_details, render_registry_details, render_s3_details, render_server_details,
    render_string_details, render_vpc_details
};
use rust_i18n::t;

use crate::tui::{
    app::{App, ResourceTab},
    themes::Palette
};

const KEY_WIDTH: usize = 13;
const RULE_WIDTH: usize = 32;

/// Renders the details panel for the selected resource.
///
/// # Arguments
///
/// * `frame` - The render frame.
/// * `area` - The area to render in.
/// * `app` - The application state.
/// * `border_color` - Color for the panel border.
pub fn render(frame: &mut Frame, area: Rect, app: &App, border_color: Color) -> u16 {
    let palette = app.theme.palette();

    let text = match app.active_tab {
        ResourceTab::Servers => render_server_details(app, palette),
        ResourceTab::Databases => render_database_details(app, palette),
        ResourceTab::S3 => render_s3_details(app, palette),
        ResourceTab::Kubernetes => render_k8s_details(app, palette),
        ResourceTab::Projects => render_project_details(app, palette),
        ResourceTab::Balancers => render_balancer_details(app, palette),
        ResourceTab::Registry => render_registry_details(app, palette),
        ResourceTab::Domains => render_domain_details(app, palette),
        ResourceTab::Firewall => render_firewall_details(app, palette),
        ResourceTab::FloatingIps => render_floating_ip_details(app, palette),
        ResourceTab::Images => render_image_details(app, palette),
        ResourceTab::NetworkDrives => render_network_drive_details(app, palette),
        ResourceTab::Vpc => render_vpc_details(app, palette),
        ResourceTab::DedicatedServers => render_dedicated_details(app, palette),
        ResourceTab::Mail => render_mail_details(app, palette),
        ResourceTab::Apps => render_app_details(app, palette),
        ResourceTab::AiAgents => render_ai_agent_details(app, palette),
        ResourceTab::KnowledgeBases => render_knowledge_details(app, palette),
        ResourceTab::SshKeys => {
            render_string_details(&app.ssh_keys, app, &t!("details.ssh_key"), palette)
        }
        ResourceTab::Finances => {
            render_string_details(&app.finances, app, &t!("details.finance"), palette)
        }
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Line::from(Span::styled(
            format!(" {} ", breadcrumbs(app)),
            Style::default()
                .fg(palette.title)
                .add_modifier(Modifier::BOLD)
        )));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 || inner.width == 0 {
        return 0;
    }

    let mut text = text;
    append_extra_sections(&mut text, app, palette);
    render_columns(frame, inner, text, app.detail_scroll)
}

/// Appends the background-fetched deep-detail sections (connection, nested
/// databases, tariff, ...) for the resource currently shown, when loaded.
fn append_extra_sections(text: &mut Vec<Line<'static>>, app: &App, palette: Palette) {
    let Some(id) = app
        .selected_resource()
        .and_then(|(id, _)| id.parse::<i32>().ok())
    else {
        return;
    };
    let Some(sections) = app.detail_extra.get(&(app.active_tab, id)) else {
        return;
    };
    for (title, rows) in sections {
        text.push(Line::from(""));
        text.push(heading(title, palette));
        text.push(rule(palette));
        for (key, value) in rows {
            text.push(kv(key, value.clone(), name_style(palette), palette));
        }
    }
}

/// Builds the breadcrumb trail for the panel border: the project the user
/// drilled through (when any) and the resource the details describe.
fn breadcrumbs(app: &App) -> String {
    let name = app
        .selected_resource()
        .map_or_else(|| t!("details.title").into_owned(), |(_, name)| name);
    app.drill_view().map_or_else(
        || format!("{} \u{2192} {name}", app.active_tab.display_name()),
        |drill| format!("{} \u{2192} {name}", drill.title)
    )
}

/// Horizontal gap between details columns.
const COLUMN_GAP: u16 = 3;

/// Lays the detail lines out smartly: when they exceed the panel height and
/// the panel is wide enough, they flow into additional columns instead of
/// hiding below the fold. The column width comes from the widest line of the
/// actual content, so nothing is hardcoded to a terminal size. Scrolling only
/// kicks in once every column is full.
///
/// Returns the clamped scroll offset actually used, so the caller can write
/// it back and the scroll state never runs past the content.
fn render_columns(frame: &mut Frame, inner: Rect, text: Vec<Line<'static>>, scroll: u16) -> u16 {
    let height = usize::from(inner.height);
    let column_width = u16::try_from(text.iter().map(Line::width).max().unwrap_or(0))
        .unwrap_or(u16::MAX)
        .clamp(1, inner.width);
    let max_cols = usize::from((inner.width + COLUMN_GAP) / (column_width + COLUMN_GAP)).max(1);
    let needed = text.len().div_ceil(height.max(1));
    let cols = needed.clamp(1, max_cols);

    let capacity = cols * height;
    let max_scroll = text.len().saturating_sub(capacity);
    let offset = usize::from(scroll).min(max_scroll);
    let clamped = u16::try_from(offset).unwrap_or(u16::MAX);
    let visible: Vec<Line> = text.into_iter().skip(offset).take(capacity).collect();

    if cols == 1 {
        frame.render_widget(Paragraph::new(visible), inner);
        return clamped;
    }

    let mut constraints = Vec::with_capacity(cols);
    for _ in 0..cols - 1 {
        constraints.push(ratatui::layout::Constraint::Length(column_width));
    }
    constraints.push(ratatui::layout::Constraint::Min(10));
    let areas = ratatui::layout::Layout::horizontal(constraints)
        .spacing(COLUMN_GAP)
        .split(inner);

    for (chunk, column) in visible.chunks(height).zip(areas.iter()) {
        frame.render_widget(Paragraph::new(chunk.to_vec()), *column);
    }
    clamped
}

/// Builds the bold heading line shown at the top of a populated panel.
pub(super) fn heading(name: &str, palette: Palette) -> Line<'static> {
    Line::from(Span::styled(
        name.to_string(),
        Style::default()
            .fg(palette.title)
            .add_modifier(Modifier::BOLD)
    ))
}

/// Builds a dim horizontal rule used to separate sections.
pub(super) fn rule(palette: Palette) -> Line<'static> {
    Line::from(Span::styled(
        "\u{2500}".repeat(RULE_WIDTH),
        Style::default().fg(palette.dim)
    ))
}

/// Builds a dim, bold section header line.
pub(super) fn section(label: &str, palette: Palette) -> Line<'static> {
    Line::from(Span::styled(
        label.to_string(),
        Style::default()
            .fg(palette.header)
            .add_modifier(Modifier::BOLD)
    ))
}

/// Builds a key/value row, dimming the key via the palette's dim color.
/// Keys longer than the standard column keep at least two spaces before the
/// value instead of gluing to it.
pub(super) fn kv(key: &str, value: String, value_style: Style, palette: Palette) -> Line<'static> {
    let padded = if key.chars().count() >= KEY_WIDTH {
        format!("{key}  ")
    } else {
        format!("{key:<KEY_WIDTH$}")
    };
    Line::from(vec![
        Span::styled(padded, Style::default().fg(palette.dim)),
        Span::styled(value, value_style),
    ])
}

/// Builds a key/value row that falls back to a dim `\u{2014}` when the value is
/// empty, keeping the field visible so the panel layout stays stable across
/// selections instead of collapsing rows in and out.
pub(super) fn kv_field(
    key: &str,
    value: &str,
    value_style: Style,
    palette: Palette
) -> Line<'static> {
    if value.is_empty() {
        return kv(
            key,
            "\u{2014}".to_string(),
            Style::default().fg(palette.dim),
            palette
        );
    }
    kv(key, value.to_string(), value_style, palette)
}

/// Builds a status row rendered as a colored `\u{25CF} label` chip.
pub(super) fn chip(key: &str, label: &str, color: Color, palette: Palette) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{key:<KEY_WIDTH$}"),
            Style::default().fg(palette.dim)
        ),
        Span::styled(
            format!("\u{25CF} {label}"),
            Style::default().fg(color).add_modifier(Modifier::BOLD)
        ),
    ])
}

/// Builds a status chip colored by the generic status classifier from
/// [`crate::tui::widgets::resource_list::status_view`].
pub(super) fn status_chip(key: &str, status: &str, palette: Palette) -> Line<'static> {
    let (color, label) = crate::tui::widgets::resource_list::status_view(status, &palette);
    chip(key, &label, color, palette)
}

/// Builds a centered, dim empty-state notice.
pub(super) fn empty(message: &str, palette: Palette) -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {message}"),
            Style::default()
                .fg(palette.dim)
                .add_modifier(Modifier::ITALIC)
        )),
    ]
}

pub(super) fn accent(palette: Palette) -> Style {
    Style::default().fg(palette.accent)
}

pub(super) fn name_style(palette: Palette) -> Style {
    Style::default().fg(palette.fg).add_modifier(Modifier::BOLD)
}

pub(super) fn warn(palette: Palette) -> Style {
    Style::default().fg(palette.warning)
}

/// Widget wrapper for the details panel.
pub struct DetailsWidget {
    enabled: bool
}

impl DetailsWidget {
    /// Creates a new details widget with enabled state.
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

impl crate::tui::widgets::Widget for DetailsWidget {
    fn id(&self) -> &'static str {
        "details"
    }

    fn name(&self) -> &'static str {
        "Details"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        let border_color = if app.focus == crate::tui::app::Focus::Details {
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
    use crate::tui::app::{App, DatabaseSummary};

    #[test]
    fn scroll_state_clamps_to_content_each_frame() {
        let mut app = App::new(5);
        app.active_tab = crate::tui::app::ResourceTab::Databases;
        app.databases = vec![DatabaseSummary {
            id: 1,
            name: "db".to_string(),
            status: "started".to_string(),
            engine: "postgres".to_string(),
            size_mb: 100,
            ..Default::default()
        }];
        app.detail_scroll = 500;
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                app.detail_scroll = render(f, Rect::new(0, 0, 80, 24), &app, Color::Reset);
            })
            .unwrap();
        assert_eq!(
            app.detail_scroll, 0,
            "short content must clamp runaway scroll back to zero"
        );
    }

    #[test]
    fn long_details_flow_into_columns_on_wide_panels() {
        let mut app = App::new(5);
        app.active_tab = crate::tui::app::ResourceTab::Databases;
        app.databases = vec![DatabaseSummary {
            id: 1,
            name: "db".to_string(),
            status: "started".to_string(),
            engine: "postgres".to_string(),
            size_mb: 100,
            config: (0..40)
                .map(|i| (format!("param_{i}"), i.to_string()))
                .collect(),
            ..Default::default()
        }];
        let (w, h) = (140u16, 14u16);
        let backend = TestBackend::new(w, h);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                render(f, Rect::new(0, 0, w, h), &app, Color::Reset);
            })
            .unwrap();
        let buf = terminal.backend().buffer().clone();
        let mut second_column = String::new();
        for y in 1..h - 1 {
            for x in w / 3..w - 1 {
                second_column.push_str(buf[(x, y)].symbol());
            }
        }
        assert!(
            second_column.contains("param_"),
            "expected a second column of parameters on a wide panel"
        );
    }
}
