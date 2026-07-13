// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! The service info header shown at the top of the content pane: what the
//! product is, a Create button, and a separator above the resource grid.

use std::borrow::Cow;

use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Paragraph, Wrap}
};
use rust_i18n::t;

use crate::tui::{app::ResourceTab, themes::Palette};

/// The description and Create-button label for a service tab, localized.
#[must_use]
pub fn texts(tab: ResourceTab) -> (Cow<'static, str>, Cow<'static, str>) {
    match tab {
        ResourceTab::Servers => (t!("service.desc.servers"), t!("service.create.servers")),
        ResourceTab::Databases => (t!("service.desc.databases"), t!("service.create.databases")),
        ResourceTab::S3 => (t!("service.desc.s3"), t!("service.create.s3")),
        ResourceTab::Kubernetes => (
            t!("service.desc.kubernetes"),
            t!("service.create.kubernetes")
        ),
        ResourceTab::Balancers => (t!("service.desc.balancers"), t!("service.create.balancers")),
        ResourceTab::Registry => (t!("service.desc.registry"), t!("service.create.registry")),
        ResourceTab::Domains => (t!("service.desc.domains"), t!("service.create.domains")),
        ResourceTab::Firewall => (t!("service.desc.firewall"), t!("service.create.firewall")),
        ResourceTab::FloatingIps => (
            t!("service.desc.floating_ips"),
            t!("service.create.floating_ips")
        ),
        ResourceTab::Images => (t!("service.desc.images"), t!("service.create.images")),
        ResourceTab::NetworkDrives => (
            t!("service.desc.network_drives"),
            t!("service.create.network_drives")
        ),
        ResourceTab::Vpc => (t!("service.desc.vpc"), t!("service.create.vpc")),
        ResourceTab::DedicatedServers => {
            (t!("service.desc.dedicated"), t!("service.create.dedicated"))
        }
        ResourceTab::Mail => (t!("service.desc.mail"), t!("service.create.mail")),
        ResourceTab::Apps => (t!("service.desc.apps"), t!("service.create.apps")),
        ResourceTab::AiAgents => (t!("service.desc.ai_agents"), t!("service.create.ai_agents")),
        ResourceTab::KnowledgeBases => (
            t!("service.desc.knowledge_bases"),
            t!("service.create.knowledge_bases")
        ),
        ResourceTab::Projects => (t!("service.desc.projects"), t!("service.create.projects")),
        ResourceTab::SshKeys => (t!("service.desc.ssh_keys"), t!("service.create.ssh_keys")),
        ResourceTab::Finances => (t!("service.desc.finances"), Cow::Borrowed(""))
    }
}

/// Rows the description text occupies when wrapped to `width` columns,
/// with slack for word wrapping (words break earlier than a character count
/// suggests).
fn desc_rows(desc: &str, width: u16) -> u16 {
    let cols = usize::from(width.saturating_sub(8).max(1));
    let rows = desc.chars().count().div_ceil(cols);
    u16::try_from(rows.clamp(1, 3)).unwrap_or(1)
}

/// Total height of the header block (description, optional button and the
/// separator) at the given inner width; 0 when the tab has no product
/// description. Tabs without a create label skip the button row.
#[must_use]
fn height(tab: ResourceTab, width: u16) -> u16 {
    let (desc, create) = texts(tab);
    if desc.is_empty() {
        return 0;
    }
    desc_rows(&desc, width) + 1 + u16::from(!create.is_empty())
}

/// Splits a panel's inner area into the header block and the space below it.
///
/// The single source of truth for whether and how tall the header renders:
/// callers just render into whatever areas come back. `None` means there is
/// no header (no description for the tab, or the panel is too small to fit
/// one and still show content).
#[must_use]
pub fn layout(inner: Rect, tab: ResourceTab) -> (Option<Rect>, Rect) {
    let needed = height(tab, inner.width);
    if needed == 0 || inner.height < needed + 1 {
        return (None, inner);
    }
    let areas = ratatui::layout::Layout::vertical([
        ratatui::layout::Constraint::Length(needed),
        ratatui::layout::Constraint::Min(1)
    ])
    .split(inner);
    (Some(areas[0]), areas[1])
}

/// Renders the header block above a resource grid.
///
/// Shows the wrapped product description, a Create button chip (filled while
/// focused) and a separator line. The panel border already carries the
/// product name, so it is not repeated here.
pub fn render(
    frame: &mut Frame,
    area: Rect,
    tab: ResourceTab,
    button_focused: bool,
    palette: &Palette
) {
    let (desc, create) = texts(tab);
    let chrome_rows = 1 + u16::from(!create.is_empty());
    if desc.is_empty() || area.height < chrome_rows + 1 {
        return;
    }

    let text_rows = area.height.saturating_sub(chrome_rows);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            desc.into_owned(),
            Style::default().fg(palette.dim)
        )))
        .wrap(Wrap {
            trim: true
        }),
        Rect::new(area.x, area.y, area.width, text_rows)
    );

    let mut next_row = area.y + text_rows;
    if !create.is_empty() {
        let button =
            crate::tui::widgets::button::chip(&format!("+ {create}"), button_focused, palette);
        frame.render_widget(
            Paragraph::new(button),
            Rect::new(area.x, next_row, area.width, 1)
        );
        next_row += 1;
    }

    let separator = Line::from(Span::styled(
        "\u{2500}".repeat(usize::from(area.width)),
        Style::default().fg(palette.border)
    ));
    frame.render_widget(
        Paragraph::new(separator),
        Rect::new(area.x, next_row, area.width, 1)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn description_renders_on_wide_panels() {
        use ratatui::{Terminal, backend::TestBackend, layout::Rect};

        use crate::tui::app::{App, Pane};
        let mut app = App::new(5);
        app.active_tab = ResourceTab::Servers;
        app.pane = Pane::Content;
        app.content_on_create = true;
        for w in [60u16, 120, 200] {
            let backend = TestBackend::new(w, 14);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|f| {
                    crate::tui::widgets::resource_list::render(
                        f,
                        Rect::new(0, 0, w, 14),
                        &app,
                        app.theme.palette().accent
                    );
                })
                .unwrap();
            let buf = terminal.backend().buffer().clone();
            let mut text = String::new();
            for y in 0..14 {
                for x in 0..w {
                    text.push_str(buf[(x, y)].symbol());
                }
            }
            assert!(
                text.contains("Cloud servers"),
                "description missing at width {w}"
            );
            assert!(
                text.contains("Create server"),
                "button missing at width {w}"
            );
        }
    }

    #[test]
    fn every_service_tab_has_texts() {
        for tab in crate::tui::app::App::service_tabs() {
            let (desc, create) = texts(tab);
            assert!(!desc.is_empty(), "{tab:?} missing description");
            assert!(!create.is_empty(), "{tab:?} missing create label");
        }
    }

    #[test]
    fn height_scales_with_width() {
        assert!(height(ResourceTab::Servers, 30) >= height(ResourceTab::Servers, 200));
        assert!(height(ResourceTab::Projects, 80) > 0);
    }

    #[test]
    fn finances_header_has_description_but_no_button_row() {
        let (desc, create) = texts(ResourceTab::Finances);
        assert!(!desc.is_empty());
        assert!(create.is_empty());
        assert_eq!(
            height(ResourceTab::Finances, 200),
            height(ResourceTab::Servers, 200) - 1
        );
    }
}
