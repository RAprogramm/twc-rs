// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Local installation health checks (duplicate binaries in `PATH`).

use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command
};

use rust_i18n::t;
use serde::Serialize;
use tabled::Tabled;

use crate::{error::TwcError, output::OutputFormat};

/// One discovered copy of the binary.
#[derive(Serialize)]
struct InstalledCopy {
    path:          String,
    version:       String,
    first_in_path: bool,
    running:       bool
}

/// Compact row for the doctor table.
#[derive(Tabled)]
struct CopyRow {
    #[tabled(rename = "Path")]
    path:    String,
    #[tabled(rename = "Version")]
    version: String,
    #[tabled(rename = "Active")]
    active:  String,
    #[tabled(rename = "Running")]
    running: String
}

/// True when the path points to an executable regular file.
fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

/// Distinct copies of `name` found in `path_var`, in `PATH` resolution order.
///
/// Entries resolving to the same canonical file (symlinked directories such
/// as `/bin` -> `/usr/bin`, duplicate `PATH` entries) are reported once, at
/// their first position.
fn scan_path(path_var: &OsStr, name: &str) -> Vec<PathBuf> {
    let file_name = format!("{name}{}", env::consts::EXE_SUFFIX);
    let mut seen: Vec<PathBuf> = Vec::new();
    let mut found: Vec<PathBuf> = Vec::new();
    for dir in env::split_paths(path_var) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        let candidate = dir.join(&file_name);
        if !is_executable(&candidate) {
            continue;
        }
        let canonical = candidate
            .canonicalize()
            .unwrap_or_else(|_| candidate.clone());
        if seen.contains(&canonical) {
            continue;
        }
        seen.push(canonical);
        found.push(candidate);
    }
    found
}

/// Version string reported by a binary, or a placeholder when it cannot run.
fn binary_version(path: &Path) -> String {
    Command::new(path)
        .arg("--version")
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| s.trim().to_owned())
        .unwrap_or_else(|| t!("cli.doctor_version_unknown").into_owned())
}

/// Checks the local installation for conflicting copies of the binary.
///
/// # Overview
///
/// Scans `PATH` for copies of `twc-rs`, reports each one with its version,
/// marks the copy that shells resolve first and the one currently running,
/// and warns when more than one distinct copy exists (a stale copy earlier
/// in `PATH` silently shadows package upgrades).
///
/// # Errors
///
/// Returns [`TwcError::Api`] when serialization of the report fails. Exits
/// the process with status 1 when a conflict is detected so scripts can gate
/// on it.
pub fn run(format: OutputFormat) -> Result<(), TwcError> {
    let path_var = env::var_os("PATH").unwrap_or_default();
    let copies = scan_path(&path_var, "twc-rs");
    let current = env::current_exe()
        .and_then(|p| p.canonicalize())
        .unwrap_or_default();

    let report: Vec<InstalledCopy> = copies
        .iter()
        .enumerate()
        .map(|(index, path)| InstalledCopy {
            path:          path.display().to_string(),
            version:       binary_version(path),
            first_in_path: index == 0,
            running:       path.canonicalize().map(|p| p == current).unwrap_or(false)
        })
        .collect();

    match format {
        OutputFormat::Table => {
            let rows: Vec<CopyRow> = report
                .iter()
                .map(|c| CopyRow {
                    path:    c.path.clone(),
                    version: c.version.clone(),
                    active:  if c.first_in_path { "yes" } else { "" }.to_owned(),
                    running: if c.running { "yes" } else { "" }.to_owned()
                })
                .collect();
            if rows.is_empty() {
                println!("{}", t!("cli.doctor_not_in_path"));
            } else {
                println!("{}", crate::output::render_table(&rows));
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &report)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for c in &report {
                println!("{}\t{}", c.path, c.version);
            }
        }
    }

    if report.len() > 1 {
        if matches!(format, OutputFormat::Table) {
            let active = report.first().map(|c| c.path.clone()).unwrap_or_default();
            eprintln!(
                "{}",
                t!("cli.doctor_conflict", count => report.len(), active => active)
            );
        }
        std::process::exit(1);
    }
    if matches!(format, OutputFormat::Table) {
        println!("{}", t!("cli.doctor_ok"));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
