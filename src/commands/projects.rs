// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

use rust_i18n::t;
use tabled::Tabled;
use timeweb_rs::{
    apis::{configuration::Configuration, projects_api},
    models::{CreateProject, UpdateProject}
};

use crate::{error::TwcError, output::OutputFormat};

/// Formats a float identifier for display.
fn fmt_id<T: std::fmt::Display>(v: T) -> String {
    v.to_string()
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

/// Compact row for the project resources table.
#[derive(Tabled)]
struct ResourceRow {
    #[tabled(rename = "Kind")]
    kind:   String,
    #[tabled(rename = "ID")]
    id:     String,
    #[tabled(rename = "Name")]
    name:   String,
    #[tabled(rename = "Status")]
    status: String
}

impl fmt::Display for ResourceRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {} {}", self.kind, self.id, self.name, self.status)
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
                println!("{}", t!("cli.no_projects_found"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.projects)
                .transpose()?
                .unwrap_or_default();
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
        "{}",
        t!("cli.project_created", name => resp.project.name, id => fmt_id(resp.project.id))
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
    println!("{}", t!("cli.project_deleted", id => id));
    Ok(())
}

/// Updates a project's metadata.
///
/// # Overview
///
/// Sends a partial update for the specified project. Only the name and
/// description are changed when provided; omitted fields are left as-is.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn set(
    config: &Configuration,
    id: i32,
    name: Option<&str>,
    description: Option<&str>,
    format: OutputFormat
) -> Result<(), TwcError> {
    let mut req = UpdateProject::new();
    req.name = name.map(String::from);
    if let Some(desc) = description {
        req.description = Some(Some(desc.to_string()));
    }

    let resp = projects_api::update_project(config, id, req).await?;
    let project = &resp.project;

    match format {
        OutputFormat::Table => {
            println!(
                "{}",
                t!("cli.project_updated", name => project.name, id => fmt_id(project.id))
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, project)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", fmt_id(project.id), project.name);
        }
    }
    Ok(())
}

/// Lists all resources belonging to a project.
///
/// # Overview
///
/// Fetches every resource attached to the project and flattens the
/// per-type vectors (servers, databases, buckets, clusters, balancers,
/// dedicated servers) into a single table with a `Kind` column.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn resources(
    config: &Configuration,
    id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = projects_api::get_all_project_resources(config, id).await?;

    let mut rows: Vec<ResourceRow> = Vec::new();
    for s in &resp.servers {
        rows.push(ResourceRow {
            kind:   "Server".to_string(),
            id:     fmt_id(s.id),
            name:   s.name.clone(),
            status: format!("{:?}", s.status)
        });
    }
    for d in &resp.databases {
        rows.push(ResourceRow {
            kind:   "Database".to_string(),
            id:     fmt_id(d.id),
            name:   d.name.clone(),
            status: format!("{:?}", d.status)
        });
    }
    for b in &resp.buckets {
        rows.push(ResourceRow {
            kind:   "S3 bucket".to_string(),
            id:     fmt_id(b.id),
            name:   b.name.clone(),
            status: format!("{:?}", b.status)
        });
    }
    for c in &resp.clusters {
        rows.push(ResourceRow {
            kind:   "Kubernetes".to_string(),
            id:     fmt_id(c.id),
            name:   c.name.clone(),
            status: format!("{:?}", c.status)
        });
    }
    for b in &resp.balancers {
        rows.push(ResourceRow {
            kind:   "Balancer".to_string(),
            id:     fmt_id(b.id),
            name:   b.name.clone(),
            status: format!("{:?}", b.status)
        });
    }
    for d in &resp.dedicated_servers {
        rows.push(ResourceRow {
            kind:   "Dedicated".to_string(),
            id:     fmt_id(d.id),
            name:   d.name.clone(),
            status: format!("{:?}", d.status)
        });
    }

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_project_resources"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for row in &rows {
                println!("{}\t{}\t{}", row.kind, row.id, row.name);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
