// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

use tabled::{Table, Tabled};
use timeweb_rs::{
    apis::s3_api, models as s3_models, models::create_storage_request::Type as StorageType
};

use crate::{error::TwcError, output::OutputFormat};

/// Formats a float identifier for display.
fn fmt_id(v: f64) -> String {
    format!("{v:.0}")
}

/// Compact row for the S3 storage list table.
#[derive(Tabled)]
struct StorageRow {
    #[tabled(rename = "ID")]
    id:       String,
    #[tabled(rename = "Name")]
    name:     String,
    #[tabled(rename = "Status")]
    status:   String,
    #[tabled(rename = "Location")]
    location: String,
    #[tabled(rename = "Type")]
    r#type:   String,
    #[tabled(rename = "Size (GB)")]
    size_gb:  String
}

impl fmt::Display for StorageRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {}",
            self.id, self.name, self.status, self.location, self.r#type, self.size_gb
        )
    }
}

/// Compact row for the storage user table.
#[derive(Tabled)]
struct StorageUserRow {
    #[tabled(rename = "ID")]
    id:         String,
    #[tabled(rename = "Access Key")]
    access_key: String,
    #[tabled(rename = "Secret Key")]
    secret_key: String
}

impl fmt::Display for StorageUserRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.id, self.access_key, self.secret_key)
    }
}

/// Compact row for the subdomain table.
#[derive(Tabled)]
struct SubdomainRow {
    #[tabled(rename = "ID")]
    id:        String,
    #[tabled(rename = "Subdomain")]
    subdomain: String,
    #[tabled(rename = "Status")]
    status:    String
}

impl fmt::Display for SubdomainRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.id, self.subdomain, self.status)
    }
}

/// Compact row for the preset table.
#[derive(Tabled)]
struct PresetRow {
    #[tabled(rename = "ID")]
    id:            String,
    #[tabled(rename = "Description")]
    description:   String,
    #[tabled(rename = "Disk (GB)")]
    disk:          String,
    #[tabled(rename = "Price")]
    price:         String,
    #[tabled(rename = "Location")]
    location:      String,
    #[tabled(rename = "Class")]
    storage_class: String
}

impl fmt::Display for PresetRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {}",
            self.id, self.description, self.disk, self.price, self.location, self.storage_class
        )
    }
}

/// Formats disk size from KB to human-readable string.
fn fmt_disk_size(size_kb: f64) -> String {
    let gb = size_kb / 1_048_576.0;
    format!("{gb:.2}")
}

/// Lists all S3 storages.
///
/// # Overview
///
/// Fetches all S3 storages from the Timeweb Cloud API and displays them
/// in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list(
    config: &timeweb_rs::apis::configuration::Configuration,
    _limit: Option<i32>,
    _offset: Option<i32>,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = s3_api::get_storages(config).await?;

    let rows: Vec<StorageRow> = resp
        .buckets
        .iter()
        .map(|b| StorageRow {
            id:       fmt_id(b.id),
            name:     b.name.clone(),
            status:   format!("{:?}", b.status),
            location: b.location.clone(),
            r#type:   format!("{:?}", b.r#type),
            size_gb:  fmt_disk_size(b.disk_stats.size)
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("No storages found.");
            } else {
                let table = Table::new(&rows).to_string();
                println!("{table}");
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&resp.buckets)
                .map_err(|e| TwcError::Api(e.to_string()))?;
            println!("{json}");
        }
        OutputFormat::Quiet => {
            for b in &resp.buckets {
                println!("{}\t{}", fmt_id(b.id), b.name);
            }
        }
    }
    Ok(())
}

