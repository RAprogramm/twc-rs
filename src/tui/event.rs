// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Event types and async event loop for the TUI dashboard.

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use tokio::{sync::mpsc, time::Duration};

use super::app::{App, Focus};

/// Events that the TUI event loop can process.
#[derive(Debug)]
#[allow(dead_code)]
pub enum AppEvent {
    /// A keyboard event from crossterm.
    Key(KeyEvent),
    /// Timer tick — time to poll API or re-render.
    Tick,
    /// Terminal resize.
    Resize(u16, u16),
    /// An error message to display.
    Error(String),
    /// A status message to display.
    Status(String)
}

/// Processes a single [`AppEvent`] and updates the [`App`] state.
///
/// Returns `false` when the app should quit.
pub fn handle_event(app: &mut App, event: AppEvent) -> bool {
    match event {
        AppEvent::Tick | AppEvent::Resize(_, _) => true,
        AppEvent::Key(key) => handle_key(app, key),
        AppEvent::Error(msg) => {
            app.error_message = Some(msg);
            true
        }
        AppEvent::Status(msg) => {
            app.status_message = Some(msg);
            true
        }
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    // Handle help overlay first
    if app.show_help {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?') => {
                app.show_help = false;
            }
            _ => {}
        }
        return true;
    }

    // Handle detail popup
    if app.detail_popup {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.detail_popup = false;
            }
            _ => {}
        }
        return true;
    }

    // Main navigation
    match key.code {
        // Quit only with Shift+Q
        KeyCode::Char('Q') => {
            app.quit();
            false
        }
        // Help
        KeyCode::Char('?') => {
            app.toggle_help();
            true
        }
        // Refresh
        KeyCode::Char('r') => {
            app.force_refresh();
            true
        }
        // Vim navigation between components
        KeyCode::Char('h') | KeyCode::Left => {
            // Move focus left/up
            match app.focus {
                Focus::Details => app.focus = Focus::ResourceList,
                Focus::ProjectTabs => app.focus = Focus::ResourceTabs,
                Focus::ResourceTabs => {} // Stay at top
                Focus::ResourceList => {} // Stay at left
            }
            true
        }
        KeyCode::Char('l') | KeyCode::Right => {
            // Move focus right/down
            match app.focus {
                Focus::ResourceTabs => app.focus = Focus::ProjectTabs,
                Focus::ResourceList => app.focus = Focus::Details,
                Focus::ProjectTabs => {} // Stay at right
                Focus::Details => {} // Stay at right
            }
            true
        }
        KeyCode::Char('k') | KeyCode::Up => {
            // Move selection up in current focus
            match app.focus {
                Focus::ResourceList | Focus::Details => {
                    app.select_previous();
                }
                Focus::ResourceTabs | Focus::ProjectTabs => {
                    // Cycle tabs up
                    app.active_tab = match app.active_tab {
                        crate::tui::app::ResourceTab::Finances => crate::tui::app::ResourceTab::Servers,
                        _ => app.active_tab.next()
                    };
                    app.selected = 0;
                }
            }
            true
        }
        KeyCode::Char('j') | KeyCode::Down => {
            // Move selection down in current focus
            match app.focus {
                Focus::ResourceList | Focus::Details => {
                    app.select_next();
                }
                Focus::ResourceTabs | Focus::ProjectTabs => {
                    // Cycle tabs down
                    app.selected = 0;
                }
            }
            true
        }
        // Enter to open detail popup
        KeyCode::Enter => {
            app.detail_popup = true;
            true
        }
        // Go to first/last
        KeyCode::Char('g') => {
            app.selected = 0;
            true
        }
        KeyCode::Char('$') => {
            if app.current_list_len() > 0 {
                app.selected = app.current_list_len() - 1;
            }
            true
        }
        // Tab to cycle between resource tabs
        KeyCode::Tab => {
            app.next_tab();
            true
        }
        _ => true
    }
}

/// Runs the async event loop, sending [`AppEvent`]s through the channel.
///
/// Stops when the receiver is dropped (sender returns `Err`).
pub async fn run_event_loop(tx: mpsc::UnboundedSender<AppEvent>) {
    let tick_rate = Duration::from_millis(100);

    loop {
        let evt = tokio::task::spawn_blocking(move || {
            event::poll(tick_rate)
                .ok()
                .and_then(|ready| if ready { event::read().ok() } else { None })
        })
        .await;

        match evt {
            Ok(Some(Event::Key(key))) => {
                if tx.send(AppEvent::Key(key)).is_err() {
                    break;
                }
            }
            Ok(Some(Event::Resize(w, h))) => {
                if tx.send(AppEvent::Resize(w, h)).is_err() {
                    break;
                }
            }
            _ => {}
        }
        if tx.send(AppEvent::Tick).is_err() {
            break;
        }
    }
}
