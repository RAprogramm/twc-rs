// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! The service info header shown at the top of the content pane: what the
//! product is, a Create button, and a separator above the resource grid.

use std::borrow::Cow;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap}
};
use rust_i18n::t;

use crate::tui::{app::ResourceTab, themes::Palette, widgets::sidebar::tab_icon};

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
        ResourceTab::Projects | ResourceTab::SshKeys | ResourceTab::Finances => {
            (Cow::Borrowed(""), Cow::Borrowed(""))
        }
    }
}

/// Rows the description text occupies when wrapped to `width` columns.
fn desc_rows(desc: &str, width: u16) -> u16 {
    let cols = usize::from(width.max(1));
    let rows = desc.chars().count().div_ceil(cols);
    u16::try_from(rows.clamp(1, 3)).unwrap_or(1)
}

/// Total height of the header block (title, description, button, separator)
/// at the given inner width; 0 when the tab has no product description.
#[must_use]
pub fn height(tab: ResourceTab, width: u16) -> u16 {
    let (desc, _) = texts(tab);
    if desc.is_empty() {
        return 0;
    }
    1 + desc_rows(&desc, width) + 2
}

/// Renders the header block: icon + product name, wrapped description, the
/// Create button (highlighted while focused) and a separator line.
pub fn render(
    frame: &mut Frame,
    area: Rect,
    tab: ResourceTab,
    button_focused: bool,
    palette: &Palette
) {
    let (desc, create) = texts(tab);
    if desc.is_empty() || area.height < 4 {
        return;
    }

    let mut lines: Vec<Line> = Vec::with_capacity(usize::from(area.height));
    lines.push(Line::from(vec![
        Span::styled(
            format!("{}  ", tab_icon(tab)),
            Style::default().fg(palette.accent)
        ),
        Span::styled(
            tab.display_name().into_owned(),
            Style::default()
                .fg(palette.title)
                .add_modifier(Modifier::BOLD)
        ),
    ]));
    lines.push(Line::from(Span::styled(
        desc.into_owned(),
        Style::default().fg(palette.dim)
    )));

    let text_rows = area.height.saturating_sub(2);
    let text_area = Rect::new(area.x, area.y, area.width, text_rows);
    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap {
            trim: true
        }),
        text_area
    );

    let button_style = if button_focused {
        Style::default()
            .fg(palette.bg)
            .bg(palette.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(palette.accent)
            .add_modifier(Modifier::BOLD)
    };
    let button = Line::from(Span::styled(format!("[ + {create} ]"), button_style));
    frame.render_widget(
        Paragraph::new(button),
        Rect::new(area.x, area.y + text_rows, area.width, 1)
    );

    let separator = Line::from(Span::styled(
        "\u{2500}".repeat(usize::from(area.width)),
        Style::default().fg(palette.border)
    ));
    frame.render_widget(
        Paragraph::new(separator),
        Rect::new(area.x, area.y + text_rows + 1, area.width, 1)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(height(ResourceTab::Projects, 80), 0);
    }
}
