use crossterm::event::KeyCode;

use super::*;
use crate::tui::event::AppEvent;

fn make_server(id: i32, name: &str, status: &str) -> ServerSummary {
    ServerSummary {
        id,
        name: name.to_string(),
        status: status.to_string(),
        cpu_count: 2,
        ram_mb: 4096,
        disk_gb: 50
    }
}

#[test]
fn new_app_defaults() {
    let app = App::new(5);
    assert!(app.servers.is_empty());
    assert_eq!(app.selected, 0);
    assert_eq!(app.active_tab, Tab::Overview);
    assert!(app.running);
    assert!(!app.show_help);
    assert_eq!(app.refresh_interval, Duration::from_secs(5));
}

#[test]
fn select_previous_clamps_at_zero() {
    let mut app = App::new(5);
    app.servers = vec![make_server(1, "s1", "running")];
    app.select_previous();
    assert_eq!(app.selected, 0);
}

#[test]
fn select_next_clamps_at_end() {
    let mut app = App::new(5);
    app.servers = vec![make_server(1, "s1", "running")];
    app.selected = 0;
    app.select_next();
    assert_eq!(app.selected, 0);
}

#[test]
fn select_next_moves_down() {
    let mut app = App::new(5);
    app.servers = vec![
        make_server(1, "s1", "running"),
        make_server(2, "s2", "running"),
    ];
    app.select_next();
    assert_eq!(app.selected, 1);
}

#[test]
fn select_previous_moves_up() {
    let mut app = App::new(5);
    app.servers = vec![
        make_server(1, "s1", "running"),
        make_server(2, "s2", "running"),
    ];
    app.selected = 1;
    app.select_previous();
    assert_eq!(app.selected, 0);
}

#[test]
fn next_tab_cycles() {
    let mut app = App::new(5);
    assert_eq!(app.active_tab, Tab::Overview);
    app.next_tab();
    assert_eq!(app.active_tab, Tab::Servers);
    app.next_tab();
    assert_eq!(app.active_tab, Tab::Databases);
    app.next_tab();
    assert_eq!(app.active_tab, Tab::Storage);
    app.next_tab();
    assert_eq!(app.active_tab, Tab::Overview);
}

#[test]
fn toggle_help() {
    let mut app = App::new(5);
    assert!(!app.show_help);
    app.toggle_help();
    assert!(app.show_help);
    app.toggle_help();
    assert!(!app.show_help);
}

#[test]
fn quit_sets_running_false() {
    let mut app = App::new(5);
    app.quit();
    assert!(!app.running);
}

#[test]
fn push_cpu_rolling_window() {
    let mut app = App::new(5);
    for i in 0..65 {
        app.push_cpu(f64::from(i));
    }
    assert_eq!(app.cpu_history.len(), 60);
    assert_eq!(*app.cpu_history.front().unwrap(), 5.0);
    assert_eq!(*app.cpu_history.back().unwrap(), 64.0);
}

#[test]
fn push_ram_rolling_window() {
    let mut app = App::new(5);
    for i in 0..65 {
        app.push_ram(f64::from(i));
    }
    assert_eq!(app.ram_history.len(), 60);
}

#[test]
fn push_net_in_rolling_window() {
    let mut app = App::new(5);
    for i in 0..65u64 {
        app.push_net_in(i);
    }
    assert_eq!(app.net_in_history.len(), 60);
}

#[test]
fn push_net_out_rolling_window() {
    let mut app = App::new(5);
    for i in 0..65u64 {
        app.push_net_out(i);
    }
    assert_eq!(app.net_out_history.len(), 60);
}

#[test]
fn update_servers_clamps_selection() {
    let mut app = App::new(5);
    app.servers = vec![
        make_server(1, "s1", "running"),
        make_server(2, "s2", "running"),
    ];
    app.selected = 5;
    app.update_servers(vec![make_server(3, "s3", "running")]);
    assert_eq!(app.selected, 0);
    assert_eq!(app.servers.len(), 1);
}

#[test]
fn needs_refresh_after_interval() {
    let app = App::new(0); // 0 second interval
    assert!(app.needs_refresh());
}

#[test]
fn tab_names_and_indices() {
    assert_eq!(Tab::Overview.index(), 0);
    assert_eq!(Tab::Servers.index(), 1);
    assert_eq!(Tab::Databases.index(), 2);
    assert_eq!(Tab::Storage.index(), 3);
    assert_eq!(Tab::names().len(), 4);
}

#[test]
fn handle_key_quit() {
    let mut app = App::new(5);
    let event = AppEvent::Key(crossterm::event::KeyEvent {
        code:      KeyCode::Char('q'),
        modifiers: crossterm::event::KeyModifiers::NONE,
        kind:      crossterm::event::KeyEventKind::Press,
        state:     crossterm::event::KeyEventState::NONE
    });
    let keep_going = crate::tui::event::handle_event(&mut app, event);
    assert!(!keep_going);
    assert!(!app.running);
}

#[test]
fn handle_key_tab() {
    let mut app = App::new(5);
    let event = AppEvent::Key(crossterm::event::KeyEvent {
        code:      KeyCode::Tab,
        modifiers: crossterm::event::KeyModifiers::NONE,
        kind:      crossterm::event::KeyEventKind::Press,
        state:     crossterm::event::KeyEventState::NONE
    });
    let keep_going = crate::tui::event::handle_event(&mut app, event);
    assert!(keep_going);
    assert_eq!(app.active_tab, Tab::Servers);
}
