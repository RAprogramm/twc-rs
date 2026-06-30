// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Event types and async event loop for the TUI dashboard.

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use tokio::{sync::mpsc, time::Duration};

use super::app::App;

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
    Status(String),
    /// A freshly fetched data snapshot from the background refresh task.
    Data(Box<super::app::DashboardData>),
    /// Live statistics for the selected resource.
    Stats(Box<super::app::ResourceStats>)
}

/// Processes a single [`AppEvent`] and updates the [`App`] state.
///
/// Returns `false` when the app should quit.
pub fn handle_event(app: &mut App, event: AppEvent) -> bool {
    match event {
        AppEvent::Tick => {
            app.tick();
            true
        }
        AppEvent::Resize(_, _) => true,
        AppEvent::Key(key) => handle_key(app, key),
        AppEvent::Error(msg) => {
            app.error_message = Some(msg);
            true
        }
        AppEvent::Status(msg) => {
            app.status_message = Some(msg);
            true
        }
        AppEvent::Data(data) => {
            app.apply_data(*data);
            true
        }
        AppEvent::Stats(stats) => {
            app.apply_stats(*stats);
            true
        }
    }
}

/// Handles a key when an overlay is active (command palette, action menu,
/// confirm dialog, help, drill view or filter input).
///
/// Returns `Some(continue_running)` when an overlay consumed the key, or
/// `None` to fall through to normal navigation keys.
fn handle_overlay_key(app: &mut App, key: KeyEvent) -> Option<bool> {
    if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('k')) {
        if app.palette_open() {
            app.close_palette();
        } else {
            app.open_palette();
        }
        return Some(true);
    }

    if app.palette_open() {
        match key.code {
            KeyCode::Esc => app.close_palette(),
            KeyCode::Enter => app.palette_run_selected(),
            KeyCode::Up => app.palette_previous(),
            KeyCode::Down => app.palette_next(),
            KeyCode::Backspace => app.palette_backspace(),
            KeyCode::Char(c) => app.palette_input(c),
            _ => {}
        }
        return Some(true);
    }

    if app.create_form_open() {
        match key.code {
            KeyCode::Esc => app.close_create_form(),
            KeyCode::Enter => {
                app.form_submit();
            }
            KeyCode::Tab | KeyCode::Down => app.form_next_field(),
            KeyCode::BackTab | KeyCode::Up => app.form_prev_field(),
            KeyCode::Backspace => app.form_backspace(),
            KeyCode::Char(c) => app.form_input(c),
            _ => {}
        }
        return Some(true);
    }

    if app.action_menu_open() {
        match key.code {
            KeyCode::Char('k') | KeyCode::Up => app.menu_previous(),
            KeyCode::Char('j') | KeyCode::Down => app.menu_next(),
            KeyCode::Enter => app.menu_select(),
            KeyCode::Esc | KeyCode::Char('q') => app.close_action_menu(),
            _ => {}
        }
        return Some(true);
    }

    if app.awaiting_confirm() {
        match key.code {
            KeyCode::Char('y' | 'Y') | KeyCode::Enter => app.confirm_action(),
            _ => app.cancel_action()
        }
        return Some(true);
    }

    if app.show_help {
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('?')) {
            app.show_help = false;
        }
        return Some(true);
    }

    if app.drill_open() {
        match key.code {
            KeyCode::Char('Q') => {
                app.quit();
                return Some(false);
            }
            KeyCode::Esc | KeyCode::Char('q' | 'h') | KeyCode::Left => app.close_drill(),
            KeyCode::Char('j') | KeyCode::Down => app.drill_next(),
            KeyCode::Char('k') | KeyCode::Up => app.drill_previous(),
            _ => {}
        }
        return Some(true);
    }

    if app.filter_editing {
        match key.code {
            KeyCode::Esc => app.filter_clear(),
            KeyCode::Enter => app.filter_apply(),
            KeyCode::Backspace => app.filter_backspace(),
            KeyCode::Char(c) => app.filter_push(c),
            _ => {}
        }
        return Some(true);
    }

    None
}

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    if let Some(result) = handle_overlay_key(app, key) {
        return result;
    }

    match key.code {
        KeyCode::Char('Q') => {
            app.quit();
            false
        }
        KeyCode::Char('/') => {
            app.start_filter();
            true
        }
        KeyCode::Char('n') => {
            app.open_create_form();
            true
        }
        KeyCode::Esc => {
            if app.filter_active() {
                app.filter_clear();
            }
            true
        }
        KeyCode::Char('?') => {
            app.toggle_help();
            true
        }
        KeyCode::Char('r') => {
            app.force_refresh();
            true
        }
        KeyCode::Tab | KeyCode::Char('l') | KeyCode::Right => {
            app.next_tab();
            true
        }
        KeyCode::BackTab | KeyCode::Char('h') | KeyCode::Left => {
            app.previous_tab();
            true
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.select_next();
            true
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.select_previous();
            true
        }
        KeyCode::Char('g') | KeyCode::Home => {
            app.selected = 0;
            true
        }
        KeyCode::Char('G' | '$') | KeyCode::End => {
            let len = app.current_list_len();
            if len > 0 {
                app.selected = len - 1;
            }
            true
        }
        KeyCode::Enter => {
            if app.can_drill() {
                app.request_drill();
            } else {
                app.open_action_menu();
            }
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
