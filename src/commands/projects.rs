// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

use tabled::Tabled;
use timeweb_rs::{
    apis::{configuration::Configuration, projects_api},
    models::CreateProject
};

use crate::{error::TwcError, output::OutputFormat};

/// Formats a float identifier for display.
fn fmt_id(v: f64) -> String {
    format!("{v:.0}")
}

/// Compact row for the project list table.
#[derive(Tabled)]
struct ProjectRow {
    #[tabled(rename = "ID")]
    id:          String,
    #[tabled(rename = "Name")]
    name:        String,
    #[tabled(rename = "Description")]
    description: String,
    #[tabled(rename = "Default")]
    is_default:  String
}

impl fmt::Display for ProjectRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.id, self.name, self.description, self.is_default
        )
    }
}

/// Lists all projects.
///
/// # Overview
///
/// Fetches all projects from the Timeweb Cloud API and displays
/// them in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list(config: &Configuration, format: OutputFormat) -> Result<(), TwcError> {
    let resp = projects_api::get_projects(config).await?;

    let rows: Vec<ProjectRow> = resp
        .projects
        .iter()
        .map(|p| ProjectRow {
            id:          fmt_id(p.id),
            name:        p.name.clone(),
            description: p.description.clone(),
            is_default:  p.is_default.to_string()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("No projects found.");
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out =
                crate::output::serialized(format, &resp.projects).expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for p in &resp.projects {
                println!("{}\t{}", fmt_id(p.id), p.name);
            }
        }
    }
    Ok(())
}

/// Creates a new project.
///
/// # Overview
///
/// Creates a project with the given name and optional description.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn create(
    config: &Configuration,
    name: &str,
    description: Option<&str>
) -> Result<(), TwcError> {
    let mut req = CreateProject::new(name.to_string());
    if let Some(desc) = description {
        req.description = Some(Some(desc.to_string()));
    }
    let resp = projects_api::create_project(config, req).await?;
    println!(
        "Project '{}' created (id: {}).",
        resp.project.name,
        fmt_id(resp.project.id)
    );
    Ok(())
}

/// Deletes a project by ID.
///
/// # Overview
///
/// Sends a delete request for the specified project.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn delete(config: &Configuration, id: i32) -> Result<(), TwcError> {
    projects_api::delete_project(config, id).await?;
    println!("Project {id} deleted.");
    Ok(())
}

#[cfg(test)]
mod tests;
