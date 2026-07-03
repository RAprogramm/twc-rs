// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Command palette state and the commands it offers.

use rust_i18n::t;

use super::PendingAction;

impl super::App {
    /// Returns true while the command palette is open.
    #[must_use]
    pub const fn palette_open(&self) -> bool {
        self.palette.is_some()
    }

    /// Opens the command palette, populated for the current context.
    pub fn open_palette(&mut self) {
        let commands = self.build_palette_commands();
        self.palette = Some(crate::tui::command_palette::CommandPalette::new(commands));
    }

    /// Opens the command palette restricted to profile-switch entries, or
    /// reports that no other profiles are configured.
    pub fn open_profile_switcher(&mut self) {
        let commands: Vec<_> = self
            .build_palette_commands()
            .into_iter()
            .filter(|c| c.id.starts_with("profile:"))
            .collect();
        if commands.is_empty() {
            self.status_message = Some(t!("app.no_other_profiles").to_string());
            return;
        }
        self.palette = Some(crate::tui::command_palette::CommandPalette::new(commands));
    }

    /// Closes the command palette.
    pub fn close_palette(&mut self) {
        self.palette = None;
    }

    /// Feeds a character to the open palette query.
    pub fn palette_input(&mut self, c: char) {
        if let Some(p) = self.palette.as_mut() {
            p.push_char(c);
        }
    }

    /// Deletes the last palette query character.
    pub fn palette_backspace(&mut self) {
        if let Some(p) = self.palette.as_mut() {
            p.backspace();
        }
    }

    /// Moves the palette selection down.
    pub const fn palette_next(&mut self) {
        if let Some(p) = self.palette.as_mut() {
            p.next();
        }
    }

    /// Moves the palette selection up.
    pub const fn palette_previous(&mut self) {
        if let Some(p) = self.palette.as_mut() {
            p.previous();
        }
    }

    /// Runs the highlighted palette command, then closes the palette.
    pub fn palette_run_selected(&mut self) {
        let id = self
            .palette
            .as_ref()
            .and_then(|p| p.selected_command())
            .map(|c| c.id.clone());
        if let Some(id) = id {
            self.run_command(&id);
        }
        self.close_palette();
    }

    fn build_palette_commands(&self) -> Vec<crate::tui::command_palette::Command> {
        use crate::tui::command_palette::Command;
        let mut commands = Vec::new();

        let general = t!("app.hint_general").to_string();
        for (id, title) in [
            ("app:create", t!("palette.cmd_create")),
            ("app:filter", t!("palette.cmd_filter")),
            ("app:refresh", t!("palette.cmd_refresh")),
            ("app:help", t!("palette.cmd_help")),
            ("app:quit", t!("palette.cmd_quit"))
        ] {
            commands.push(Command {
                id:    id.to_string(),
                title: title.to_string(),
                hint:  general.clone()
            });
        }

        if let Some((_, name)) = self.selected_resource() {
            for action in self.active_tab.actions() {
                commands.push(Command {
                    id:    format!("action:{}", action.label().to_lowercase()),
                    title: format!("{} {name}", action.display_label()),
                    hint:  t!("app.hint_action").to_string()
                });
            }
        }

        for (id, _) in Self::TOGGLEABLE_WIDGETS {
            let verb = if self.is_widget_enabled(id) {
                t!("app.palette_hide")
            } else {
                t!("app.palette_show")
            };
            commands.push(Command {
                id:    format!("widget:{id}"),
                title: format!("{verb} {}", Self::widget_display_label(id)),
                hint:  t!("app.hint_layout").to_string()
            });
        }

        for theme in crate::tui::themes::Theme::ALL {
            commands.push(Command {
                id:    format!("theme:{}", theme.id()),
                title: format!("Theme: {}", theme.label()),
                hint:  t!("app.hint_theme").to_string()
            });
        }

        commands.push(Command {
            id:    "tabs:toggle_empty".to_string(),
            title: if self.hide_empty_tabs {
                t!("app.palette_show_empty_tabs").to_string()
            } else {
                t!("app.palette_hide_empty_tabs").to_string()
            },
            hint:  t!("app.hint_layout").to_string()
        });

        commands.push(Command {
            id:    "lang:en".to_string(),
            title: rust_i18n::t!("palette.language_english").to_string(),
            hint:  t!("app.hint_language").to_string()
        });
        commands.push(Command {
            id:    "lang:ru".to_string(),
            title: rust_i18n::t!("palette.language_russian").to_string(),
            hint:  t!("app.hint_language").to_string()
        });

        for profile in &self.profiles {
            if *profile == self.active_profile {
                continue;
            }
            commands.push(Command {
                id:    format!("profile:{profile}"),
                title: t!("palette.switch_profile", name => profile).to_string(),
                hint:  t!("app.hint_account").to_string()
            });
        }

        commands
    }

    pub(super) fn run_command(&mut self, id: &str) {
        if id == "app:help" {
            self.toggle_help();
        } else if id == "app:refresh" {
            self.force_refresh();
        } else if id == "app:filter" {
            self.start_filter();
        } else if id == "app:create" {
            self.open_create_form();
        } else if id == "app:quit" {
            self.quit();
        } else if id == "tabs:toggle_empty" {
            self.toggle_hide_empty_tabs();
        } else if id == "lang:en" {
            self.set_language(crate::config::Language::En);
        } else if id == "lang:ru" {
            self.set_language(crate::config::Language::Ru);
        } else if let Some(profile) = id.strip_prefix("profile:") {
            self.switch_profile = Some(profile.to_string());
        } else if let Some(rest) = id.strip_prefix("theme:") {
            if let Some(theme) = crate::tui::themes::Theme::ALL
                .into_iter()
                .find(|t| t.id() == rest)
            {
                self.set_theme(theme);
            }
        } else if let Some(widget_id) = id.strip_prefix("widget:") {
            self.toggle_widget(widget_id);
        } else if let Some(action_label) = id.strip_prefix("action:")
            && let Some((resource_id, resource_name)) = self.selected_resource()
            && let Some(&kind) = self
                .active_tab
                .actions()
                .iter()
                .find(|a| a.label().eq_ignore_ascii_case(action_label))
        {
            let pending = PendingAction {
                tab: self.active_tab,
                kind,
                resource_id,
                resource_name
            };
            if kind.is_destructive() {
                self.confirm = Some(pending);
            } else {
                self.dispatch = Some(pending);
            }
        }
    }
}
