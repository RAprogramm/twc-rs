// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

use tabled::Tabled;
use timeweb_rs::apis::{configuration::Configuration, images_api};

use crate::{error::TwcError, output::OutputFormat};

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
                println!("No images found.");
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out =
                crate::output::serialized(format, &resp.images).expect("json or yaml branch")?;
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
    println!("Image {id} deleted.");
    Ok(())
}
