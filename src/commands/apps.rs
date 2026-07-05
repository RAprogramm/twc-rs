// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fmt;

use chrono::{DateTime, FixedOffset, Local, NaiveDate, TimeZone};
use rust_i18n::t;
use tabled::Tabled;
use timeweb_rs::{
    apis::{apps_api, configuration::Configuration},
    models
};

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

/// Resolves an app selector — a numeric ID or an exact app name — to an ID.
///
/// Numeric selectors pass through without a network round-trip; anything else
/// is matched against the names returned by the apps list endpoint.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network failures, when no app carries the
/// name, or when several apps share it.
pub async fn resolve_app(config: &Configuration, selector: &str) -> Result<String, TwcError> {
    if is_numeric_selector(selector) {
        return Ok(selector.to_owned());
    }
    let resp = apps_api::get_apps(config).await?;
    let apps: Vec<(String, String)> = resp
        .apps
        .iter()
        .map(|a| (fmt_id(a.id), a.name.clone()))
        .collect();
    match_app_selector(&apps, selector)
}

/// Returns `true` when the selector consists solely of ASCII digits.
fn is_numeric_selector(selector: &str) -> bool {
    !selector.is_empty() && selector.bytes().all(|b| b.is_ascii_digit())
}

/// Picks the single app whose name equals `selector` from `(id, name)` pairs.
///
/// # Errors
///
/// Returns [`TwcError::Api`] when the name matches zero or several apps.
fn match_app_selector(apps: &[(String, String)], selector: &str) -> Result<String, TwcError> {
    let mut ids = apps
        .iter()
        .filter(|(_, name)| name == selector)
        .map(|(id, _)| id.clone());
    let Some(first) = ids.next() else {
        return Err(TwcError::Api(
            t!("cli.app_not_found", name => selector).into_owned()
        ));
    };
    if ids.next().is_some() {
        return Err(TwcError::Api(
            t!("cli.app_ambiguous", name => selector).into_owned()
        ));
    }
    Ok(first)
}

/// Shows detailed info for a single app.
///
/// # Overview
///
/// Fetches an app by its identifier and prints its key fields.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn info(config: &Configuration, id: &str, format: OutputFormat) -> Result<(), TwcError> {
    let resp = apps_api::get_app(config, id).await?;
    let app = &resp.app;

    match format {
        OutputFormat::Table => {
            println!("ID:         {}", fmt_id(app.id));
            println!("Name:       {}", app.name);
            println!("Type:       {:?}", app.r#type);
            println!("Status:     {:?}", app.status);
            println!("IP:         {}", app.ip.clone().unwrap_or_default());
            println!("Location:   {}", app.location.clone().unwrap_or_default());
            println!(
                "Preset ID:  {}",
                app.preset_id.map(fmt_id).unwrap_or_default()
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            if let Some(out) = crate::output::serialized(format, &resp.app) {
                println!("{}", out?);
            }
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", fmt_id(app.id), app.name);
        }
    }
    Ok(())
}

/// Deletes an app by its identifier.
///
/// # Overview
///
/// Sends a delete request for the specified app and prints a confirmation.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn delete(config: &Configuration, id: &str) -> Result<(), TwcError> {
    apps_api::delete_app(config, id).await?;
    println!("{}", t!("cli.app_deleted", id => id));
    Ok(())
}

/// Compact row for the app presets table.
#[derive(Tabled)]
struct PresetRow {
    #[tabled(rename = "ID")]
    id:    String,
    #[tabled(rename = "Kind")]
    kind:  String,
    #[tabled(rename = "CPU")]
    cpu:   String,
    #[tabled(rename = "RAM")]
    ram:   String,
    #[tabled(rename = "Disk")]
    disk:  String,
    #[tabled(rename = "Price")]
    price: String
}