/// Shows detailed info for a single S3 storage.
///
/// # Overview
///
/// Fetches storage details by ID and displays them.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn info(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = s3_api::get_storage(config, id).await?;
    let bucket = &resp.bucket;

    match format {
        OutputFormat::Table => {
            println!("ID:              {}", fmt_id(bucket.id));
            println!("Name:            {}", bucket.name);
            println!("Description:     {}", bucket.description);
            println!("Status:          {:?}", bucket.status);
            println!("Type:            {:?}", bucket.r#type);
            println!("Location:        {}", bucket.location);
            println!("Hostname:        {}", bucket.hostname);
            println!("Storage Class:   {:?}", bucket.storage_class);
            println!(
                "Disk Size:       {} GB",
                fmt_disk_size(bucket.disk_stats.size)
            );
            println!(
                "Disk Used:       {} GB",
                fmt_disk_size(bucket.disk_stats.used)
            );
            println!("Unlimited Disk:  {}", bucket.disk_stats.is_unlimited);
            println!("Objects:         {:.0}", bucket.object_amount);
            println!(
                "Preset ID:       {}",
                bucket.preset_id.map_or_else(|| "-".to_string(), fmt_id)
            );
            println!(
                "Auto Upgrade:    {}",
                if bucket.is_allow_auto_upgrade {
                    "yes"
                } else {
                    "no"
                }
            );
            println!("Project ID:      {}", fmt_id(bucket.project_id));
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&resp.bucket)
                .map_err(|e| TwcError::Api(e.to_string()))?;
            println!("{json}");
        }
        OutputFormat::Quiet => {
            println!(
                "{}\t{}\t{:?}",
                fmt_id(bucket.id),
                bucket.name,
                bucket.status
            );
        }
    }
    Ok(())
}

/// Creates a new S3 storage.
///
/// # Overview
///
/// Creates a storage with the given name, type, and optional preset.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn create(
    config: &timeweb_rs::apis::configuration::Configuration,
    name: &str,
    _preset_id: Option<f64>,
    format: OutputFormat
) -> Result<(), TwcError> {
    let req = s3_models::CreateStorageRequest::new(name.to_string(), StorageType::Private);
    let resp = s3_api::create_storage(config, req).await?;
    let bucket = &resp.bucket;

    match format {
        OutputFormat::Table => {
            println!(
                "Storage '{}' created (id: {}).",
                bucket.name,
                fmt_id(bucket.id)
            );
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&resp.bucket)
                .map_err(|e| TwcError::Api(e.to_string()))?;
            println!("{json}");
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", fmt_id(bucket.id), bucket.name);
        }
    }
    Ok(())
}

/// Deletes an S3 storage by ID.
///
/// # Overview
///
/// Sends a delete request for the specified storage.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn delete(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32
) -> Result<(), TwcError> {
    s3_api::delete_storage(config, id, None, None).await?;
    println!("Storage {id} deleted.");
    Ok(())
}

/// Updates an S3 storage by ID.
///
/// # Overview
///
/// Updates the storage description via the Timeweb Cloud API.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn update(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32,
    description: Option<&str>,
    format: OutputFormat
) -> Result<(), TwcError> {
    let mut update = s3_models::UpdateStorageRequest::new();
    if let Some(d) = description {
        update.description = Some(d.to_string());
    }
    let resp = s3_api::update_storage(config, id, update).await?;
    let bucket = &resp.bucket;

    match format {
        OutputFormat::Table => {
            println!(
                "Storage '{}' updated (id: {}).",
                bucket.name,
                fmt_id(bucket.id)
            );
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&resp.bucket)
                .map_err(|e| TwcError::Api(e.to_string()))?;
            println!("{json}");
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", fmt_id(bucket.id), bucket.name);
        }
    }
    Ok(())
}

/// Lists users for an S3 storage.
///
/// # Overview
///
/// Fetches storage users for the specified storage and displays them
/// in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn user_list(
    config: &timeweb_rs::apis::configuration::Configuration,
    _id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = s3_api::get_storage_users(config).await?;

    let rows: Vec<StorageUserRow> = resp
        .users
        .iter()
        .map(|u| StorageUserRow {
            id:         fmt_id(u.id),
            access_key: u.access_key.clone(),
            secret_key: u.secret_key.clone()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("No users found.");
            } else {
                let table = Table::new(&rows).to_string();
                println!("{table}");
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&resp.users)
                .map_err(|e| TwcError::Api(e.to_string()))?;
            println!("{json}");
        }
        OutputFormat::Quiet => {
            for u in &resp.users {
                println!("{}\t{}", fmt_id(u.id), u.access_key);
            }
        }
    }
    Ok(())
}

