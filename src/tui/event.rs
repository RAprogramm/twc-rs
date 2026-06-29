// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Event types and async event loop for the TUI dashboard.

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use tokio::{sync::mpsc, time::Duration};

use super::app::{App, Focus, NavLevel};

/// Events that the TUI event loop can process.
#[allow(dead_code)]
#[derive(Debug)]
pub enum AppEvent {
    /// A keyboard event from crossterm.
    Key(KeyEvent),
    /// Timer tick — time to poll API or re-render.
    Tick,
    /// Terminal resize.
    // JUSTIFY: Resize events are sent by the event loop but values are unused.
    Resize(u16, u16),
    /// An error message to display.
    // JUSTIFY: Error events are part of the public event API.
    Error(String),
    /// A status message to display.
    // JUSTIFY: Status events are part of the public event API.
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
    if app.action_menu_open() {
        match key.code {
            KeyCode::Char('k') | KeyCode::Up => app.menu_previous(),
            KeyCode::Char('j') | KeyCode::Down => app.menu_next(),
            KeyCode::Enter => app.menu_select(),
            KeyCode::Esc | KeyCode::Char('q') => app.close_action_menu(),
            _ => {}
        }
        return true;
    }

    if app.awaiting_confirm() {
        match key.code {
            KeyCode::Char('y' | 'Y') | KeyCode::Enter => app.confirm_action(),
            _ => app.cancel_action()
        }
        return true;
    }

    if app.show_help {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?') => {
                app.show_help = false;
            }
            _ => {}
        }
        return true;
    }

    match key.code {
        KeyCode::Char('Q') => {
            app.quit();
            return false;
        }
        KeyCode::Char('?') => {
            app.toggle_help();
            return true;
        }
        KeyCode::Char('r') => {
            app.force_refresh();
            return true;
        }
        KeyCode::Tab => {
            app.next_tab();
            return true;
        }
        _ => {}
    }

    if matches!(key.code, KeyCode::Enter)
        && app.focus == Focus::ResourceList
        && app.selected_server().is_some()
    {
        app.open_action_menu();
        return true;
    }

    match app.nav_level {
        NavLevel::Overview => handle_overview_key(app, key),
        NavLevel::Inner => handle_inner_key(app, key)
    }
    true
}

const fn handle_overview_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('h') | KeyCode::Left => {
            app.focus = app.focus.left();
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.focus = app.focus.right();
        }
        KeyCode::Char('j' | 'k') | KeyCode::Down | KeyCode::Up | KeyCode::Enter => {
            app.nav_level = NavLevel::Inner;
        }
        _ => {}
    }
}

fn handle_inner_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.nav_level = NavLevel::Overview;
        }
        KeyCode::Char('h') | KeyCode::Left => {
            app.nav_level = NavLevel::Overview;
            app.focus = app.focus.left();
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.nav_level = NavLevel::Overview;
            app.focus = app.focus.right();
        }
        KeyCode::Char('k') | KeyCode::Up => match app.focus {
            Focus::ResourceTabs => {
                app.active_tab = app.active_tab.previous();
                app.selected = 0;
            }
            Focus::ResourceList => app.select_previous(),
            Focus::Details => {}
        },
        KeyCode::Char('j') | KeyCode::Down => match app.focus {
            Focus::ResourceTabs => {
                app.active_tab = app.active_tab.next();
                app.selected = 0;
            }
            Focus::ResourceList => app.select_next(),
            Focus::Details => {}
        },
        KeyCode::Enter => {
            if app.focus == Focus::ResourceList {
                app.focus = Focus::Details;
            }
        }
        KeyCode::Char('g') => {
            app.selected = 0;
        }
        KeyCode::Char('$') => {
            let len = app.current_list_len();
            if len > 0 {
                app.selected = len - 1;
            }
        }
        _ => {}
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
            Ok(Some(Event::Key(key))) if tx.send(AppEvent::Key(key)).is_err() => {
                break;
            }
            Ok(Some(Event::Resize(w, h))) if tx.send(AppEvent::Resize(w, h)).is_err() => {
                break;
            }
            _ => {}
        }
        if tx.send(AppEvent::Tick).is_err() {
            break;
        }
    }
}
