// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Resource actions: the action vocabulary and the pending-action state.

use std::borrow::Cow;

use rust_i18n::t;

use super::ResourceTab;

/// A kind of action that can be performed on a resource.
///
/// Each [`ResourceTab`] declares which kinds apply to it via
/// [`ResourceTab::actions`]; the dashboard loop maps a chosen
/// `(tab, kind)` pair to the matching Timeweb API call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    /// Power the resource on.
    Start,
    /// Gracefully power the resource off.
    Shutdown,
    /// Reboot the resource.
    Reboot,
    /// Create a clone of the resource.
    Clone,
    /// Create a backup of the resource.
    Backup,
    /// Permanently delete the resource.
    Delete
}

impl ActionKind {
    /// Returns the label shown in the action menu and confirmation prompt.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Start => "Start",
            Self::Shutdown => "Shutdown",
            Self::Reboot => "Reboot",
            Self::Clone => "Clone",
            Self::Backup => "Backup",
            Self::Delete => "Delete"
        }
    }

    /// Returns the localized label shown in menus and confirmation prompts.
    #[must_use]
    pub fn display_label(self) -> Cow<'static, str> {
        match self {
            Self::Start => t!("app.action_start"),
            Self::Shutdown => t!("app.action_shutdown"),
            Self::Reboot => t!("app.action_reboot"),
            Self::Clone => t!("app.action_clone"),
            Self::Backup => t!("app.action_backup"),
            Self::Delete => t!("app.action_delete")
        }
    }

    /// Returns true when the action is destructive and irreversible.
    ///
    /// Destructive actions require an extra confirmation step.
    #[must_use]
    pub const fn is_destructive(self) -> bool {
        matches!(self, Self::Delete)
    }
}

/// A resource action awaiting confirmation, or ready to be dispatched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingAction {
    /// The resource category the action targets.
    pub tab:           ResourceTab,
    /// The action to perform.
    pub kind:          ActionKind,
    /// Target resource id (numeric or UUID, depending on the resource).
    pub resource_id:   String,
    /// Target resource name, for display.
    pub resource_name: String
}

impl super::App {
    /// Confirms the pending destructive action, queueing it for dispatch.
    pub fn confirm_action(&mut self) {
        self.dispatch = self.confirm.take();
    }

    /// Dismisses the pending action without dispatching it.
    pub fn cancel_action(&mut self) {
        self.confirm = None;
    }

    /// Returns the action awaiting confirmation, for rendering the modal.
    #[must_use]
    pub const fn pending_action(&self) -> Option<&PendingAction> {
        self.confirm.as_ref()
    }

    /// Returns true while a confirmation modal is open.
    #[must_use]
    pub const fn awaiting_confirm(&self) -> bool {
        self.confirm.is_some()
    }

    /// Takes the action queued for dispatch, if the user confirmed one.
    #[must_use]
    pub const fn take_dispatch(&mut self) -> Option<PendingAction> {
        self.dispatch.take()
    }
}
