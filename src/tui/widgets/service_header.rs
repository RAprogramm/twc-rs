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
        ResourceTab::SshKeys | ResourceTab::Finances => (Cow::Borrowed(""), Cow::Borrowed(""))
    }
}

/// Rows the description text occupies when wrapped to `width` columns.
fn desc_rows(desc: &str, width: u16) -> u16 {
    let cols = usize::from(width.max(1));
    let rows = desc.chars().count().div_ceil(cols);
    u16::try_from(rows.clamp(1, 3)).unwrap_or(1)
}

/// Total height of the header block (description, button, separator) at the
/// given inner width; 0 when the tab has no product description.
#[must_use]
pub fn height(tab: ResourceTab, width: u16) -> u16 {
    let (desc, _) = texts(tab);
    if desc.is_empty() {
        return 0;
    }
    desc_rows(&desc, width) + 2
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
    if desc.is_empty() || area.height < 3 {
        return;
    }

    let text_rows = area.height.saturating_sub(2);
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

    let (chip_bg, chip_fg) = if button_focused {
        (palette.accent, palette.bg)
    } else {
        (palette.border, palette.fg)
    };
    let cap = Style::default().fg(chip_bg);
    let body = Style::default()
        .fg(chip_fg)
        .bg(chip_bg)
        .add_modifier(Modifier::BOLD);
    let button = Line::from(vec![
        Span::styled("\u{2590}", cap),
        Span::styled(format!(" + {create} "), body),
        Span::styled("\u{258C}", cap),
    ]);
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
        assert!(height(ResourceTab::Projects, 80) > 0);
        assert_eq!(height(ResourceTab::Finances, 80), 0);
    }
}
