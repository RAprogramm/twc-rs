// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

#[cfg(feature = "auth")]
mod auth;
mod cli;
mod commands;
mod config;
mod error;
mod output;
#[cfg(feature = "tui")]
mod tui;

use clap::Parser;
use cli::{
    AuthCommands, Cli, Commands, ConfigCommands, ProjectCommands, ServerCommands, SshCommands
};
use config::AppConfig;
use error::TwcError;
use output::OutputFormat;
use timeweb_rs::authenticated;

/// Resolves the API token from CLI flag, environment, or config file.
///
/// # Overview
///
/// Priority order: `--token` flag > `TWC_TOKEN` env var >
/// config file (`~/.config/twc-rs/config.toml`).
///
/// # Errors
///
/// Returns [`TwcError::TokenMissing`] when no token is found.
fn resolve_token(cli_token: Option<&str>) -> Result<String, TwcError> {
    if let Some(token) = cli_token {
        return Ok(token.to_string());
    }

    let app_config = AppConfig::load()?;
    if let Some(token) = app_config.token {
        return Ok(token);
    }

    Err(TwcError::TokenMissing)
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), TwcError> {
    let cli = Cli::parse();
    let format = OutputFormat::parse(&cli.format).map_err(TwcError::Api)?;

    match cli.command {
        Commands::Config(cmd) => match cmd {
            ConfigCommands::Show => {
                let cfg = AppConfig::load()?;
                let has_token = cfg.token.is_some();
                println!(
                    "Config path: {}",
                    AppConfig::path()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default()
                );
                println!("Token set:   {has_token}");
                Ok(())
            }
            ConfigCommands::SetToken {
                token
            } => {
                let mut cfg = AppConfig::load()?;
                cfg.token = Some(token);
                cfg.save()?;
                println!("Token saved.");
                Ok(())
            }
        },
        Commands::Server(cmd) => {
            let token = resolve_token(cli.token.as_deref())?;
            let config = authenticated(token);
            match cmd {
                ServerCommands::List {
                    limit,
                    offset
                } => commands::servers::list(&config, limit, offset, format).await,
                ServerCommands::Info {
                    id
                } => commands::servers::info(&config, id, format).await,
                ServerCommands::Delete {
                    id
                } => commands::servers::delete(&config, id).await,
                ServerCommands::Reboot {
                    id
                } => commands::servers::reboot(&config, id).await
            }
        }
        Commands::Ssh(cmd) => {
            let token = resolve_token(cli.token.as_deref())?;
            let config = authenticated(token);
            match cmd {
                SshCommands::List => commands::ssh_keys::list(&config, format).await,
                SshCommands::Add {
                    name,
                    file,
                    default
                } => commands::ssh_keys::add(&config, &name, file.as_deref(), default).await,
                SshCommands::Delete {
                    id
                } => commands::ssh_keys::delete(&config, id).await
            }
        }
        Commands::Project(cmd) => {
            let token = resolve_token(cli.token.as_deref())?;
            let config = authenticated(token);
            match cmd {
                ProjectCommands::List => commands::projects::list(&config, format).await,
                ProjectCommands::Create {
                    name,
                    description
                } => commands::projects::create(&config, &name, description.as_deref()).await,
                ProjectCommands::Delete {
                    id
                } => commands::projects::delete(&config, id).await
            }
        }
        #[cfg(feature = "auth")]
        Commands::Auth(cmd) => {
            let config_path = config::AppConfig::path()?;
            match cmd {
                AuthCommands::Flow => {
                    auth::run_auth_flow(&config_path).map_err(|e| TwcError::Api(e.to_string()))
                }
                AuthCommands::Status => {
                    auth::show_status(&config_path).map_err(|e| TwcError::Api(e.to_string()))
                }
                AuthCommands::Logout => {
                    auth::logout(&config_path).map_err(|e| TwcError::Api(e.to_string()))
                }
                AuthCommands::Token {
                    token
                } => auth::accept_token_direct(&token, &config_path)
                    .map_err(|e| TwcError::Api(e.to_string()))
            }
        }
        #[cfg(not(feature = "auth"))]
        Commands::Auth(_) => {
            eprintln!(
                "Error: auth feature not enabled. \
                 Rebuild with --features auth"
            );
            std::process::exit(1);
        }
        #[cfg(feature = "tui")]
        Commands::Monitor {
            interval
        } => {
            let token = resolve_token(cli.token.as_deref())?;
            run_tui(token, interval).await
        }
        #[cfg(not(feature = "tui"))]
        Commands::Monitor {
            ..
        } => {
            eprintln!(
                "Error: tui feature not enabled. \
                 Rebuild with --features tui"
            );
            std::process::exit(1);
        }
    }
}

