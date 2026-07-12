// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Interactive dashboard runtime: the event loop and preference persistence.

mod actions;
mod refresh;
mod splash;
mod stats;

use timeweb_rs::authenticated;

use self::{
    actions::{fetch_drill, perform_action, perform_create},
    refresh::{spawn_one_shot_refresh, spawn_refresh_loop},
    splash::draw_splash,
    stats::spawn_stats_fetch
};
use crate::{config::AppConfig, error::TwcError, tui};

fn persist_dashboard_prefs(app: &tui::app::App) {
    let Ok(mut cfg) = AppConfig::load() else {
        return;
    };
    cfg.theme = app.theme;
    cfg.language = app.language;
    cfg.dashboard.hidden_widgets = app.hidden_widget_ids();
    cfg.dashboard.list_width_pct = app.list_width_pct;
    cfg.dashboard.hide_empty_tabs = app.hide_empty_tabs;
    let _ = cfg.save();
}

pub(crate) async fn run_dashboard(
    mut token: String,
    interval: u64,
    theme: crate::tui::themes::Theme,
    prefs: crate::config::DashboardPrefs,
    profile: Option<String>
) -> Result<(), TwcError> {
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

    let mut app = tui::app::App::new_with_theme(interval, theme, Some(token.clone()));
    app.apply_prefs(
        &prefs.hidden_widgets,
        prefs.list_width_pct,
        prefs.hide_empty_tabs
    );
    if let Ok(cfg) = AppConfig::load() {
        app.language = cfg.language;
        let mut names = vec!["default".to_string()];
        let mut profile_names: Vec<String> = cfg.profiles.keys().cloned().collect();
        profile_names.sort();
        names.extend(profile_names);
        app.profiles = names;
    }
    app.active_profile = profile.unwrap_or_else(|| "default".to_string());
    app.is_loading = true;
    draw_splash(&mut terminal);

    let (tx, mut rx) = mpsc::unbounded_channel();
    let event_tx = tx.clone();

    tokio::spawn(async move {
        tui::event::run_event_loop(event_tx).await;
    });

    let mut refresh_handle = spawn_refresh_loop(tx.clone(), token.clone(), theme, interval);

    while let Some(event) = rx.recv().await {
        if !tui::event::handle_event(&mut app, event) {
            break;
        }

        if let Ok(size) = terminal.size() {
            let sidebar_w = tui::widgets::sidebar::width_for(size.width, app.nav_longest_label());
            let content_w = size.width.saturating_sub(sidebar_w).saturating_sub(2);
            app.resource_cols =
                tui::widgets::card_grid::columns(content_w, app.content_longest_label());
        }

        terminal
            .draw(|f| tui::ui::draw(f, &app))
            .map_err(|e| TwcError::Io(e.to_string()))?;

        if app.prefs_dirty {
            persist_dashboard_prefs(&app);
            app.prefs_dirty = false;
        }

        if app.refresh_requested {
            app.refresh_requested = false;
            spawn_one_shot_refresh(tx.clone(), token.clone(), theme, interval);
        }

        if let Some((drill_tab, drill_id, drill_name)) = app.take_drill_request() {
            use tui::app::LogLevel;
            let config = authenticated(token.clone());
            match fetch_drill(&config, drill_tab, drill_id, &drill_name).await {
                Ok(view) => {
                    app.log(LogLevel::Info, format!("opened {drill_name}"));
                    app.open_drill(view);
                }
                Err(e) => {
                    app.close_drill();
                    app.log(LogLevel::Error, format!("open {drill_name} failed: {e}"));
                }
            }
        }

        if let Some(action) = app.take_dispatch() {
            use tui::app::LogLevel;
            app.log(
                LogLevel::Info,
                format!("{} {}", action.kind.label(), action.resource_name)
            );
            let config = authenticated(token.clone());
            perform_action(&config, &mut app, action).await;
            spawn_one_shot_refresh(tx.clone(), token.clone(), theme, interval);
        }

        if let Some(form) = app.take_create_request() {
            let config = authenticated(token.clone());
            perform_create(&config, &mut app, form).await;
            spawn_one_shot_refresh(tx.clone(), token.clone(), theme, interval);
        }

        if let Some(profile) = app.take_switch_profile() {
            use tui::app::LogLevel;
            let lookup = (profile != "default").then_some(profile.as_str());
            match AppConfig::load().and_then(|c| c.token_for(lookup)) {
                Ok(Some(new_token)) => {
                    token = new_token;
                    refresh_handle.abort();
                    refresh_handle =
                        spawn_refresh_loop(tx.clone(), token.clone(), theme, interval);
                    app.token = Some(token.clone());
                    app.active_profile.clone_from(&profile);
                    app.is_loading = true;
                    app.log(
                        LogLevel::Success,
                        format!("switched to profile '{profile}'")
                    );
                    app.status_message = Some(format!("Profile: {profile}"));
                }
                _ => {
                    app.log(
                        LogLevel::Error,
                        format!("profile '{profile}' has no token configured")
                    );
                }
            }
        }

        if let Some(req) = app.poll_stats_request() {
            spawn_stats_fetch(tx.clone(), token.clone(), req);
        }
    }

    drop(tx);

    disable_raw_mode().map_err(|e| TwcError::Io(e.to_string()))?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .map_err(|e| TwcError::Io(e.to_string()))?;
    terminal
        .show_cursor()
        .map_err(|e| TwcError::Io(e.to_string()))?;
    Ok(())
}
