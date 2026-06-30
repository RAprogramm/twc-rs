// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

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
                    field: "Login".to_string(),
                    value: summary.login.clone()
                },
                AccountRow {
                    field: "Company".to_string(),
                    value: summary.company.clone()
                },
                AccountRow {
                    field: "Balance".to_string(),
                    value: format!("{} {}", summary.balance, summary.currency)
                },
                AccountRow {
                    field: "Hourly cost".to_string(),
                    value: format!("{:.2} {}", finances.hourly_cost, summary.currency)
                },
                AccountRow {
                    field: "Monthly cost".to_string(),
                    value: format!("{:.2} {}", finances.monthly_cost, summary.currency)
                },
                AccountRow {
                    field: "Blocked".to_string(),
                    value: summary.blocked.to_string()
                },
            ];
            let table = crate::output::render_table(&rows);
            println!("{table}");
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&summary)
                .map_err(|e| TwcError::Api(e.to_string()))?;
            println!("{json}");
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
