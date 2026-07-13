// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! In-dashboard resource creation forms.

use std::borrow::Cow;

use rust_i18n::t;

use super::ResourceTab;

/// Prefilled subnet for a new private network.
const DEFAULT_SUBNET_V4: &str = "192.168.0.0/24";

/// Prefilled location code for new resources.
const DEFAULT_LOCATION: &str = "ru-1";

/// Prefilled size, in gigabytes, for a new network drive.
const DEFAULT_DRIVE_SIZE_GB: &str = "10";

/// Prefilled drive type for a new network drive.
const DEFAULT_DRIVE_TYPE: &str = "nvme";

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

impl InputField {
    /// An empty field that must be filled before submit.
    fn required(label: Cow<'_, str>) -> Self {
        Self {
            label:    label.into_owned(),
            value:    String::new(),
            required: true
        }
    }

    /// An empty field that may stay blank.
    fn optional(label: Cow<'_, str>) -> Self {
        Self {
            label:    label.into_owned(),
            value:    String::new(),
            required: false
        }
    }

    /// A required field prefilled with an editable default.
    fn prefilled(label: Cow<'_, str>, value: &str) -> Self {
        Self {
            label:    label.into_owned(),
            value:    value.to_string(),
            required: true
        }
    }
}

/// Parses a create-form size field as a positive number of gigabytes.
///
/// # Errors
///
/// Returns a localized message when the value is not a positive finite
/// number.
pub fn parse_size_gb(value: &str) -> Result<f64, String> {
    let trimmed = value.trim();
    match trimmed.parse::<f64>() {
        Ok(size) if size > 0.0 && size.is_finite() => Ok(size),
        _ => Err(t!("form.invalid_size", value => trimmed).into_owned())
    }
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

    /// Builds the create form for `tab`, or `None` when that resource has no
    /// in-dashboard form. Resources whose creation needs many or structured
    /// fields stay CLI-only.
    fn build_create_form(tab: ResourceTab) -> Option<CreateForm> {
        let fields = match tab {
            ResourceTab::Projects => vec![
                InputField::required(t!("form.field.name")),
                InputField::optional(t!("form.field.description")),
            ],
            ResourceTab::SshKeys => vec![
                InputField::required(t!("form.field.name")),
                InputField::required(t!("form.field.public_key")),
            ],
            ResourceTab::Vpc => vec![
                InputField::required(t!("form.field.name")),
                InputField::prefilled(t!("form.field.subnet"), DEFAULT_SUBNET_V4),
                InputField::prefilled(t!("form.field.location"), DEFAULT_LOCATION),
            ],
            ResourceTab::NetworkDrives => vec![
                InputField::required(t!("form.field.name")),
                InputField::prefilled(t!("form.field.size_gb"), DEFAULT_DRIVE_SIZE_GB),
                InputField::prefilled(t!("form.field.drive_type"), DEFAULT_DRIVE_TYPE),
            ],
            _ => return None
        };
        Some(CreateForm {
            tab,
            title: crate::tui::widgets::service_header::texts(tab)
                .1
                .into_owned(),
            fields,
            active: 0
        })
    }

    /// Returns true when `tab` supports in-dashboard creation.
    #[must_use]
    pub fn tab_has_create_form(tab: ResourceTab) -> bool {
        Self::build_create_form(tab).is_some()
    }

    /// Opens a create form for the active tab, when that resource supports
    /// in-dashboard creation.
    pub fn open_create_form(&mut self) {
        if let Some(form) = Self::build_create_form(self.active_tab) {
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
