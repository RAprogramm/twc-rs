// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Event types and async event loop for the TUI dashboard.

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use tokio::{sync::mpsc, time::Duration};

use super::app::App;

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
    if app.show_help {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?' | 'q') => {
                app.show_help = false;
            }
            _ => {}
        }
        return true;
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.quit();
            false
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_previous();
            true
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next();
            true
        }
        KeyCode::Left | KeyCode::Char('h') => {
            app.select_previous();
            true
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.select_next();
            true
        }
        KeyCode::Tab => {
            app.next_tab();
            true
        }
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
        KeyCode::Char('r') => {
            app.force_refresh();
            true
        }
        KeyCode::Char('?') => {
            app.toggle_help();
            true
        }
        KeyCode::Enter => true,
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