/// Lists available app presets (backend and frontend tariffs).
///
/// # Overview
///
/// Fetches all app presets and renders them in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list_presets(config: &Configuration, format: OutputFormat) -> Result<(), TwcError> {
    let resp = apps_api::get_apps_presets(config, "").await?;

    let mut rows: Vec<PresetRow> = Vec::new();
    if let Some(backend) = &resp.backend_presets {
        for p in backend {
            rows.push(PresetRow {
                id:    fmt_id(p.id),
                kind:  "backend".to_owned(),
                cpu:   fmt_id(p.cpu),
                ram:   fmt_id(p.ram),
                disk:  fmt_id(p.disk),
                price: fmt_id(p.price)
            });
        }
    }
    if let Some(frontend) = &resp.frontend_presets {
        for p in frontend {
            rows.push(PresetRow {
                id:    fmt_id(p.id),
                kind:  "frontend".to_owned(),
                cpu:   "-".to_owned(),
                ram:   "-".to_owned(),
                disk:  fmt_id(p.disk),
                price: fmt_id(p.price)
            });
        }
    }

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_app_presets"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            if let Some(out) = crate::output::serialized(format, &resp) {
                println!("{}", out?);
            }
        }
        OutputFormat::Quiet => {
            for r in &rows {
                println!("{}\t{}", r.id, r.kind);
            }
        }
    }
    Ok(())
}

/// Compact row for the VCS providers table.
#[derive(Tabled)]
struct ProviderRow {
    #[tabled(rename = "Provider ID")]
    provider_id: String,
    #[tabled(rename = "Login")]
    login:       String,
    #[tabled(rename = "Type")]
    kind:        String
}

/// Lists configured VCS providers.
///
/// # Overview
///
/// Fetches VCS providers linked to the account and renders them.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list_vcs_providers(
    config: &Configuration,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = apps_api::get_providers(config).await?;

    let rows: Vec<ProviderRow> = resp
        .providers
        .iter()
        .map(|p| ProviderRow {
            provider_id: fmt_id(p.provider_id),
            login:       p.login.clone(),
            kind:        format!("{:?}", p.provider_type)
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_vcs_providers"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            if let Some(out) = crate::output::serialized(format, &resp.providers) {
                println!("{}", out?);
            }
        }
        OutputFormat::Quiet => {
            for p in &resp.providers {
                println!("{}\t{}", fmt_id(p.provider_id), p.login);
            }
        }
    }
    Ok(())
}

/// Compact row for the repositories table.
#[derive(Tabled)]
struct RepoRow {
    #[tabled(rename = "ID")]
    id:        String,
    #[tabled(rename = "Name")]
    name:      String,
    #[tabled(rename = "Full Name")]
    full_name: String,
    #[tabled(rename = "URL")]
    url:       String,
    #[tabled(rename = "Private")]
    private:   String
}

/// Lists repositories available from a connected VCS provider.
///
/// # Overview
///
/// Fetches repositories of the given VCS provider and renders them.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list_repositories(
    config: &Configuration,
    provider_id: &str,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = apps_api::get_repositories(config, provider_id).await?;

    let rows: Vec<RepoRow> = resp
        .repositories
        .iter()
        .map(|r| RepoRow {
            id:        fmt_id(r.id),
            name:      r.name.clone(),
            full_name: r.full_name.clone(),
            url:       r.url.clone(),
            private:   r.is_private.to_string()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_repositories"));
            } else {
                let table = crate::output::render_table(&rows);
                println!("{table}");
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            if let Some(out) = crate::output::serialized(format, &resp.repositories) {
                println!("{}", out?);
            }
        }
        OutputFormat::Quiet => {
            for r in &resp.repositories {
                println!("{}\t{}", fmt_id(r.id), r.name);
            }
        }
    }
    Ok(())
}

/// Canonical wire value for the app type.
///
/// # Errors
///
/// Returns [`TwcError::Api`] when the value is not `backend` or `frontend`.
fn parse_app_type(value: &str) -> Result<&'static str, TwcError> {
    match value.to_lowercase().as_str() {
        "backend" => Ok("backend"),
        "frontend" => Ok("frontend"),
        _ => Err(TwcError::Api(
            t!("cli.app_invalid_type", value => value).into_owned()
        ))
    }
}

/// Canonical wire value for a supported framework.
///
/// Matches the user-supplied name case-insensitively against the framework
/// identifiers accepted by the Timeweb Cloud API.
///
/// # Errors
///
/// Returns [`TwcError::Api`] when the framework is not recognised.
fn parse_framework(value: &str) -> Result<&'static str, TwcError> {
    const NAMES: [&str; 26] = [
        "django",
        "express",
        "phoenix",
        "Spring",
        "laravel",
        "beego",
        "fastapi",
        "ASP.NET Core",
        "hapi",
        "celery",
        "flask",
        "gin",
        "docker",
        "fastify",
        "nest",
        "symfony",
        "yii",
        "angular",
        "ember",
        "next.js",
        "nuxt",
        "preact",
        "react",
        "svelte",
        "vue",
        "another"
    ];

    let lower = value.to_lowercase();
    NAMES
        .into_iter()
        .find(|name| name.to_lowercase() == lower)
        .ok_or_else(|| TwcError::Api(t!("cli.app_invalid_framework", value => value).into_owned()))
}

