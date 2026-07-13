// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! The settings panel model: setting cards navigated like every other card
//! grid, toggled with Enter, with a picker popup for multi-value settings.

use std::borrow::Cow;

use rust_i18n::t;

use crate::config::Language;

/// One setting card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingRow {
    /// Color theme; Enter opens a picker.
    Theme,
    /// Interface language; Enter opens a picker.
    Language,
    /// The account header widget; Enter toggles.
    WidgetAccount,
    /// The events log widget; Enter toggles.
    WidgetEvents,
    /// Hiding zero-count services from the sidebar; Enter toggles.
    HideEmptySections,
    /// The active credentials profile; Enter opens a picker.
    Profile
}

/// The setting cards, in display order.
pub const SETTING_ROWS: [SettingRow; 6] = [
    SettingRow::Theme,
    SettingRow::Language,
    SettingRow::WidgetAccount,
    SettingRow::WidgetEvents,
    SettingRow::HideEmptySections,
    SettingRow::Profile
];

/// A picker popup for a multi-value setting.
#[derive(Debug, Clone)]
pub struct SettingsPicker {
    /// The setting being changed.
    pub row:      SettingRow,
    /// Popup title.
    pub title:    String,
    /// The selectable values.
    pub options:  Vec<String>,
    /// Index of the highlighted option.
    pub selected: usize
}

impl SettingRow {
    /// The localized card label.
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

    /// The Nerd Font glyph shown on the card.
    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Theme => "\u{f1fc}",
            Self::Language => "\u{f0ac}",
            Self::WidgetAccount => "\u{f2bb}",
            Self::WidgetEvents => "\u{f03a}",
            Self::HideEmptySections => "\u{f070}",
            Self::Profile => "\u{f2c0}"
        }
    }
}

impl super::App {
    /// The current display value of a setting card.
    #[must_use]
    pub fn setting_value(&self, row: SettingRow) -> String {
        match row {
            SettingRow::Theme => theme_name(self.theme).to_string(),
            SettingRow::Language => language_name(self.language).to_string(),
            SettingRow::WidgetAccount
            | SettingRow::WidgetEvents
            | SettingRow::HideEmptySections => {
                if self.setting_enabled(row).unwrap_or(false) {
                    t!("settings.on").into_owned()
                } else {
                    t!("settings.off").into_owned()
                }
            }
            SettingRow::Profile => self.active_profile.clone()
        }
    }

    /// Whether a toggle setting is on; `None` for value settings.
    #[must_use]
    pub fn setting_enabled(&self, row: SettingRow) -> Option<bool> {
        match row {
            SettingRow::WidgetAccount => Some(self.is_widget_enabled("account")),
            SettingRow::WidgetEvents => Some(self.is_widget_enabled("events")),
            SettingRow::HideEmptySections => Some(self.hide_empty_tabs),
            SettingRow::Theme | SettingRow::Language | SettingRow::Profile => None
        }
    }

    /// Moves the settings-card selection one step on the grid, clamping at
    /// every edge exactly like the resource grids.
    pub fn settings_move(&mut self, dir: super::FocusDir) {
        self.settings_selected = super::grid_step(
            self.settings_selected,
            SETTING_ROWS.len(),
            self.resource_cols,
            dir
        );
    }

    /// Activates the selected setting card: toggles a switch in place, or
    /// opens the picker popup for multi-value settings.
    pub fn settings_activate(&mut self) {
        let row = SETTING_ROWS[self.settings_selected.min(SETTING_ROWS.len() - 1)];
        match row {
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
            SettingRow::Theme => {
                let options: Vec<String> = crate::tui::themes::Theme::ALL
                    .iter()
                    .map(|t| theme_name(*t).to_string())
                    .collect();
                let selected = crate::tui::themes::Theme::ALL
                    .iter()
                    .position(|t| *t == self.theme)
                    .unwrap_or(0);
                self.picker = Some(SettingsPicker {
                    row,
                    title: row.label().into_owned(),
                    options,
                    selected
                });
            }
            SettingRow::Language => {
                let selected = usize::from(self.language == Language::Ru);
                self.picker = Some(SettingsPicker {
                    row,
                    title: row.label().into_owned(),
                    options: vec!["English".to_string(), "Русский".to_string()],
                    selected
                });
            }
            SettingRow::Profile => {
                if self.profiles.len() < 2 {
                    self.status_message = Some(t!("app.no_other_profiles").to_string());
                    return;
                }
                let selected = self
                    .profiles
                    .iter()
                    .position(|p| *p == self.active_profile)
                    .unwrap_or(0);
                self.picker = Some(SettingsPicker {
                    row,
                    title: row.label().into_owned(),
                    options: self.profiles.clone(),
                    selected
                });
            }
        }
    }

    /// Returns true while the settings picker popup is open.
    #[must_use]
    pub const fn picker_open(&self) -> bool {
        self.picker.is_some()
    }

    /// Moves the picker highlight down.
    pub fn picker_next(&mut self) {
        if let Some(p) = self.picker.as_mut()
            && p.selected + 1 < p.options.len()
        {
            p.selected += 1;
        }
    }

    /// Moves the picker highlight up, clamping at the first option.
    pub fn picker_previous(&mut self) {
        if let Some(p) = self.picker.as_mut() {
            p.selected = p.selected.saturating_sub(1);
        }
    }

    /// Closes the picker without applying.
    pub fn picker_close(&mut self) {
        self.picker = None;
    }

    /// Applies the highlighted picker option and closes the popup.
    pub fn picker_apply(&mut self) {
        let Some(picker) = self.picker.take() else {
            return;
        };
        match picker.row {
            SettingRow::Theme => {
                if let Some(theme) = crate::tui::themes::Theme::ALL.get(picker.selected) {
                    self.theme = *theme;
                    self.prefs_dirty = true;
                }
            }
            SettingRow::Language => {
                let next = if picker.selected == 1 {
                    Language::Ru
                } else {
                    Language::En
                };
                self.set_language(next);
            }
            SettingRow::Profile => {
                if let Some(profile) = picker.options.get(picker.selected)
                    && *profile != self.active_profile
                {
                    self.switch_profile = Some(profile.clone());
                }
            }
            SettingRow::WidgetAccount
            | SettingRow::WidgetEvents
            | SettingRow::HideEmptySections => {}
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

/// Human-readable name of a language.
#[must_use]
pub const fn language_name(language: Language) -> &'static str {
    match language {
        Language::En => "English",
        Language::Ru => "Русский"
    }
}
