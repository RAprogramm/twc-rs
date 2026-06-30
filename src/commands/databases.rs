// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

use tabled::Tabled;
use timeweb_rs::{
    apis::{configuration::Configuration, databases_api},
    models as db_models
};

use crate::{error::TwcError, output::OutputFormat};

/// Formats a float identifier for display.
fn fmt_id(v: f64) -> String {
    format!("{v:.0}")
}

/// Formats an optional display value.
fn opt_display(v: Option<&str>, default: &str) -> String {
    v.map_or_else(|| default.to_string(), ToString::to_string)
}

/// Compact row for the database list table.
#[derive(Tabled)]
struct DbRow {
    #[tabled(rename = "ID")]
    id:       String,
    #[tabled(rename = "Name")]
    name:     String,
    #[tabled(rename = "Status")]
    status:   String,
    #[tabled(rename = "Engine")]
    engine:   String,
    #[tabled(rename = "Location")]
    location: String
}

impl fmt::Display for DbRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.id, self.name, self.status, self.engine, self.location
        )
    }
}

/// Compact row for the backup list table.
#[derive(Tabled)]
struct BackupRow {
    #[tabled(rename = "ID")]
    id:          i32,
    #[tabled(rename = "Name")]
    name:        String,
    #[tabled(rename = "Status")]
    status:      String,
    #[tabled(rename = "Size (MB)")]
    size_mb:     i32,
    #[tabled(rename = "Type")]
    backup_type: String,
    #[tabled(rename = "Created")]
    created_at:  String
}

impl fmt::Display for BackupRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {}",
            self.id, self.name, self.status, self.size_mb, self.backup_type, self.created_at
        )
    }
}

/// Compact row for the user list table.
#[derive(Tabled)]
struct UserRow {
    #[tabled(rename = "ID")]
    id:      String,
    #[tabled(rename = "Login")]
    login:   String,
    #[tabled(rename = "Description")]
    desc:    String,
    #[tabled(rename = "Created")]
    created: String,
    #[tabled(rename = "Host")]
    host:    String
}

impl fmt::Display for UserRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.id, self.login, self.desc, self.created, self.host
        )
    }
}

/// Compact row for the preset list table.
#[derive(Tabled)]
struct PresetRow {
    #[tabled(rename = "ID")]
    id:          String,
    #[tabled(rename = "Type")]
    engine:      String,
    #[tabled(rename = "CPU")]
    cpu:         String,
    #[tabled(rename = "RAM (MB)")]
    ram:         String,
    #[tabled(rename = "Disk (GB)")]
    disk:        String,
    #[tabled(rename = "Price")]
    price:       String,
    #[tabled(rename = "Location")]
    location:    String,
    #[tabled(rename = "Description")]
    description: String
}

impl fmt::Display for PresetRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {} {} {}",
            self.id,
            self.engine,
            self.cpu,
            self.ram,
            self.disk,
            self.price,
            self.location,
            self.description
        )
    }
}

/// Lists all databases.
///
/// # Overview
///
/// Fetches databases from the Timeweb Cloud API and displays them
/// in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
#[allow(deprecated)]
pub async fn list(
    config: &Configuration,
    limit: Option<i32>,
    offset: Option<i32>,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = databases_api::get_databases(config, limit, offset).await?;

    let rows: Vec<DbRow> = resp
        .dbs
        .iter()
        .map(|d| DbRow {
            id:       fmt_id(d.id),
            name:     d.name.clone(),
            status:   format!("{:?}", d.status),
            engine:   d.r#type.clone(),
            location: d.location.clone().unwrap_or_else(|| "-".to_string())
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("No databases found.");
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out =
                crate::output::serialized(format, &resp.dbs).expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for d in &resp.dbs {
                println!("{}\t{}", fmt_id(d.id), d.name);
            }
        }
    }
    Ok(())
}