/// Creates a new cloud app from a connected VCS repository.
///
/// # Overview
///
/// Builds a [`models::CreateApp`] request for either a `backend` app (built and
/// run from a git repository) or a `frontend` app (static/SSR site), submits it
/// to the Timeweb Cloud API, and prints the new app id and name.
///
/// A backend app requires `run_cmd`; a frontend app requires `index_dir`.
/// The request body is assembled as JSON and deserialised into the SDK model so
/// that provider/repository identifiers and the framework are validated by the
/// SDK's own `serde` definitions without any panic-prone conversions.
///
/// # Errors
///
/// Returns [`TwcError::Api`] when the app type or framework is invalid, when a
/// required field for the chosen type is missing, when the identifiers cannot
/// be parsed, or on any network or API failure.
pub async fn create(
    config: &Configuration,
    name: &str,
    comment: Option<&str>,
    provider_id: &str,
    repository_id: &str,
    preset_id: i64,
    app_type: &str,
    framework: &str,
    branch: &str,
    commit_sha: Option<&str>,
    build_cmd: Option<&str>,
    run_cmd: Option<&str>,
    index_dir: Option<&str>,
    is_auto_deploy: bool,
    project_id: Option<i64>,
    format: OutputFormat
) -> Result<(), TwcError> {
    let type_value = parse_app_type(app_type)?;
    let framework_value = parse_framework(framework)?;

    if type_value == "backend" && run_cmd.is_none() {
        return Err(TwcError::Api(
            t!("cli.app_backend_needs_run_cmd").into_owned()
        ));
    }
    if type_value == "frontend" && index_dir.is_none() {
        return Err(TwcError::Api(
            t!("cli.app_frontend_needs_index_dir").into_owned()
        ));
    }

    let mut body = serde_json::json!({
        "provider_id": provider_id,
        "type": type_value,
        "repository_id": repository_id,
        "build_cmd": build_cmd.unwrap_or_default(),
        "branch_name": branch,
        "is_auto_deploy": is_auto_deploy,
        "commit_sha": commit_sha.unwrap_or_default(),
        "name": name,
        "comment": comment.unwrap_or_default(),
        "preset_id": preset_id,
        "framework": framework_value
    });

    if let Some(map) = body.as_object_mut() {
        if let Some(cmd) = run_cmd {
            map.insert(
                "run_cmd".to_owned(),
                serde_json::Value::String(cmd.to_owned())
            );
        }
        if let Some(dir) = index_dir {
            map.insert(
                "index_dir".to_owned(),
                serde_json::Value::String(dir.to_owned())
            );
        }
        if let Some(project) = project_id {
            map.insert("project_id".to_owned(), serde_json::Value::from(project));
        }
    }

    let request: models::CreateApp = serde_json::from_value(body).map_err(|e| {
        TwcError::Api(t!("cli.app_invalid_input", value => e.to_string()).into_owned())
    })?;

    let resp = apps_api::create_app(config, request).await?;
    let app = &resp.app;

    match format {
        OutputFormat::Table => {
            println!(
                "{}",
                t!("cli.app_created", id => fmt_id(app.id), name => app.name.clone())
            );
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            if let Some(out) = crate::output::serialized(format, &resp.app) {
                println!("{}", out?);
            }
        }
        OutputFormat::Quiet => {
            println!("{}\t{}", fmt_id(app.id), app.name);
        }
    }
    Ok(())
}

/// Lower bound for log filtering resolved from `--since` / `--today`.
///
/// Accepts `YYYY-MM-DD` (interpreted as local midnight) or a full RFC 3339
/// timestamp. `--today` resolves to the current local midnight.
///
/// # Errors
///
/// Returns [`TwcError::Api`] when the value is neither a date nor an RFC 3339
/// timestamp.
fn resolve_since(since: Option<&str>, today: bool) -> Result<Option<DateTime<Local>>, TwcError> {
    if today {
        let midnight = Local::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .and_then(|naive| Local.from_local_datetime(&naive).single());
        return Ok(midnight);
    }
    let Some(raw) = since else {
        return Ok(None);
    };
    if let Ok(ts) = DateTime::parse_from_rfc3339(raw) {
        return Ok(Some(ts.with_timezone(&Local)));
    }
    if let Ok(date) = NaiveDate::parse_from_str(raw, "%Y-%m-%d") {
        let midnight = date
            .and_hms_opt(0, 0, 0)
            .and_then(|naive| Local.from_local_datetime(&naive).single());
        return Ok(midnight);
    }
    Err(TwcError::Api(
        t!("cli.app_invalid_since", value => raw).into_owned()
    ))
}

