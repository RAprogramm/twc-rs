// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

use rust_i18n::t;
use serde::Serialize;
use tabled::Tabled;
use timeweb_rs::apis::{account_api, configuration::Configuration, payments_api};

use crate::{error::TwcError, output::OutputFormat};

/// A single field/value row of account information.
#[derive(Tabled)]
struct AccountRow {
    #[tabled(rename = "Field")]
    field: String,
    #[tabled(rename = "Value")]
    value: String
}

impl fmt::Display for AccountRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.value)
    }
}

/// Machine-readable account summary for JSON output.
#[derive(Serialize)]
struct AccountSummary {
    login:    String,
    company:  String,
    balance:  String,
    currency: String,
    blocked:  bool
}

/// Machine-readable view of the account auth access restrictions.
#[derive(Serialize)]
struct AccessSummary {
    ip_restrictions_enabled:      bool,
    country_restrictions_enabled: bool,
    allowed_ips:                  Vec<String>,
    allowed_countries:            Vec<String>
}

/// Shows the current account: login, company, balance and status.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn show(config: &Configuration, format: OutputFormat) -> Result<(), TwcError> {
    let status = account_api::get_account_status(config).await?.status;
    let finances = payments_api::get_finances(config).await?.finances;

    let summary = AccountSummary {
        login:    status.login.clone().unwrap_or_default(),
        company:  status.company_info.name.clone(),
        balance:  format!("{:.2}", finances.balance),
        currency: finances.currency.clone(),
        blocked:  status.is_blocked
    };

    match format {
        OutputFormat::Table => {
            let rows = vec![
                AccountRow {
                    field: t!("cli.account_field_login").into_owned(),
                    value: summary.login.clone()
                },
                AccountRow {
                    field: t!("cli.account_field_company").into_owned(),
                    value: summary.company.clone()
                },
                AccountRow {
                    field: t!("cli.account_field_balance").into_owned(),
                    value: format!("{} {}", summary.balance, summary.currency)
                },
                AccountRow {
                    field: t!("cli.account_field_hourly_cost").into_owned(),
                    value: format!("{:.2} {}", finances.hourly_cost, summary.currency)
                },
                AccountRow {
                    field: t!("cli.account_field_monthly_cost").into_owned(),
                    value: format!("{:.2} {}", finances.monthly_cost, summary.currency)
                },
                AccountRow {
                    field: t!("cli.account_field_blocked").into_owned(),
                    value: summary.blocked.to_string()
                },
            ];
            let table = crate::output::render_table(&rows);
            println!("{table}");
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &summary)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            println!(
                "{}\t{} {}",
                summary.login, summary.balance, summary.currency
            );
        }
    }
    Ok(())
}

/// Joins a list of values for display, falling back to a placeholder when the
/// list is empty.
fn join_or_placeholder(values: &[String]) -> String {
    if values.is_empty() {
        t!("cli.account_access_none").into_owned()
    } else {
        values.join(", ")
    }
}

/// Shows the account auth access restrictions: whether IP and country
/// restrictions are enabled, and the allowed IPs and countries lists.
///
/// Mirrors the official `twc account access restrictions` view (read-only).
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn access(config: &Configuration, format: OutputFormat) -> Result<(), TwcError> {
    let settings = account_api::get_auth_access_settings(config).await?;

    let summary = AccessSummary {
        ip_restrictions_enabled:      settings.is_ip_restrictions_enabled,
        country_restrictions_enabled: settings.is_country_restrictions_enabled,
        allowed_ips:                  settings.white_list.ips.clone(),
        allowed_countries:            settings.white_list.countries.clone()
    };

    match format {
        OutputFormat::Table => {
            let rows = vec![
                AccountRow {
                    field: t!("cli.account_access_ip_restrictions").into_owned(),
                    value: summary.ip_restrictions_enabled.to_string()
                },
                AccountRow {
                    field: t!("cli.account_access_country_restrictions").into_owned(),
                    value: summary.country_restrictions_enabled.to_string()
                },
                AccountRow {
                    field: t!("cli.account_access_allowed_ips").into_owned(),
                    value: join_or_placeholder(&summary.allowed_ips)
                },
                AccountRow {
                    field: t!("cli.account_access_allowed_countries").into_owned(),
                    value: join_or_placeholder(&summary.allowed_countries)
                },
            ];
            let table = crate::output::render_table(&rows);
            println!("{table}");
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &summary)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            println!(
                "{}\t{}",
                summary.ip_restrictions_enabled, summary.country_restrictions_enabled
            );
        }
    }
    Ok(())
}
