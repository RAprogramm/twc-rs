// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

use rust_i18n::t;
use tabled::Tabled;
use timeweb_rs::{
    apis::{configuration::Configuration, databases_api},
    models as db_models
};

use crate::{error::TwcError, output::OutputFormat};

/// Formats a float identifier for display.
fn fmt_id<T: std::fmt::Display>(v: T) -> String {
    v.to_string()
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

/// Compact row for the database type list table.
#[derive(Tabled)]
struct TypeRow {
    #[tabled(rename = "Type")]
    engine:      String,
    #[tabled(rename = "Version")]
    version:     String,
    #[tabled(rename = "Name")]
    name:        String,
    #[tabled(rename = "Replication")]
    replication: String,
    #[tabled(rename = "Deprecated")]
    deprecated:  String
}

impl fmt::Display for TypeRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.engine, self.version, self.name, self.replication, self.deprecated
        )
    }
}

/// Compact row for the database instance list table.
#[derive(Tabled)]
struct InstanceRow {
    #[tabled(rename = "ID")]
    id:          String,
    #[tabled(rename = "Name")]
    name:        String,
    #[tabled(rename = "Description")]
    description: String,
    #[tabled(rename = "Created")]
    created_at:  String
}

impl fmt::Display for InstanceRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.id, self.name, self.description, self.created_at
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
pub async fn list(
    config: &Configuration,
    limit: Option<i32>,
    offset: Option<i32>,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = databases_api::get_database_clusters(config, limit, offset).await?;

    let rows: Vec<DbRow> = resp
        .dbs
        .iter()
        .map(|d| DbRow {
            id:       fmt_id(d.id),
            name:     d.name.clone(),
            status:   format!("{:?}", d.status),
            engine:   d.r#type.clone().unwrap_or_else(|| "-".to_string()),
            location: d.location.clone().unwrap_or_else(|| "-".to_string())
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_databases_found"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.dbs)
                .transpose()?
                .unwrap_or_default();
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
pub async fn info(config: &Configuration, id: i32, format: OutputFormat) -> Result<(), TwcError> {
    let resp = databases_api::get_database_cluster(config, id).await?;
    let db = &resp.db;

    match format {
        OutputFormat::Table => {
            let disk_size = db.disk.as_ref().map_or(0.0, |disk| disk.size);
            println!("ID:             {}", fmt_id(db.id));
            println!("Name:           {}", db.name);
            println!("Status:         {:?}", db.status);
            println!(
                "Engine:         {}",
                db.r#type.clone().unwrap_or_else(|| "-".to_string())
            );
            println!(
                "Port:           {}",
                db.port.map_or_else(|| "-".to_string(), |p| p.to_string())
            );
            println!("Location:       {:?}", db.location);
            println!(
                "Preset ID:      {}",
                db.preset_id
                    .map_or_else(|| "-".to_string(), |id| id.to_string())
            );
            println!("Created at:     {}", db.created_at);
            println!("Disk (GB):      {disk_size}");
            println!(
                "Public network: {}",
                if db.is_enabled_public_network {
                    "yes"
                } else {
                    "no"
                }
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.db)
                .transpose()?
                .unwrap_or_default();
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
pub async fn delete(config: &Configuration, id: i32) -> Result<(), TwcError> {
    databases_api::delete_database_cluster(config, id).await?;
    println!("{}", t!("cli.database_deleted", id => id));
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
pub async fn update(
    config: &Configuration,
    id: i32,
    name: Option<&str>,
    format: OutputFormat
) -> Result<(), TwcError> {
    let mut update = db_models::UpdateCluster::default();
    if let Some(n) = name {
        update.name = Some(n.to_string());
    }
    let resp = databases_api::update_database_cluster(config, id, update).await?;
    let db = &resp.db;

    match format {
        OutputFormat::Table => {
            println!(
                "{}",
                t!("cli.database_updated", name => db.name, id => fmt_id(db.id))
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.db)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", fmt_id(db.id), db.name);
        }
    }
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
                println!("{}", t!("cli.no_backups_found"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.backups)
                .transpose()?
                .unwrap_or_default();
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
    println!("{}", t!("cli.backup_created", id => id));
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
                println!("{}", t!("cli.no_users_found"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.admins)
                .transpose()?
                .unwrap_or_default();
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
    let mut req = db_models::CreateAdmin::new(login.to_string(), password.to_string());
    req.privileges = Some(vec![db_models::create_admin::Privileges::Select]);
    let resp = databases_api::create_database_user(config, db_id, req).await?;
    let admin = &resp.admin;

    match format {
        OutputFormat::Table => {
            println!(
                "{}",
                t!("cli.db_user_created", login => admin.login, db_id => db_id, id => fmt_id(admin.id))
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.admin)
                .transpose()?
                .unwrap_or_default();
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
    println!(
        "{}",
        t!("cli.db_user_deleted", login => user_name, db_id => db_id)
    );
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
    let resp = databases_api::get_databases_presets(config, None, None).await?;

    let rows: Vec<PresetRow> = resp
        .databases_presets
        .iter()
        .map(|p| PresetRow {
            id:          p.id.map_or_else(|| "-".to_string(), fmt_id),
            engine:      p
                .r#type
                .map_or_else(|| "-".to_string(), |t| format!("{t:?}")),
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
                println!("{}", t!("cli.no_presets_found"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.databases_presets)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for p in &resp.databases_presets {
                println!(
                    "{}\t{}\t{}",
                    p.id.map_or_else(|| "-".to_string(), fmt_id),
                    p.r#type
                        .map_or_else(|| "-".to_string(), |t| format!("{t:?}")),
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
pub async fn create(
    config: &Configuration,
    name: &str,
    db_type: &str,
    preset_id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let password = format!("twc-{}", chrono::Utc::now().timestamp_micros());
    let type_val = parse_db_type(db_type)?;

    let mut req = db_models::CreateCluster::new(name.to_string(), type_val);
    req.preset_id = Some(preset_id);
    let resp = databases_api::create_database_cluster(config, req).await?;
    let db = &resp.db;

    match format {
        OutputFormat::Table => {
            println!(
                "{}",
                t!("cli.database_created", name => db.name, id => fmt_id(db.id))
            );
            println!("{}", t!("cli.password", password => password));
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &resp.db)
                .transpose()?
                .unwrap_or_default();
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

/// Lists available database cluster types (DBMS engines and versions).
///
/// # Overview
///
/// Fetches the catalog of supported database cluster types from the
/// Timeweb Cloud API and displays them in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list_types(config: &Configuration, format: OutputFormat) -> Result<(), TwcError> {
    let resp = databases_api::get_database_cluster_types(config).await?;

    let rows: Vec<TypeRow> = resp
        .types
        .iter()
        .map(|t| TypeRow {
            engine:      t.r#type.clone(),
            version:     t.version.clone(),
            name:        t.name.clone(),
            replication: if t.is_available_replication {
                "yes".to_string()
            } else {
                "no".to_string()
            },
            deprecated:  if t.is_deprecated {
                "yes".to_string()
            } else {
                "no".to_string()
            }
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_database_types_found"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            if let Some(out) = crate::output::serialized(format, &resp.types) {
                println!("{}", out?);
            }
        }
        OutputFormat::Quiet => {
            for t in &resp.types {
                println!("{}\t{}", t.r#type, t.version);
            }
        }
    }
    Ok(())
}

/// Lists individual database instances within a cluster.
///
/// # Overview
///
/// Fetches the individual databases hosted within the specified cluster
/// and displays them in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list_instances(
    config: &Configuration,
    id: i32,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = databases_api::get_database_instances(config, id).await?;

    let rows: Vec<InstanceRow> = resp
        .instances
        .iter()
        .map(|i| InstanceRow {
            id:          fmt_id(i.id),
            name:        i.name.clone(),
            description: i.description.clone(),
            created_at:  i.created_at.clone()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_database_instances_found"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            if let Some(out) = crate::output::serialized(format, &resp.instances) {
                println!("{}", out?);
            }
        }
        OutputFormat::Quiet => {
            for i in &resp.instances {
                println!("{}\t{}", fmt_id(i.id), i.name);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_id_converts_to_string() {
        assert_eq!(fmt_id(42), "42");
        assert_eq!(fmt_id("abc"), "abc");
    }

    #[test]
    fn opt_display_some_returns_value() {
        assert_eq!(opt_display(Some("val"), "-"), "val");
    }

    #[test]
    fn opt_display_none_returns_default() {
        assert_eq!(opt_display(None, "-"), "-");
    }

    #[test]
    fn parse_db_type_mysql() {
        assert_eq!(parse_db_type("mysql").unwrap(), "mysql");
        assert_eq!(parse_db_type("mysql8").unwrap(), "mysql8_4");
    }

    #[test]
    fn parse_db_type_postgres() {
        assert_eq!(parse_db_type("postgres").unwrap(), "postgres14");
        assert_eq!(parse_db_type("pg").unwrap(), "postgres14");
        assert_eq!(parse_db_type("postgres17").unwrap(), "postgres17");
    }

    #[test]
    fn parse_db_type_redis() {
        assert_eq!(parse_db_type("redis").unwrap(), "redis7");
        assert_eq!(parse_db_type("redis8").unwrap(), "redis8_1");
    }

    #[test]
    fn parse_db_type_mongodb() {
        assert_eq!(parse_db_type("mongo").unwrap(), "mongodb7");
        assert_eq!(parse_db_type("mongodb8").unwrap(), "mongodb8_0");
    }

    #[test]
    fn parse_db_type_other_engines() {
        assert_eq!(parse_db_type("clickhouse").unwrap(), "clickhouse");
        assert_eq!(parse_db_type("kafka").unwrap(), "kafka");
        assert_eq!(parse_db_type("rabbitmq").unwrap(), "rabbitmq4_0");
        assert_eq!(parse_db_type("opensearch").unwrap(), "opensearch");
    }

    #[test]
    fn parse_db_type_invalid_returns_error() {
        let result = parse_db_type("unknown_db");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("unknown database type"));
    }

    #[test]
    fn db_row_display() {
        let row = DbRow {
            id:       "1".to_string(),
            name:     "mydb".to_string(),
            status:   "ready".to_string(),
            engine:   "mysql".to_string(),
            location: "ru-1".to_string()
        };
        assert_eq!(row.to_string(), "1 mydb ready mysql ru-1");
    }

    #[test]
    fn backup_row_display() {
        let row = BackupRow {
            id:          1,
            name:        "backup-1".to_string(),
            status:      "ready".to_string(),
            size_mb:     1024,
            backup_type: "full".to_string(),
            created_at:  "2026-01-01".to_string()
        };
        assert_eq!(row.to_string(), "1 backup-1 ready 1024 full 2026-01-01");
    }

    #[test]
    fn user_row_display() {
        let row = UserRow {
            id:      "1".to_string(),
            login:   "admin".to_string(),
            desc:    "primary".to_string(),
            created: "2026-01-01".to_string(),
            host:    "localhost".to_string()
        };
        assert_eq!(row.to_string(), "1 admin primary 2026-01-01 localhost");
    }

    #[test]
    fn preset_row_display() {
        let row = PresetRow {
            id:          "1".to_string(),
            engine:      "mysql".to_string(),
            cpu:         "2".to_string(),
            ram:         "4096".to_string(),
            disk:        "40".to_string(),
            price:       "5000".to_string(),
            location:    "ru-1".to_string(),
            description: "Basic".to_string()
        };
        assert_eq!(row.to_string(), "1 mysql 2 4096 40 5000 ru-1 Basic");
    }

    #[test]
    fn type_row_display() {
        let row = TypeRow {
            engine:      "mysql".to_string(),
            version:     "8.0".to_string(),
            name:        "MySQL".to_string(),
            replication: "yes".to_string(),
            deprecated:  "no".to_string()
        };
        assert_eq!(row.to_string(), "mysql 8.0 MySQL yes no");
    }

    #[test]
    fn instance_row_display() {
        let row = InstanceRow {
            id:          "1".to_string(),
            name:        "db-1".to_string(),
            description: "primary".to_string(),
            created_at:  "2026-01-01".to_string()
        };
        assert_eq!(row.to_string(), "1 db-1 primary 2026-01-01");
    }
}