/// Removes ANSI CSI escape sequences from a log line.
///
/// App runtime logs arrive colorized (`ESC[2m2026-07-05T07:13:30Z…`), so the
/// leading timestamp is wrapped in styling sequences that must be dropped
/// before parsing.
fn strip_ansi(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            out.push(c);
            continue;
        }
        if chars.peek() == Some(&'[') {
            chars.next();
            for esc in chars.by_ref() {
                if esc.is_ascii_alphabetic() {
                    break;
                }
            }
        }
    }
    out
}

/// Timestamp parsed from a log line, if present.
///
/// Plain runtime lines start with an RFC 3339 timestamp (possibly wrapped in
/// ANSI styling). Structured JSON lines (`{"timestamp":"…"}`) carry it inside
/// the payload instead, so the first embedded RFC 3339 substring is used as a
/// fallback. Continuation lines (stack traces, wrapped output) have no
/// timestamp at all and return `None`.
fn line_timestamp(line: &str) -> Option<DateTime<FixedOffset>> {
    let clean = strip_ansi(line);
    clean
        .split_whitespace()
        .next()
        .and_then(|head| DateTime::parse_from_rfc3339(head).ok())
        .or_else(|| embedded_rfc3339(&clean))
}

/// Finds and parses the first RFC 3339 timestamp embedded anywhere in `line`.
///
/// Locates a `YYYY-MM-DDT` shaped anchor, extends it across the characters an
/// RFC 3339 timestamp may contain and hands the slice to the strict parser,
/// so arbitrary digits in the line cannot produce false positives.
fn embedded_rfc3339(line: &str) -> Option<DateTime<FixedOffset>> {
    let bytes = line.as_bytes();
    for (start, window) in bytes.windows(11).enumerate() {
        let anchored = window[4] == b'-'
            && window[7] == b'-'
            && window[10] == b'T'
            && window[..4].iter().all(u8::is_ascii_digit)
            && window[5..7].iter().all(u8::is_ascii_digit)
            && window[8..10].iter().all(u8::is_ascii_digit);
        if !anchored {
            continue;
        }
        let tail = &line[start..];
        let end = tail
            .find(|c: char| {
                !(c.is_ascii_digit() || matches!(c, '-' | ':' | '.' | 'T' | 'Z' | '+'))
            })
            .unwrap_or(tail.len());
        if let Ok(ts) = DateTime::parse_from_rfc3339(&tail[..end]) {
            return Some(ts);
        }
    }
    None
}

/// Timestamp of the most recent stamped line, scanning from the end.
fn last_line_timestamp(lines: &[String]) -> Option<DateTime<FixedOffset>> {
    lines.iter().rev().find_map(|line| line_timestamp(line))
}

/// Applies `since` and `tail` filters to raw log lines.
///
/// A line without its own timestamp inherits the timestamp of the closest
/// preceding stamped line, so multi-line entries stay intact. Lines seen
/// before any stamped line are kept only when no lower bound is set.
fn filter_log_lines(
    lines: &[String],
    since: Option<DateTime<Local>>,
    tail: Option<usize>
) -> Vec<String> {
    let mut kept: Vec<String> = Vec::with_capacity(lines.len());
    let mut current: Option<DateTime<Local>> = None;
    for line in lines {
        if let Some(ts) = line_timestamp(line) {
            current = Some(ts.with_timezone(&Local));
        }
        let keep = match since {
            None => true,
            Some(bound) => current.is_some_and(|ts| ts >= bound)
        };
        if keep {
            kept.push(line.clone());
        }
    }
    if let Some(n) = tail {
        let skip = kept.len().saturating_sub(n);
        kept.drain(..skip);
    }
    kept
}

