// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

use rust_i18n::t;
use tabled::Tabled;
use timeweb_rs::apis::{apps_api, configuration::Configuration};

use crate::{error::TwcError, output::OutputFormat};

/// Formats a float identifier for display.
fn fmt_id<T: std::fmt::Display>(v: T) -> String {
    v.to_string()
}

/// Compact row for the app list table.
#[derive(Tabled)]
struct AppRow {
    #[tabled(rename = "ID")]
    id:       String,
    #[tabled(rename = "Name")]
    name:     String,
    #[tabled(rename = "Status")]
    status:   String,
    #[tabled(rename = "IP")]
    ip:       String,
    #[tabled(rename = "Location")]
    location: String
}

impl fmt::Display for AppRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.id, self.name, self.status, self.ip, self.location
        )
    }
}

/// Lists all apps.
///
/// # Overview
///
/// Fetches all apps from the Timeweb Cloud API and displays
/// them in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list(config: &Configuration, format: OutputFormat) -> Result<(), TwcError> {
    let resp = apps_api::get_apps(config).await?;

    let rows: Vec<AppRow> = resp
        .apps
        .iter()
        .map(|a| AppRow {
            id:       fmt_id(a.id),
            name:     a.name.clone(),
            status:   format!("{:?}", a.status),
            ip:       a.ip.clone().unwrap_or_default(),
            location: a.location.clone().unwrap_or_default()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_apps"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.apps)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for a in &resp.apps {
                println!("{}\t{}", fmt_id(a.id), a.name);
            }
        }
    }
    Ok(())
}