#[cfg(feature = "tui")]
async fn run_tui(token: String, interval: u64) -> Result<(), TwcError> {
    use crossterm::{
        execute,
        terminal::{
            EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode
        }
    };
    use ratatui::{Terminal, backend::CrosstermBackend};
    use tokio::sync::mpsc;

    enable_raw_mode().map_err(|e| TwcError::Io(e.to_string()))?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| TwcError::Io(e.to_string()))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| TwcError::Io(e.to_string()))?;

    let mut app = tui::app::App::new(interval);

    let (tx, mut rx) = mpsc::unbounded_channel();
    let event_tx = tx.clone();

    tokio::spawn(async move {
        tui::event::run_event_loop(event_tx).await;
    });

    // Initial server fetch
    let config = authenticated(token.clone());
    if let Ok(resp) = timeweb_rs::apis::servers_api::get_servers(&config, None, None).await {
        let summaries: Vec<tui::app::ServerSummary> = resp
            .servers
            .iter()
            .map(|s| tui::app::ServerSummary {
                id:        s.id as i32,
                name:      s.name.clone(),
                status:    format!("{:?}", s.status),
                cpu_count: s.cpu as i32,
                ram_mb:    s.ram as i32,
                disk_gb:   0
            })
            .collect();
        app.update_servers(summaries);
    }

    loop {
        terminal
            .draw(|f| tui::ui::draw(f, &app))
            .map_err(|e| TwcError::Io(e.to_string()))?;

        if let Some(event) = rx.recv().await {
            if !tui::event::handle_event(&mut app, event) {
                break;
            }

            if app.needs_refresh() {
                let config = authenticated(token.clone());
                match timeweb_rs::apis::servers_api::get_servers(&config, None, None).await {
                    Ok(resp) => {
                        let summaries: Vec<tui::app::ServerSummary> = resp
                            .servers
                            .iter()
                            .map(|s| tui::app::ServerSummary {
                                id:        s.id as i32,
                                name:      s.name.clone(),
                                status:    format!("{:?}", s.status),
                                cpu_count: s.cpu as i32,
                                ram_mb:    s.ram as i32,
                                disk_gb:   0
                            })
                            .collect();
                        app.update_servers(summaries);
                    }
                    Err(e) => {
                        app.error_message = Some(format!("API error: {e}"));
                    }
                }
            }
        }
    }

    disable_raw_mode().map_err(|e| TwcError::Io(e.to_string()))?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .map_err(|e| TwcError::Io(e.to_string()))?;
    terminal
        .show_cursor()
        .map_err(|e| TwcError::Io(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_token_cli_flag_takes_priority() {
        let result = resolve_token(Some("my-token"));
        assert_eq!(result.unwrap(), "my-token");
    }

    #[test]
    fn resolve_token_missing_without_config() {
        let dir = tempfile::tempdir().unwrap();
        let orig = std::env::var("XDG_CONFIG_HOME").ok();
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", dir.path());
        }
        let result = resolve_token(None);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("no API token configured"));
        unsafe {
            match orig {
                Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
                None => std::env::remove_var("XDG_CONFIG_HOME")
            }
        }
    }
}
