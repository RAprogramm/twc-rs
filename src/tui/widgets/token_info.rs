// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Token Info widget — displays JWT token expiration status.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph}
};
use rust_i18n::t;

use crate::{jwt::JwtPayload, tui::app::App};

/// Renders the JWT token expiration information panel.
///
/// # Overview
///
/// Displays the token expiration date, remaining days, and a color-coded
/// status indicator (green for valid, yellow for expiring soon, red for
/// expired). Shows "No token configured" when no token is set.
///
/// # Examples
///
/// ```ignore
/// use twc_rs::tui::widgets::token_info::TokenInfoWidget;
///
/// let widget = TokenInfoWidget::new(true);
/// assert_eq!(widget.id(), "token_info");
/// ```
pub struct TokenInfoWidget {
    enabled: bool
}

impl TokenInfoWidget {
    /// Creates a new token info widget with the given enabled state.
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

    /// Builds the styled lines for the token info panel.
    ///
    /// # Arguments
    ///
    /// * `token` - Optional token string to parse and display.
    /// * `palette` - The theme color palette.
    ///
    /// # Returns
    ///
    /// A vector of `Line` structs representing the rendered content.
    fn build_lines(
        token: Option<&str>,
        palette: crate::tui::themes::Palette
    ) -> Vec<Line<'static>> {
        let Some(token_str) = token else {
            return vec![Line::from(Span::styled(
                t!("token_info.no_token").to_string(),
                Style::default().fg(palette.dim)
            ))];
        };

        let payload = JwtPayload::parse(token_str);
        if payload.exp.is_none() {
            return vec![
                Line::from(Span::styled(
                    t!("token_info.api_key").to_string(),
                    Style::default().fg(palette.fg).add_modifier(Modifier::BOLD)
                )),
                Line::from(Span::styled(
                    t!("token_info.no_jwt_expiry").to_string(),
                    Style::default().fg(palette.dim)
                )),
            ];
        }
        let expires_line = payload.exp.map_or_else(
            || {
                Line::from(Span::styled(
                    t!("token_info.expires_unknown").to_string(),
                    Style::default().fg(palette.dim)
                ))
            },
            |exp| {
                Line::from(Span::styled(
                    t!(
                        "token_info.expires",
                        date => exp.format("%Y-%m-%d %H:%M:%S").to_string()
                    )
                    .to_string(),
                    Style::default().fg(palette.fg)
                ))
            }
        );

        let days_line = payload.days_remaining().map_or_else(
            || {
                Line::from(Span::styled(
                    t!("token_info.days_left_na").to_string(),
                    Style::default().fg(palette.dim)
                ))
            },
            |days| {
                Line::from(Span::styled(
                    t!("token_info.days_left", days => days).to_string(),
                    Style::default().fg(palette.fg)
                ))
            }
        );

        let status_style = if payload.is_expired() {
            Style::default().fg(palette.error)
        } else if payload.is_expiring_soon() {
            Style::default().fg(palette.warning)
        } else {
            Style::default().fg(palette.success)
        };

        let status_text = if payload.is_expired() {
            t!("token_info.status_expired").to_string()
        } else if payload.is_expiring_soon() {
            t!("token_info.status_expiring").to_string()
        } else {
            t!("token_info.status_valid").to_string()
        };

        let status_line = Line::from(Span::styled(
            format!("{}: \u{25cf} {status_text}", t!("token_info.status")),
            status_style
        ));

        vec![expires_line, days_line, status_line]
    }
}

impl crate::tui::widgets::Widget for TokenInfoWidget {
    fn id(&self) -> &'static str {
        "token_info"
    }

    fn name(&self) -> &'static str {
        "Token Info"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        let palette = app.theme.palette();
        let lines = Self::build_lines(app.token.as_deref(), palette);

        let paragraph = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(palette.border))
                .title(Line::from(Span::styled(
                    format!(" {} ", t!("token_info.title")),
                    Style::default()
                        .fg(palette.title)
                        .add_modifier(Modifier::BOLD)
                )))
        );

        frame.render_widget(paragraph, area);
    }
}
