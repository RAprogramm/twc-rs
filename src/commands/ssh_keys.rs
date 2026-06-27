// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::{fmt, fs, io::Read as _};

use tabled::{Table, Tabled};
use timeweb_rs::{
    apis::{configuration::Configuration, ssh_api},
    models::CreateKeyRequest
};

use crate::{error::TwcError, output::OutputFormat};

/// Formats a float identifier for display.
fn fmt_id(v: f64) -> String {
    format!("{v:.0}")
}

/// Compact row for the SSH key list table.
#[derive(Tabled)]
struct SshKeyRow {
    #[tabled(rename = "ID")]
    id:          String,
    #[tabled(rename = "Name")]
    name:        String,
    #[tabled(rename = "Fingerprint")]
    fingerprint: String,
    #[tabled(rename = "Default")]
    is_default:  String
}

impl fmt::Display for SshKeyRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.id, self.name, self.fingerprint, self.is_default
        )
    }
}

/// Computes a human-readable fingerprint from an SSH public key body.
fn fingerprint(body: &str) -> String {
    let fp = body.split_whitespace().nth(1);
    fp.map_or_else(
        || "-".to_string(),
        |f| {
            let bytes: Vec<u8> = f
                .as_bytes()
                .chunks(2)
                .filter_map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap_or(""), 16).ok())
                .collect();
            if bytes.len() >= 16 {
                let hex: Vec<String> = bytes[..16].iter().map(|b| format!("{b:02x}")).collect();
                hex.join(":")
            } else {
                f.to_string()
            }
        }
    )
}

/// Lists all SSH keys.
///
/// # Overview
///
/// Fetches all SSH keys from the Timeweb Cloud API and displays
/// them in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list(config: &Configuration, format: OutputFormat) -> Result<(), TwcError> {
    let resp = ssh_api::get_keys(config).await?;

    let rows: Vec<SshKeyRow> = resp
        .ssh_keys
        .iter()
        .map(|k| SshKeyRow {
            id:          fmt_id(k.id),
            name:        k.name.clone(),
            fingerprint: fingerprint(&k.body),
            is_default:  k
                .is_default
                .map_or_else(|| "-".to_string(), |d| d.to_string())
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("No SSH keys found.");
            } else {
                let table = Table::new(&rows).to_string();
                println!("{table}");
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&resp.ssh_keys)
                .map_err(|e| TwcError::Api(e.to_string()))?;
            println!("{json}");
        }
        OutputFormat::Quiet => {
            for k in &resp.ssh_keys {
                println!("{}\t{}", fmt_id(k.id), k.name);
            }
        }
    }
    Ok(())
}

/// Adds an SSH key.
///
/// # Overview
///
/// Reads a public key from a file or stdin and uploads it
/// to the Timeweb Cloud API.
///
/// # Errors
///
/// Returns [`TwcError::Io`] on file read failure or
/// [`TwcError::Api`] on API failure.
pub async fn add(
    config: &Configuration,
    name: &str,
    file_path: Option<&str>,
    is_default: bool
) -> Result<(), TwcError> {
    let body = if let Some(path) = file_path {
        fs::read_to_string(path)
            .map_err(|e| TwcError::Io(format!("failed to read {path}: {e}")))?
    } else {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| TwcError::Io(format!("failed to read stdin: {e}")))?;
        buf
    };

    let body = body.trim().to_string();
    if body.is_empty() {
        return Err(TwcError::Io("SSH key body is empty".to_string()));
    }

    let req = CreateKeyRequest::new(body, is_default, name.to_string());
    let resp = ssh_api::create_key(config, req).await?;
    println!(
        "SSH key '{}' created (id: {}).",
        resp.ssh_key.name,
        fmt_id(resp.ssh_key.id)
    );
    Ok(())
}

/// Deletes an SSH key by ID.
///
/// # Overview
///
/// Sends a delete request for the specified SSH key.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn delete(config: &Configuration, id: i32) -> Result<(), TwcError> {
    ssh_api::delete_key(config, id).await?;
    println!("SSH key {id} deleted.");
    Ok(())
}
