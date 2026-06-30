// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

use rust_i18n::t;
use tabled::Tabled;
use timeweb_rs::{
    apis::{configuration::Configuration, images_api},
    models
};

use crate::{error::TwcError, output::OutputFormat};

/// Formats an integer identifier for display without numeric casts.
fn fmt_id<T: std::fmt::Display>(v: T) -> String {
    v.to_string()
}

/// Compact row for the image list table.
#[derive(Tabled)]
struct ImageRow {
    #[tabled(rename = "ID")]
    id:       String,
    #[tabled(rename = "Name")]
    name:     String,
    #[tabled(rename = "Status")]
    status:   String,
    #[tabled(rename = "Size")]
    size:     String,
    #[tabled(rename = "Location")]
    location: String
}

impl fmt::Display for ImageRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.id, self.name, self.status, self.size, self.location
        )
    }
}

/// Lists all images.
///
/// # Overview
///
/// Fetches all images from the Timeweb Cloud API and displays
/// them in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list(config: &Configuration, format: OutputFormat) -> Result<(), TwcError> {
    let resp = images_api::get_images(config, None, None).await?;

    let rows: Vec<ImageRow> = resp
        .images
        .iter()
        .map(|i| ImageRow {
            id:       i.id.clone(),
            name:     i.name.clone(),
            status:   format!("{:?}", i.status),
            size:     format!("{} MB", i.size),
            location: i.location.clone()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_images"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.images)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for i in &resp.images {
                println!("{}\t{}", i.id, i.name);
            }
        }
    }
    Ok(())
}

/// Deletes an image by ID.
///
/// # Overview
///
/// Sends a delete request for the specified image.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn delete(config: &Configuration, id: &str) -> Result<(), TwcError> {
    images_api::delete_image(config, id).await?;
    println!("{}", t!("cli.image_deleted", id => id));
    Ok(())
}

/// Shows detailed information about a single image.
///
/// # Overview
///
/// Fetches one image by ID and displays its core attributes in the
/// requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn info(config: &Configuration, id: &str, format: OutputFormat) -> Result<(), TwcError> {
    let resp = images_api::get_image(config, id).await?;
    let image = &resp.image;

    match format {
        OutputFormat::Table => {
            println!("ID:          {}", image.id);
            println!("Name:        {}", image.name);
            println!("Status:      {:?}", image.status);
            println!("Size:        {} MB", image.size);
            println!("Location:    {}", image.location);
            println!("Disk ID:     {}", fmt_id(image.disk_id));
            println!("Created At:  {}", image.created_at);
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            if let Some(out) = crate::output::serialized(format, &resp.image) {
                println!("{}", out?);
            }
        }
        OutputFormat::Quiet => {
            println!("{}\t{}\t{:?}", image.id, image.name, image.status);
        }
    }
    Ok(())
}

/// Creates a new image in the given location.
///
/// # Overview
///
/// Builds an [`models::ImageInApi`] request with the supplied name and
/// location and creates the image via the Timeweb Cloud API. The operating
/// system is set to [`models::Os::CustomOs`] for a user-provided image.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn create(
    config: &Configuration,
    name: &str,
    location: &str,
    format: OutputFormat
) -> Result<(), TwcError> {
    let mut req = models::ImageInApi::new(location.to_string(), models::Os::CustomOs);
    req.name = Some(name.to_string());

    let resp = images_api::create_image(config, req).await?;
    let image = &resp.image;

    match format {
        OutputFormat::Table => {
            println!(
                "{}",
                t!("cli.image_created", name => image.name, id => image.id)
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            if let Some(out) = crate::output::serialized(format, &resp.image) {
                println!("{}", out?);
            }
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", image.id, image.name);
        }
    }
    Ok(())
}

/// Updates an image's name.
///
/// # Overview
///
/// Sends a partial update for the specified image. Only the name is
/// changed when provided.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn set(config: &Configuration, id: &str, name: Option<&str>) -> Result<(), TwcError> {
    let mut req = models::ImageUpdateApi::new();
    req.name = name.map(String::from);

    let resp = images_api::update_image(config, id, req).await?;
    let image = &resp.image;
    println!(
        "{}",
        t!("cli.image_updated", name => image.name, id => image.id)
    );
    Ok(())
}
