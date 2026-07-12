// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! In-dashboard resource creation forms.

use rust_i18n::t;

use super::ResourceTab;

/// A single editable field in a create form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputField {
    /// Field label shown to the left of the input.
    pub label:    String,
    /// Current value typed by the user.
    pub value:    String,
    /// Whether the field must be non-empty to submit.
    pub required: bool
}

/// An in-dashboard form for creating a new resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateForm {
    /// The resource category being created.
    pub tab:    ResourceTab,
    /// Form title.
    pub title:  String,
    /// Editable fields in tab order.
    pub fields: Vec<InputField>,
    /// Index of the focused field.
    pub active: usize
}

impl super::App {
    /// Returns true when the create form overlay is open.
    #[must_use]
    pub const fn create_form_open(&self) -> bool {
        self.create_form.is_some()
    }

    /// Opens a create form for the active tab, when that resource supports
    /// in-dashboard creation. Resources whose creation needs many or
    /// structured fields stay CLI-only.
    pub fn open_create_form(&mut self) {
        let form = match self.active_tab {
            ResourceTab::Projects => Some(CreateForm {
                tab:    ResourceTab::Projects,
                title:  "Create project".to_string(),
                fields: vec![
                    InputField {
                        label:    "Name".to_string(),
                        value:    String::new(),
                        required: true
                    },
                    InputField {
                        label:    "Description".to_string(),
                        value:    String::new(),
                        required: false
                    },
                ],
                active: 0
            }),
            _ => None
        };
        if let Some(form) = form {
            self.create_form = Some(form);
        } else {
            self.status_message = Some(t!("app.create_not_supported").to_string());
        }
    }

    /// Closes the create form without submitting.
    pub fn close_create_form(&mut self) {
        self.create_form = None;
    }

    /// Moves focus to the next form field.
    pub const fn form_next_field(&mut self) {
        if let Some(form) = &mut self.create_form
            && !form.fields.is_empty()
        {
            form.active = (form.active + 1) % form.fields.len();
        }
    }

    /// Moves focus to the previous form field.
    pub const fn form_prev_field(&mut self) {
        if let Some(form) = &mut self.create_form
            && !form.fields.is_empty()
        {
            let len = form.fields.len();
            form.active = (form.active + len - 1) % len;
        }
    }

    /// Appends a character to the focused field.
    pub fn form_input(&mut self, c: char) {
        if let Some(form) = &mut self.create_form
            && let Some(field) = form.fields.get_mut(form.active)
        {
            field.value.push(c);
        }
    }

    /// Removes the last character from the focused field.
    pub fn form_backspace(&mut self) {
        if let Some(form) = &mut self.create_form
            && let Some(field) = form.fields.get_mut(form.active)
        {
            field.value.pop();
        }
    }

    /// Submits the form when all required fields are filled, queuing it for the
    /// dashboard loop to perform the create. Returns false when a required
    /// field is empty (the form stays open).
    pub fn form_submit(&mut self) -> bool {
        let Some(form) = &self.create_form else {
            return false;
        };
        if form
            .fields
            .iter()
            .any(|f| f.required && f.value.trim().is_empty())
        {
            return false;
        }
        self.create_request = self.create_form.take();
        true
    }

    /// Takes a submitted create request for the dashboard loop to perform.
    pub const fn take_create_request(&mut self) -> Option<CreateForm> {
        self.create_request.take()
    }
}
