// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Account widget — displays account information in a bordered panel.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph}
};

use crate::tui::app::{AccountInfo, App};

/// Renders the account information panel.
///
/// # Overview
///
/// Displays account ID, balance, and status inside a bordered block
/// styled with the current theme palette.
pub struct AccountWidget {
    enabled: bool
}

impl AccountWidget {
    /// Creates a new account widget with enabled state.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether the widget is initially visible.
    pub const fn new(enabled: bool) -> Self {
        Self {
            enabled
        }
    }

    /// Formats an f64 account ID as a clean integer string when possible.
    ///
    /// # Arguments
    ///
    /// * `id` - The account ID value.
    ///
    /// # Returns
    ///
    /// A string representation without trailing decimal points.
    fn format_account_id(id: f64) -> String {
        if id.fract() == 0.0 {
            format!("{id:.0}")
        } else {
            format!("{id}")
        }
    }

    /// Builds the styled lines for the account info panel.
    ///
    /// # Arguments
    ///
    /// * `account` - The account information to display.
    /// * `palette` - The theme color palette.
    fn build_lines(
        account: &AccountInfo,
        palette: crate::tui::themes::Palette
    ) -> Vec<Line<'static>> {
        let id_text = Self::format_account_id(account.account_id);
        let status_style = match account.status.as_str() {
            "active" | "running" | "enabled" => Style::default().fg(palette.success),
            "inactive" | "suspended" => Style::default().fg(palette.warning),
            "error" | "failed" => Style::default().fg(palette.error),
            _ => Style::default().fg(palette.fg)
        };

        vec![
            Line::from(Span::styled(
                format!("ID: {id_text}"),
                Style::default()
                    .fg(palette.header)
                    .add_modifier(Modifier::BOLD)
            )),
            Line::from(Span::styled(
                format!("Balance: {}", account.balance),
                Style::default().fg(palette.accent)
            )),
            Line::from(Span::styled(
                format!("Status: {}", account.status),
                status_style
            )),
        ]
    }
}

impl crate::tui::widgets::Widget for AccountWidget {
    fn id(&self) -> &'static str {
        "account"
    }

    fn name(&self) -> &'static str {
        "Account"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        let palette = app.theme.palette();
        let lines = Self::build_lines(&app.account, palette);

        let paragraph = Paragraph::new(lines).block(
            Block::default().borders(Borders::ALL).title(Line::from(Span::styled(
                " Account ",
                Style::default()
                    .fg(palette.title)
                    .add_modifier(Modifier::BOLD)
            )))
        );

        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::style::Color;

    use super::*;
    use crate::tui::{themes::Theme, widgets::Widget};

    fn make_account(id: f64, balance: &str, status: &str) -> AccountInfo {
        AccountInfo {
            account_id: id,
            balance:    balance.to_string(),
            status:     status.to_string()
        }
    }

    #[test]
    fn widget_id_is_account() {
        let widget = AccountWidget::new(true);
        assert_eq!(widget.id(), "account");
    }

    #[test]
    fn widget_name_is_account() {
        let widget = AccountWidget::new(true);
        assert_eq!(widget.name(), "Account");
    }

    #[test]
    fn widget_enabled_by_default() {
        let widget = AccountWidget::new(true);
        assert!(widget.enabled());
    }

    #[test]
    fn widget_disabled_when_not_enabled() {
        let widget = AccountWidget::new(false);
        assert!(!widget.enabled());
    }

    #[test]
    fn widget_toggle_enables() {
        let mut widget = AccountWidget::new(false);
        widget.toggle();
        assert!(widget.enabled());
    }

    #[test]
    fn widget_toggle_disables() {
        let mut widget = AccountWidget::new(true);
        widget.toggle();
        assert!(!widget.enabled());
    }

    #[test]
    fn format_account_id_integer() {
        assert_eq!(AccountWidget::format_account_id(12345.0), "12345");
    }

    #[test]
    fn format_account_id_zero() {
        assert_eq!(AccountWidget::format_account_id(0.0), "0");
    }

    #[test]
    fn format_account_id_negative() {
        assert_eq!(AccountWidget::format_account_id(-100.0), "-100");
    }

    #[test]
    fn format_account_id_float() {
        assert_eq!(AccountWidget::format_account_id(12345.67), "12345.67");
    }

    #[test]
    fn build_lines_active_status() {
        let account = make_account(12345.0, "1,234.56 RUB", "active");
        let palette = Theme::GruvboxDark.palette();
        let lines = AccountWidget::build_lines(&account, palette);

        assert_eq!(lines.len(), 3);
        assert!(
            lines[0]
                .spans
                .iter()
                .any(|s| s.content.contains("ID: 12345"))
        );
        assert!(
            lines[1]
                .spans
                .iter()
                .any(|s| s.content.contains("1,234.56 RUB"))
        );
        assert!(
            lines[2]
                .spans
                .iter()
                .any(|s| s.content.contains("Status: active"))
        );
    }

    #[test]
    fn build_lines_inactive_status_has_warning_color() {
        let account = make_account(100.0, "0.00 RUB", "inactive");
        let palette = Theme::GruvboxDark.palette();
        let lines = AccountWidget::build_lines(&account, palette);

        let status_line = &lines[2];
        let span = status_line.spans.first().expect("status span");
        assert!(matches!(span.style.fg, Some(Color::Rgb(249, 226, 175))));
    }

    #[test]
    fn build_lines_error_status_has_error_color() {
        let account = make_account(200.0, "0.00 RUB", "error");
        let palette = Theme::GruvboxDark.palette();
        let lines = AccountWidget::build_lines(&account, palette);

        let status_line = &lines[2];
        let span = status_line.spans.first().expect("status span");
        assert!(matches!(span.style.fg, Some(Color::Rgb(243, 113, 113))));
    }

    #[test]
    fn build_lines_default_status_has_fg_color() {
        let account = make_account(300.0, "0.00 RUB", "unknown");
        let palette = Theme::GruvboxDark.palette();
        let lines = AccountWidget::build_lines(&account, palette);

        let status_line = &lines[2];
        let span = status_line.spans.first().expect("status span");
        assert!(matches!(span.style.fg, Some(Color::Rgb(250, 241, 242))));
    }

    #[test]
    fn build_lines_empty_account() {
        let account = AccountInfo::default();
        let palette = Theme::GruvboxDark.palette();
        let lines = AccountWidget::build_lines(&account, palette);

        assert_eq!(lines.len(), 3);
        assert!(lines[0].spans.iter().any(|s| s.content.contains("ID: 0")));
        assert!(
            lines[1]
                .spans
                .iter()
                .any(|s| s.content.is_empty() || s.content.contains("Balance:"))
        );
        assert!(lines[2].spans.iter().any(|s| s.content.contains("Status:")));
    }

    #[test]
    fn widget_trait_object_is_send() {
        let widget: Box<dyn Widget + Send> = Box::new(AccountWidget::new(true));
        assert_eq!(widget.id(), "account");
    }
}