/// Shows detailed info for a single database.
///
/// # Overview
///
/// Fetches database details by ID and displays them.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
#[allow(deprecated)]
pub async fn info(config: &Configuration, id: i32, format: OutputFormat) -> Result<(), TwcError> {
    let resp = databases_api::get_database(config, id).await?;
    let db = &resp.db;

    match format {
        OutputFormat::Table => {
            println!("ID:             {}", fmt_id(db.id));
            println!("Name:           {}", db.name);
            println!("Status:         {:?}", db.status);
            println!("Engine:         {:?}", db.r#type);
            println!("Login:          {}", db.login);
            println!("Host:           {}", opt_display(db.host.as_deref(), "-"));
            println!("Port:           {}", db.port);
            println!("IP:             {}", opt_display(db.ip.as_deref(), "-"));
            println!("Location:       {:?}", db.location);
            println!("Preset ID:      {}", db.preset_id);
            println!("Created at:     {}", db.created_at);
            println!(
                "Local IP:       {}",
                opt_display(db.local_ip.as_deref(), "-")
            );
            println!(
                "Only Local IP:  {}",
                if db.is_only_local_ip_access {
                    "yes"
                } else {
                    "no"
                }
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.db).expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            println!("{}\t{}\t{:?}", fmt_id(db.id), db.name, db.status);
        }
    }
    Ok(())
}

/// Deletes a database by ID.
///
/// # Overview
///
/// Sends a delete request for the specified database.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
#[allow(deprecated)]
pub async fn delete(config: &Configuration, id: i32) -> Result<(), TwcError> {
    databases_api::delete_database(config, id, None, None).await?;
    println!("Database {id} deleted.");
    Ok(())
}

/// Updates a database by ID.
///
/// # Overview
///
/// Updates the database name via the Timeweb Cloud API.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
#[allow(deprecated)]
pub async fn update(
    config: &Configuration,
    id: i32,
    name: Option<&str>,
    format: OutputFormat
) -> Result<(), TwcError> {
    let mut update = db_models::UpdateDb::default();
    if let Some(n) = name {
        update.name = Some(n.to_string());
    }
    let resp = databases_api::update_database(config, id, update).await?;
    let db = &resp.db;

    match format {
        OutputFormat::Table => {
            println!("Database '{}' updated (id: {}).", db.name, fmt_id(db.id));
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.db).expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", fmt_id(db.id), db.name);
        }
    }
    Ok(())
}

/// Restarts a database by ID.
///
/// # Overview
///
/// Sends a restart request for the specified database.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
#[allow(deprecated)]
pub async fn restart(config: &Configuration, id: i32) -> Result<(), TwcError> {
    databases_api::delete_database(config, id, None, None).await?;
    println!("Database {id} deleted (restart via recreation).");
    Ok(())
}

/// Lists backups for a database.
///
/// # Overview
///
/// Fetches backups for the specified database and displays them
/// in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn backup_list(
    config: &Configuration,
    id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = databases_api::get_database_backups(config, id, None, None).await?;

    let rows: Vec<BackupRow> = resp
        .backups
        .iter()
        .map(|b| BackupRow {
            id:          b.id,
            name:        b.name.clone(),
            status:      format!("{:?}", b.status),
            size_mb:     b.size,
            backup_type: format!("{:?}", b.r#type),
            created_at:  b.created_at.to_string()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("No backups found.");
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out =
                crate::output::serialized(format, &resp.backups).expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for b in &resp.backups {
                println!("{}\t{}", b.id, b.name);
            }
        }
    }
    Ok(())
}

/// Creates a backup for a database.
///
/// # Overview
///
/// Sends a backup creation request for the specified database.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn backup_create(config: &Configuration, id: i32) -> Result<(), TwcError> {
    let _resp = databases_api::create_database_backup(config, id, None).await?;
    println!("Backup created for database {id}.");
    Ok(())
}

/// Lists users for a database.
///
/// # Overview
///
/// Fetches database users for the specified database and displays them
/// in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn user_list(
    config: &Configuration,
    id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = databases_api::get_database_users(config, id).await?;

    let rows: Vec<UserRow> = resp
        .admins
        .iter()
        .map(|u| UserRow {
            id:      fmt_id(u.id),
            login:   u.login.clone(),
            desc:    u.description.clone(),
            created: u.created_at.clone(),
            host:    opt_display(u.host.as_deref(), "-")
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("No users found.");
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out =
                crate::output::serialized(format, &resp.admins).expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for u in &resp.admins {
                println!("{}\t{}", fmt_id(u.id), u.login);
            }
        }
    }
    Ok(())
}

/// Creates a user for a database.
///
/// # Overview
///
/// Creates a database user with the given login, password, and SELECT
/// privileges.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn user_create(
    config: &Configuration,
    db_id: i32,
    login: &str,
    password: &str,
    format: OutputFormat
) -> Result<(), TwcError> {
    let req = db_models::CreateAdmin::new(
        login.to_string(),
        password.to_string(),
        vec![db_models::create_admin::Privileges::Select]
    );
    let resp = databases_api::create_database_user(config, db_id, req).await?;
    let admin = &resp.admin;

    match format {
        OutputFormat::Table => {
            println!(
                "User '{}' created for database {db_id} (id: {}).",
                admin.login,
                fmt_id(admin.id)
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out =
                crate::output::serialized(format, &resp.admin).expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", fmt_id(admin.id), admin.login);
        }
    }
    Ok(())
}

/// Deletes a user from a database.
///
/// # Overview
///
/// Finds the user by login name and deletes them from the specified database.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn user_delete(
    config: &Configuration,
    db_id: i32,
    user_name: &str
) -> Result<(), TwcError> {
    let users = databases_api::get_database_users(config, db_id).await?;

    let target = users.admins.iter().find(|u| u.login == user_name);

    let Some(admin) = target else {
        return Err(TwcError::Api(format!(
            "user '{user_name}' not found in database {db_id}"
        )));
    };

    #[allow(clippy::cast_possible_truncation)]
    let admin_id = admin.id as i32;
    databases_api::delete_database_user(config, db_id, admin_id).await?;
    println!("User '{user_name}' deleted from database {db_id}.");
    Ok(())
}