/// Prints filtered log lines in the requested output format.
///
/// # Errors
///
/// Returns [`TwcError::Api`] when serialization fails.
fn print_log_lines(
    lines: &[String],
    empty_key: &str,
    format: OutputFormat
) -> Result<(), TwcError> {
    match format {
        OutputFormat::Table => {
            if lines.is_empty() {
                println!("{}", t!(empty_key));
            } else {
                for line in lines {
                    println!("{line}");
                }
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &lines)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for line in lines {
                println!("{line}");
            }
        }
    }
    Ok(())
}

/// Shows runtime logs of an app with optional date and tail filters.
///
/// # Overview
///
/// Fetches all runtime log lines of the app (the API has no server-side
/// pagination for app logs) and applies `--since`/`--today` and `--tail`
/// filters on the client before printing.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures, or when the
/// `--since` value cannot be parsed.
pub async fn logs(
    config: &Configuration,
    id: &str,
    tail: Option<usize>,
    since: Option<&str>,
    today: bool,
    format: OutputFormat
) -> Result<(), TwcError> {
    let bound = resolve_since(since, today)?;
    let resp = apps_api::get_app_logs(config, id).await?;
    let lines = filter_log_lines(&resp.app_logs, bound, tail);
    if lines.is_empty()
        && matches!(format, OutputFormat::Table)
        && let Some(bound) = bound
        && let Some(last) = last_line_timestamp(&resp.app_logs)
    {
        println!(
            "{}",
            t!(
                "cli.no_app_logs_since",
                since => bound.format("%Y-%m-%d %H:%M:%S").to_string(),
                last => last.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string()
            )
        );
        return Ok(());
    }
    print_log_lines(&lines, "cli.no_app_logs", format)
}

/// Compact row for the deploys table.
#[derive(Tabled)]
struct DeployRow {
    #[tabled(rename = "ID")]
    id:         String,
    #[tabled(rename = "Status")]
    status:     String,
    #[tabled(rename = "Commit")]
    commit:     String,
    #[tabled(rename = "Message")]
    message:    String,
    #[tabled(rename = "Started")]
    started_at: String,
    #[tabled(rename = "Ended")]
    ended_at:   String
}

/// Lists deploys of an app, newest first.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures.
pub async fn list_deploys(
    config: &Configuration,
    id: &str,
    format: OutputFormat
) -> Result<(), TwcError> {
    let resp = apps_api::get_app_deploys(config, id, None, None).await?;
    let mut deploys = resp.deploys.clone().unwrap_or_default();
    deploys.sort_by(|a, b| b.started_at.cmp(&a.started_at));

    let rows: Vec<DeployRow> = deploys
        .iter()
        .map(|d| DeployRow {
            id:         d.id.to_string(),
            status:     format!("{:?}", d.status),
            commit:     d.commit_sha.chars().take(10).collect(),
            message:    d.commit_msg.lines().next().unwrap_or_default().to_owned(),
            started_at: d.started_at.clone(),
            ended_at:   d.ended_at.clone().unwrap_or_default()
        })
        .collect();

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", t!("cli.no_deploys"));
            } else {
                println!("{}", crate::output::render_table(&rows));
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => {
            let out = crate::output::serialized(format, &deploys)
                .transpose()?
                .unwrap_or_default();
            println!("{out}");
        }
        OutputFormat::Quiet => {
            for d in &deploys {
                println!("{}\t{:?}", d.id, d.status);
            }
        }
    }
    Ok(())
}

/// Shows build/deploy logs of a deploy, defaulting to the most recent one.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network or API failures, or when the app has
/// no deploys to default to.
pub async fn deploy_logs(
    config: &Configuration,
    id: &str,
    deploy_id: Option<&str>,
    debug: bool,
    format: OutputFormat
) -> Result<(), TwcError> {
    let target = if let Some(explicit) = deploy_id {
        explicit.to_owned()
    } else {
        let resp = apps_api::get_app_deploys(config, id, None, None).await?;
        resp.deploys
            .unwrap_or_default()
            .iter()
            .max_by(|a, b| a.started_at.cmp(&b.started_at))
            .map(|d| d.id.to_string())
            .ok_or_else(|| TwcError::Api(t!("cli.no_deploys").into_owned()))?
    };
    let resp = apps_api::get_deploy_logs(config, id, &target, Some(debug)).await?;
    print_log_lines(&resp.deploy_logs, "cli.no_deploy_logs", format)
}

#[cfg(test)]
mod tests;
