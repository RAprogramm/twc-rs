// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Self-update: latest-release check plus per-channel upgrade logic.

use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
    time::Duration
};

use rust_i18n::t;

use crate::error::TwcError;

/// GitHub repository queried for the latest release.
const REPO: &str = "RAprogramm/twc-rs";

/// Hard deadline for the release-metadata request.
const HTTP_TIMEOUT: Duration = Duration::from_secs(10);

/// How the running binary was installed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallChannel {
    /// Arch package (AUR `twc-rs-bin`), owned by pacman.
    Pacman,
    /// `cargo install` into `~/.cargo/bin`.
    Cargo,
    /// Debian/Ubuntu `.deb` package, owned by dpkg.
    Deb,
    /// The `install.sh` script (`~/.local/bin` or `/usr/local/bin`).
    Installer,
    /// Anything else (manual copy, Windows archive, ...).
    Unknown
}

/// Checks the latest release and updates through the detected channel.
///
/// With `check_only` the command reports the versions and the exact update
/// command without running anything.
///
/// # Errors
///
/// Returns [`TwcError::Api`] when the release metadata cannot be fetched or
/// the spawned update command fails to start.
pub async fn run(check_only: bool) -> Result<(), TwcError> {
    let current = env!("CARGO_PKG_VERSION");
    let latest = fetch_latest_version().await?;
    println!(
        "{}",
        t!("cli.update_current", current => current, latest => latest.as_str())
    );
    if !is_newer(&latest, current) {
        println!("{}", t!("cli.update_up_to_date"));
        return Ok(());
    }
    println!(
        "{}",
        t!("cli.update_available", current => current, latest => latest.as_str())
    );

    let exe = env::current_exe()
        .and_then(std::fs::canonicalize)
        .map_err(|e| TwcError::Io(e.to_string()))?;
    let channel = detect_channel(&exe);
    println!(
        "{}",
        t!("cli.update_channel", channel => format!("{channel:?}"))
    );

    match update_plan(channel, &latest, &exe) {
        UpdatePlan::Run(argv) => {
            let shown = argv.join(" ");
            if check_only {
                println!("{}", t!("cli.update_do", command => shown.as_str()));
                return Ok(());
            }
            println!("{}", t!("cli.update_running", command => shown.as_str()));
            let status = Command::new(&argv[0])
                .args(&argv[1..])
                .status()
                .map_err(|e| TwcError::Io(e.to_string()))?;
            if status.success() {
                println!("{}", t!("cli.update_done"));
            } else {
                return Err(TwcError::Api(
                    t!("cli.update_failed", command => shown.as_str()).into_owned()
                ));
            }
        }
        UpdatePlan::Instruct(message) => println!("{message}")
    }
    Ok(())
}

/// What `update` should do for a given channel.
enum UpdatePlan {
    /// Spawn this argv with inherited stdio.
    Run(Vec<String>),
    /// Print this localized instruction and stop.
    Instruct(String)
}

/// Picks the update action for the detected channel.
fn update_plan(channel: InstallChannel, latest: &str, exe: &Path) -> UpdatePlan {
    match channel {
        InstallChannel::Pacman => aur_helper().map_or_else(
            || UpdatePlan::Instruct(t!("cli.update_no_aur_helper").into_owned()),
            |helper| UpdatePlan::Run(vec![helper, "-Sy".to_owned(), "twc-rs-bin".to_owned()])
        ),
        InstallChannel::Cargo => UpdatePlan::Run(vec![
            "cargo".to_owned(),
            "install".to_owned(),
            "twc-rs".to_owned(),
        ]),
        InstallChannel::Installer => UpdatePlan::Run(vec![
            "sh".to_owned(),
            "-c".to_owned(),
            format!("curl -fsSL https://raw.githubusercontent.com/{REPO}/main/install.sh | sh"),
        ]),
        InstallChannel::Deb => {
            UpdatePlan::Instruct(t!("cli.update_instruction_deb", latest => latest).into_owned())
        }
        InstallChannel::Unknown => UpdatePlan::Instruct(
            t!("cli.update_instruction_unknown", path => exe.display().to_string()).into_owned()
        )
    }
}

/// First AUR helper found in `PATH`, if any.
fn aur_helper() -> Option<String> {
    ["paru", "yay"]
        .into_iter()
        .find(|helper| which(helper))
        .map(ToOwned::to_owned)
}

/// Returns `true` when `binary` resolves in `PATH`.
fn which(binary: &str) -> bool {
    env::var_os("PATH")
        .is_some_and(|paths| env::split_paths(&paths).any(|dir| dir.join(binary).is_file()))
}

/// Detects the install channel of the running executable.
fn detect_channel(exe: &Path) -> InstallChannel {
    if owned_by(exe, "pacman", &["-Qo"]) {
        return InstallChannel::Pacman;
    }
    if owned_by(exe, "dpkg", &["-S"]) {
        return InstallChannel::Deb;
    }
    channel_from_path(exe, env::var_os("HOME").map(PathBuf::from).as_deref())
}

/// Returns `true` when the package `manager` claims ownership of `exe`.
fn owned_by(exe: &Path, manager: &str, args: &[&str]) -> bool {
    Command::new(manager)
        .args(args)
        .arg(exe)
        .output()
        .is_ok_and(|out| out.status.success())
}

/// Path-only channel detection, separated from ownership probes for testing.
fn channel_from_path(exe: &Path, home: Option<&Path>) -> InstallChannel {
    if let Some(home) = home {
        if exe.starts_with(home.join(".cargo").join("bin")) {
            return InstallChannel::Cargo;
        }
        if exe.starts_with(home.join(".local").join("bin")) {
            return InstallChannel::Installer;
        }
    }
    if exe.starts_with("/usr/local/bin") {
        return InstallChannel::Installer;
    }
    InstallChannel::Unknown
}

/// Fetches the latest release tag (without the `v` prefix) from GitHub.
///
/// # Errors
///
/// Returns [`TwcError::Api`] on network failures or an unexpected payload.
async fn fetch_latest_version() -> Result<String, TwcError> {
    let response = reqwest::Client::new()
        .get(format!(
            "https://api.github.com/repos/{REPO}/releases/latest"
        ))
        .header("User-Agent", concat!("twc-rs/", env!("CARGO_PKG_VERSION")))
        .header("Accept", "application/vnd.github+json")
        .timeout(HTTP_TIMEOUT)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|e| TwcError::Api(e.to_string()))?;
    let payload: serde_json::Value = response
        .json()
        .await
        .map_err(|e| TwcError::Api(e.to_string()))?;
    payload
        .get("tag_name")
        .and_then(serde_json::Value::as_str)
        .map(|tag| tag.trim_start_matches('v').to_owned())
        .ok_or_else(|| TwcError::Api("release payload without tag_name".to_owned()))
}

/// Parses `x.y.z` (an optional leading `v` is ignored).
fn parse_version(version: &str) -> Option<(u64, u64, u64)> {
    let mut parts = version.trim().trim_start_matches('v').splitn(3, '.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    Some((major, minor, patch))
}

/// Returns `true` when `latest` is strictly newer than `current`.
fn is_newer(latest: &str, current: &str) -> bool {
    match (parse_version(latest), parse_version(current)) {
        (Some(l), Some(c)) => l > c,
        _ => false
    }
}

#[cfg(test)]
mod tests;