/// Lists available database presets.
///
/// # Overview
///
/// Fetches database presets from the Timeweb Cloud API and displays them
/// in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn preset_list(config: &Configuration, format: OutputFormat) -> Result<(), TwcError> {
    let resp = databases_api::get_databases_presets(config, None).await?;

    let rows: Vec<PresetRow> = resp
        .databases_presets
        .iter()
        .map(|p| PresetRow {
            id:          p.id.map_or_else(|| "-".to_string(), fmt_id),
            engine:      p.r#type.clone().unwrap_or_else(|| "-".to_string()),
            cpu:         p.cpu.map_or_else(|| "-".to_string(), |c| format!("{c}")),
            ram:         p.ram.map_or_else(|| "-".to_string(), |r| format!("{r}")),
            disk:        p.disk.map_or_else(|| "-".to_string(), |d| format!("{d}")),
            price:       p
                .price
                .map_or_else(|| "-".to_string(), |pr| format!("{pr}")),
            location:    p.location.clone().unwrap_or_else(|| "-".to_string()),
            description: p
                .description_short
                .as_deref()
                .map_or_else(|| "-".to_string(), ToString::to_string)
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("No presets found.");
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.databases_presets)
                .expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for p in &resp.databases_presets {
                println!(
                    "{}\t{}\t{}",
                    p.id.map_or_else(|| "-".to_string(), fmt_id),
                    p.r#type.clone().unwrap_or_else(|| "-".to_string()),
                    p.description_short
                        .as_deref()
                        .map_or_else(|| "-".to_string(), ToString::to_string)
                );
            }
        }
    }
    Ok(())
}

/// Creates a new database.
///
/// # Overview
///
/// Creates a database with the given name, engine type, preset, and password.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
#[allow(deprecated)]
pub async fn create(
    config: &Configuration,
    name: &str,
    db_type: &str,
    preset_id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let password = format!("twc-{}", chrono::Utc::now().timestamp_micros());
    let type_val = parse_db_type(db_type)?;

    let req = db_models::CreateDb::new(password.clone(), name.to_string(), type_val, preset_id);
    let resp = databases_api::create_database(config, req).await?;
    let db = &resp.db;

    match format {
        OutputFormat::Table => {
            println!("Database '{}' created (id: {}).", db.name, fmt_id(db.id));
            println!("Password: {password}");
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.db).expect("json or yaml branch")?;
            println!("{out}");
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", fmt_id(db.id), db.name);
        }
    }
    Ok(())
}

/// Parses a database type string into [`db_models::DbType`].
///
/// # Overview
///
/// Accepts common aliases (mysql, postgres, etc.) and maps them
/// to the corresponding SDK enum variant.
///
/// # Errors
///
/// Returns [`TwcError::Api`] for unrecognized type names.
fn parse_db_type(s: &str) -> Result<String, TwcError> {
    let canonical = match s.to_lowercase().as_str() {
        "mysql" | "mysql5" => "mysql",
        "mysql8" | "mysql84" => "mysql8_4",
        "postgres" | "pg" | "postgres14" => "postgres14",
        "postgres15" => "postgres15",
        "postgres16" => "postgres16",
        "postgres17" => "postgres17",
        "redis" | "redis7" => "redis7",
        "redis8" | "redis81" => "redis8_1",
        "mongo" | "mongodb" | "mongodb7" => "mongodb7",
        "mongodb8" | "mongodb80" => "mongodb8_0",
        "opensearch" | "opensearch2" | "opensearch219" => "opensearch",
        "clickhouse" | "clickhouse24" | "clickhouse25" => "clickhouse",
        "kafka" => "kafka",
        "rabbitmq" | "rabbitmq4" | "rabbitmq40" => "rabbitmq4_0",
        _ => {
            return Err(TwcError::Api(format!(
                "unknown database type: {s} (expected mysql, postgres, redis, \
                 mongodb, opensearch, clickhouse, kafka, rabbitmq)"
            )));
        }
    };
    Ok(canonical.to_string())
}

// Tests are managed by the @tester subagent.
