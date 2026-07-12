// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! The settings panel model: the rows shown in the content pane and the
//! logic that cycles their values.

use std::borrow::Cow;

use rust_i18n::t;

use crate::config::Language;

/// One row of the settings panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingRow {
    /// Color theme, cycling through every built-in theme.
    Theme,
    /// Interface language.
    Language,
    /// The account header widget.
    WidgetAccount,
    /// The events log widget.
    WidgetEvents,
    /// Hiding services with zero resources from the sidebar.
    HideEmptySections,
    /// The active credentials profile.
    Profile
}

/// The rows of the settings panel, in display order.
pub const SETTING_ROWS: [SettingRow; 6] = [
    SettingRow::Theme,
    SettingRow::Language,
    SettingRow::WidgetAccount,
    SettingRow::WidgetEvents,
    SettingRow::HideEmptySections,
    SettingRow::Profile
];

impl SettingRow {
    /// The localized row label.
    #[must_use]
    pub fn label(self) -> Cow<'static, str> {
        match self {
            Self::Theme => t!("settings.theme"),
            Self::Language => t!("settings.language"),
            Self::WidgetAccount => t!("settings.widget_account"),
            Self::WidgetEvents => t!("settings.widget_events"),
            Self::HideEmptySections => t!("settings.hide_empty"),
            Self::Profile => t!("settings.profile")
        }
    }
}

impl super::App {
    /// The current display value of a settings row.
    #[must_use]
    pub fn setting_value(&self, row: SettingRow) -> String {
        let on_off = |enabled: bool| {
            if enabled {
                t!("settings.on").into_owned()
            } else {
                t!("settings.off").into_owned()
            }
        };
        match row {
            SettingRow::Theme => theme_name(self.theme).to_string(),
            SettingRow::Language => match self.language {
                Language::En => "English".to_string(),
                Language::Ru => "Русский".to_string()
            },
            SettingRow::WidgetAccount => on_off(self.is_widget_enabled("account")),
            SettingRow::WidgetEvents => on_off(self.is_widget_enabled("events")),
            SettingRow::HideEmptySections => on_off(self.hide_empty_tabs),
            SettingRow::Profile => self.active_profile.clone()
        }
    }

    /// Moves the settings selection one row up.
    pub const fn settings_up(&mut self) {
        if self.settings_selected > 0 {
            self.settings_selected -= 1;
        }
    }

    /// Moves the settings selection one row down.
    pub fn settings_down(&mut self) {
        if self.settings_selected + 1 < SETTING_ROWS.len() {
            self.settings_selected += 1;
        }
    }

    /// Cycles the selected setting's value forward or backward, applying it
    /// immediately and marking preferences dirty for persistence.
    pub fn settings_adjust(&mut self, forward: bool) {
        let row = SETTING_ROWS[self.settings_selected.min(SETTING_ROWS.len() - 1)];
        match row {
            SettingRow::Theme => {
                let all = crate::tui::themes::Theme::ALL;
                let pos = all.iter().position(|t| *t == self.theme).unwrap_or(0);
                let next = if forward {
                    (pos + 1) % all.len()
                } else {
                    (pos + all.len() - 1) % all.len()
                };
                self.theme = all[next];
                self.prefs_dirty = true;
            }
            SettingRow::Language => {
                let next = match self.language {
                    Language::En => Language::Ru,
                    Language::Ru => Language::En
                };
                self.set_language(next);
            }
            SettingRow::WidgetAccount => {
                self.widgets.toggle("account");
                self.prefs_dirty = true;
            }
            SettingRow::WidgetEvents => {
                self.widgets.toggle("events");
                self.prefs_dirty = true;
            }
            SettingRow::HideEmptySections => {
                self.toggle_hide_empty_tabs();
                if let Some(index) = self
                    .nav_items()
                    .iter()
                    .position(|i| matches!(i.kind, super::NavKind::Settings))
                {
                    self.nav_selected = index;
                }
            }
            SettingRow::Profile => {
                if self.profiles.len() > 1 {
                    let pos = self
                        .profiles
                        .iter()
                        .position(|p| *p == self.active_profile)
                        .unwrap_or(0);
                    let next = if forward {
                        (pos + 1) % self.profiles.len()
                    } else {
                        (pos + self.profiles.len() - 1) % self.profiles.len()
                    };
                    self.switch_profile = Some(self.profiles[next].clone());
                }
            }
        }
    }
}

/// Human-readable name of a theme.
#[must_use]
pub const fn theme_name(theme: crate::tui::themes::Theme) -> &'static str {
    match theme {
        crate::tui::themes::Theme::GruvboxDark => "Gruvbox Dark",
        crate::tui::themes::Theme::GruvboxLight => "Gruvbox Light",
        crate::tui::themes::Theme::CatppuccinMocha => "Catppuccin Mocha",
        crate::tui::themes::Theme::CatppuccinLatte => "Catppuccin Latte"
    }
}
