// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Event types and async event loop for the TUI dashboard.

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use tokio::{sync::mpsc, time::Duration};

use super::app::{App, FocusDir, Pane};

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
    /// A streamed load cycle began.
    LoadStarted,
    /// One resource finished loading and can be shown immediately.
    Slice(Box<super::app::DataSlice>),
    /// All endpoints of a streamed load cycle finished.
    LoadFinished,
    /// A project's drill contents finished loading in the background.
    Drill {
        /// The project id the contents belong to.
        id:   i32,
        /// The fetched contents.
        view: Box<super::app::DrillView>
    },
    /// Deep details for a resource finished loading in the background.
    DetailExtra {
        /// The resource category.
        tab:      super::app::ResourceTab,
        /// The resource id.
        id:       i32,
        /// Extra detail sections: `(title, rows)`.
        sections: super::app::DetailSections
    },
    /// A background drill fetch failed.
    DrillFailed {
        /// The project name, for the log entry.
        name:  String,
        /// The failure message.
        error: String
    },
    /// Live statistics for the selected resource.
    Stats(Box<super::app::ResourceStats>),
    /// A statistics fetch failed; logged without blocking the dashboard.
    StatsError(String)
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
        AppEvent::LoadStarted => {
            app.load_started();
            true
        }
        AppEvent::Slice(slice) => {
            app.apply_slice(*slice);
            true
        }
        AppEvent::LoadFinished => {
            app.load_finished();
            true
        }
        AppEvent::Drill {
            id,
            view
        } => {
            app.apply_drill(id, *view);
            true
        }
        AppEvent::DetailExtra {
            tab,
            id,
            sections
        } => {
            app.detail_extra.insert((tab, id), sections);
            true
        }
        AppEvent::DrillFailed {
            name,
            error
        } => {
            app.log(
                super::app::LogLevel::Error,
                format!("open {name} failed: {error}")
            );
            true
        }
        AppEvent::Stats(stats) => {
            app.apply_stats(*stats);
            true
        }
        AppEvent::StatsError(msg) => {
            app.log(super::app::LogLevel::Warn, msg);
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

    if app.info_popup_open() {
        if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q')) {
            app.info_popup_close();
        }
        return Some(true);
    }

    if app.picker_open() {
        match key.code {
            KeyCode::Esc => app.picker_close(),
            KeyCode::Enter => app.picker_apply(),
            KeyCode::Up | KeyCode::Char('k') => app.picker_previous(),
            KeyCode::Down | KeyCode::Char('j') => app.picker_next(),
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

/// Handles a key outside overlays: sidebar navigation on the left pane,
/// exact one-step grid movement on the content pane.
/// Handles a key while the settings panel owns the content pane: arrows walk
/// the setting cards like any grid, Enter toggles or opens the picker.
fn handle_settings_key(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.settings_move(FocusDir::Up),
        KeyCode::Down | KeyCode::Char('j') => app.settings_move(FocusDir::Down),
        KeyCode::Right | KeyCode::Char('l') => app.settings_move(FocusDir::Right),
        KeyCode::Left | KeyCode::Char('h') => app.settings_move(FocusDir::Left),
        KeyCode::Enter => app.settings_activate(),
        KeyCode::Esc => app.focus_sidebar(),
        _ => return false
    }
    true
}

/// Handles a key while the create hub owns the content pane.
fn handle_create_key(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.create_move(FocusDir::Up),
        KeyCode::Down | KeyCode::Char('j') => app.create_move(FocusDir::Down),
        KeyCode::Right | KeyCode::Char('l') => app.create_move(FocusDir::Right),
        KeyCode::Left | KeyCode::Char('h') => app.create_move(FocusDir::Left),
        KeyCode::Enter => app.create_activate(),
        KeyCode::Esc => app.focus_sidebar(),
        _ => return false
    }
    true
}

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    if let Some(result) = handle_overlay_key(app, key) {
        return result;
    }

    if app.detail_open {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => app.detail_open = false,
            KeyCode::Down | KeyCode::Char('j') => {
                let len = crate::tui::widgets::details::interactive_len(app);
                if app.detail_selected + 1 < len {
                    app.detail_selected += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.detail_selected = app.detail_selected.saturating_sub(1);
            }
            KeyCode::Char('y' | 'c') => {
                if let Some((Some(value), _)) =
                    crate::tui::widgets::details::interactive_at(app, app.detail_selected)
                {
                    crate::tui::clipboard::copy(&value);
                    app.status_message =
                        Some(rust_i18n::t!("details.copied", value => value).into_owned());
                }
            }
            KeyCode::Enter => {
                if let Some((_, Some(action))) =
                    crate::tui::widgets::details::interactive_at(app, app.detail_selected)
                    && let Some((id, _)) = app.selected_resource()
                    && let Ok(id) = id.parse::<i32>()
                {
                    app.detail_action = Some((id, action));
                }
            }
            KeyCode::Char('Q') => {
                app.quit();
                return false;
            }
            _ => {}
        }
        return true;
    }

    if app.pane == Pane::Content
        && matches!(app.nav_current(), Some(super::app::NavKind::Settings))
        && handle_settings_key(app, key)
    {
        return true;
    }

    if app.pane == Pane::Content
        && matches!(app.nav_current(), Some(super::app::NavKind::Create))
        && handle_create_key(app, key)
    {
        return true;
    }

    match key.code {
        KeyCode::Char('Q') => {
            app.quit();
            return false;
        }
        KeyCode::Char('?') => app.toggle_help(),
        KeyCode::Char('r') => app.force_refresh(),
        KeyCode::Char('p') => app.open_profile_switcher(),
        KeyCode::Char('/') => {
            if app.pane == Pane::Content && !app.drill_open() {
                app.start_filter();
            }
        }
        KeyCode::Char('n') => {
            if app.pane == Pane::Content {
                app.open_create_form();
            }
        }
        KeyCode::Esc => {
            if app.pane == Pane::Content {
                if app.filter_active() {
                    app.filter_clear();
                } else {
                    app.focus_sidebar();
                }
            }
        }
        KeyCode::Tab => app.nav_down(),
        KeyCode::BackTab => app.nav_up(),
        KeyCode::Up | KeyCode::Char('k') => {
            if app.pane == Pane::Sidebar {
                app.nav_up();
            } else {
                app.content_move(FocusDir::Up);
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.pane == Pane::Sidebar {
                app.nav_down();
            } else {
                app.content_move(FocusDir::Down);
            }
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if app.pane == Pane::Content {
                app.content_move(FocusDir::Right);
            }
        }
        KeyCode::Left | KeyCode::Char('h') => {
            if app.pane == Pane::Content {
                app.content_move(FocusDir::Left);
            }
        }
        KeyCode::Char('g') | KeyCode::Home => {
            if app.pane == Pane::Content {
                app.set_content_selected(0);
            }
        }
        KeyCode::Char('G' | '$') | KeyCode::End => {
            if app.pane == Pane::Content {
                let len = app.content_len();
                if len > 0 {
                    app.set_content_selected(len - 1);
                }
            }
        }
        KeyCode::Enter => {
            if app.pane == Pane::Sidebar {
                app.nav_open();
            } else if app.content_on_create {
                app.open_create_form();
            } else if app.drill_open() {
                app.open_drill_item_detail();
            } else if !matches!(app.nav_current(), Some(super::app::NavKind::Project(_))) {
                app.open_action_menu();
            }
        }
        _ => {}
    }
    true
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
