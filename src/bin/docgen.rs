// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Generates the mdBook CLI reference from the live clap command tree.
//!
//! Run as `cargo run --bin docgen -- <book-src-dir>`; it (re)writes
//! `cli/index.md` plus one `cli/<group>.md` page per top-level command and
//! the book `SUMMARY.md`, so the published reference can never drift from
//! the code.

use std::{env, fmt::Write as _, fs, io::Write, path::Path};

use clap::CommandFactory;
use twc_rs::cli::Cli;

/// Static guide pages listed in `SUMMARY.md` ahead of the generated
/// reference, as `(title, file)` pairs.
const GUIDE_PAGES: &[(&str, &str)] = &[
    ("Introduction", "introduction.md"),
    ("Installation", "install.md"),
    ("Authentication", "authentication.md"),
    ("Usage", "usage.md"),
    ("Dashboard (TUI)", "dashboard.md"),
    ("Shell completions", "completions.md"),
    ("Comparison with the official CLI", "comparison.md")
];

fn main() -> std::io::Result<()> {
    let src = env::args()
        .nth(1)
        .unwrap_or_else(|| "docs/book/src".to_owned());
    let src = Path::new(&src);
    let cli_dir = src.join("cli");
    fs::create_dir_all(&cli_dir)?;

    let mut cmd = Cli::command();
    cmd.build();

    let mut index = String::from("# CLI reference\n\n");
    index.push_str(
        "Generated from the `clap` command definitions by `cargo run --bin \
         docgen`; do not edit by hand.\n\n| Command | Description |\n|---|---|\n"
    );
    let mut summary_cli = String::new();

    for sub in cmd.get_subcommands().filter(|s| s.get_name() != "help") {
        let name = sub.get_name().to_owned();
        let about = sub.get_about().map(ToString::to_string).unwrap_or_default();
        _ = writeln!(index, "| [`{name}`]({name}.md) | {about} |");
        _ = writeln!(summary_cli, "  - [{name}](cli/{name}.md)");
        let page = render_group(sub);
        fs::write(cli_dir.join(format!("{name}.md")), page)?;
    }
    fs::write(cli_dir.join("index.md"), index)?;

    let mut summary = String::from("# Summary\n\n");
    for (title, file) in GUIDE_PAGES {
        _ = writeln!(summary, "- [{title}]({file})");
    }
    summary.push_str("- [CLI reference](cli/index.md)\n");
    summary.push_str(&summary_cli);
    fs::write(src.join("SUMMARY.md"), summary)?;

    writeln!(std::io::stdout(), "docgen: wrote {}", cli_dir.display())
}

/// Renders one command group (or leaf command) as a markdown page.
fn render_group(cmd: &clap::Command) -> String {
    let name = cmd.get_name();
    let mut page = format!("# `twc-rs {name}`\n\n");
    if let Some(about) = cmd.get_about() {
        _ = write!(page, "{about}\n\n");
    }
    let leaves: Vec<&clap::Command> = cmd
        .get_subcommands()
        .filter(|s| s.get_name() != "help")
        .collect();
    if leaves.is_empty() {
        render_leaf(&mut page, cmd, name, None);
        return page;
    }
    page.push_str("| Subcommand | Description |\n|---|---|\n");
    for leaf in &leaves {
        let about = leaf
            .get_about()
            .map(ToString::to_string)
            .unwrap_or_default();
        _ = writeln!(
            page,
            "| [`{sub}`](#twc-rs-{name}-{sub}) | {about} |",
            sub = leaf.get_name()
        );
    }
    page.push('\n');
    for leaf in &leaves {
        render_leaf(&mut page, leaf, leaf.get_name(), Some(name));
    }
    page
}

/// Appends usage and an argument table for a single leaf command.
fn render_leaf(page: &mut String, cmd: &clap::Command, name: &str, group: Option<&str>) {
    if let Some(group) = group {
        _ = write!(page, "## `twc-rs {group} {name}`\n\n");
        if let Some(about) = cmd.get_about() {
            _ = write!(page, "{about}\n\n");
        }
    }
    let mut rows = String::new();
    for arg in cmd.get_arguments() {
        if matches!(arg.get_id().as_str(), "help" | "version") {
            continue;
        }
        let mut flag = arg.get_long().map_or_else(
            || format!("`<{}>`", arg.get_id().as_str().to_uppercase()),
            |l| format!("`--{l}`")
        );
        if let Some(short) = arg.get_short() {
            flag = format!("`-{short}`, {flag}");
        }
        let help = arg
            .get_help()
            .map(ToString::to_string)
            .unwrap_or_default()
            .replace('\n', " ");
        let env = arg
            .get_env()
            .and_then(|e| e.to_str())
            .map(|e| format!("`{e}`"))
            .unwrap_or_default();
        let default = arg
            .get_default_values()
            .first()
            .and_then(|v| v.to_str())
            .map(|v| format!("`{v}`"))
            .unwrap_or_default();
        let required = if arg.is_required_set() { "yes" } else { "" };
        _ = writeln!(rows, "| {flag} | {help} | {required} | {env} | {default} |");
    }
    if rows.is_empty() {
        page.push_str("This command takes no arguments beyond the global flags.\n\n");
    } else {
        page.push_str(
            "| Argument | Description | Required | Env | Default |\n|---|---|---|---|---|\n"
        );
        page.push_str(&rows);
        page.push('\n');
    }
}