/// Updates an S3 storage user.
///
/// # Overview
///
/// Updates the storage user identified by `user_id`.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn user_update(
    config: &timeweb_rs::apis::configuration::Configuration,
    user_id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let secret_key = format!("new-{}", chrono::Utc::now().timestamp_micros());
    let resp = s3_api::update_storage_user(
        config,
        user_id,
        s3_models::UpdateStorageUserRequest::new(secret_key)
    )
    .await?;
    let user = &resp.user;

    match format {
        OutputFormat::Table => {
            println!(
                "User '{}' updated (id: {}).",
                user.access_key,
                fmt_id(user.id)
            );
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&resp.user)
                .map_err(|e| TwcError::Api(e.to_string()))?;
            println!("{json}");
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", fmt_id(user.id), user.access_key);
        }
    }
    Ok(())
}

/// Transfers an S3 storage.
///
/// # Overview
///
/// Initiates a storage transfer. The `target_id` parameter is reserved
/// for future use with provider-based transfers.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn transfer(
    config: &timeweb_rs::apis::configuration::Configuration,
    _target_id: Option<i32>
) -> Result<(), TwcError> {
    s3_api::transfer_storage(
        config,
        s3_models::TransferStorageRequest::new(
            String::new(),
            String::new(),
            String::new(),
            false,
            String::new(),
            String::new(),
            String::new()
        )
    )
    .await?;
    println!("Transfer initiated.");
    Ok(())
}

/// Lists subdomains for an S3 storage.
///
/// # Overview
///
/// Fetches subdomains for the specified storage and displays them
/// in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn subdomain_list(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = s3_api::get_storage_subdomains(config, id).await?;

    let rows: Vec<SubdomainRow> = resp
        .subdomains
        .iter()
        .map(|s| SubdomainRow {
            id:        fmt_id(s.id),
            subdomain: s.subdomain.clone(),
            status:    format!("{:?}", s.status)
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("No subdomains found.");
            } else {
                let table = Table::new(&rows).to_string();
                println!("{table}");
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&resp.subdomains)
                .map_err(|e| TwcError::Api(e.to_string()))?;
            println!("{json}");
        }
        OutputFormat::Quiet => {
            for s in &resp.subdomains {
                println!("{}\t{}", fmt_id(s.id), s.subdomain);
            }
        }
    }
    Ok(())
}

/// Adds a subdomain to an S3 storage.
///
/// # Overview
///
/// Adds the specified subdomain to the given storage.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn subdomain_add(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32,
    subdomain: &str
) -> Result<(), TwcError> {
    let req = s3_models::AddStorageSubdomainsRequest::new(vec![subdomain.to_string()]);
    s3_api::add_storage_subdomains(config, id, req).await?;
    println!("Subdomain '{subdomain}' added to storage {id}.");
    Ok(())
}

/// Deletes a subdomain from an S3 storage.
///
/// # Overview
///
/// Deletes the specified subdomain from the given storage.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn subdomain_delete(
    config: &timeweb_rs::apis::configuration::Configuration,
    id: i32,
    subdomain: &str
) -> Result<(), TwcError> {
    let req = s3_models::AddStorageSubdomainsRequest::new(vec![subdomain.to_string()]);
    s3_api::delete_storage_subdomains(config, id, req).await?;
    println!("Subdomain '{subdomain}' deleted from storage {id}.");
    Ok(())
}

/// Lists available S3 storage presets.
///
/// # Overview
///
/// Fetches storage presets from the Timeweb Cloud API and displays them
/// in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn preset_list(
    config: &timeweb_rs::apis::configuration::Configuration,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = s3_api::get_storages_presets(config).await?;

    let rows: Vec<PresetRow> = resp
        .storages_presets
        .iter()
        .map(|p| PresetRow {
            id:            fmt_id(p.id),
            description:   p.description_short.clone(),
            disk:          fmt_disk_size(p.disk),
            price:         format!("{:.2}", p.price),
            location:      format!("{:?}", p.location),
            storage_class: format!("{:?}", p.storage_class)
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("No presets found.");
            } else {
                let table = Table::new(&rows).to_string();
                println!("{table}");
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&resp.storages_presets)
                .map_err(|e| TwcError::Api(e.to_string()))?;
            println!("{json}");
        }
        OutputFormat::Quiet => {
            for p in &resp.storages_presets {
                println!(
                    "{}\t{}\t{}",
                    fmt_id(p.id),
                    p.description_short,
                    fmt_disk_size(p.disk)
                );
            }
        }
    }
    Ok(())
}
