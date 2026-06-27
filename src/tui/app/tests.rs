use crossterm::event::KeyCode;

use super::*;
use crate::tui::event::AppEvent;

fn make_server(id: i32, name: &str, status: &str) -> ServerSummary {
    ServerSummary {
        id,
        name: name.to_string(),
        status: status.to_string(),
        cpu: 2,
        ram_mb: 4096,
        disk_gb: 50,
        ip: "192.168.1.1".to_string(),
        location: "msk".to_string()
    }
}

fn make_database(id: i32, name: &str, engine: &str) -> DatabaseSummary {
    DatabaseSummary {
        id,
        name: name.to_string(),
        status: "active".to_string(),
        engine: engine.to_string(),
        size_mb: 512
    }
}

fn make_s3(id: i32, name: &str) -> S3Summary {
    S3Summary {
        id,
        name: name.to_string(),
        region: "ru-1".to_string(),
        size_bytes: 1_000_000_000,
        bucket_count: 3
    }
}

fn make_k8s(id: i32, name: &str) -> K8sSummary {
    K8sSummary {
        id,
        name: name.to_string(),
        status: "running".to_string(),
        version: "1.28".to_string(),
        node_count: 3
    }
}

fn make_project(id: i32, name: &str) -> ProjectSummary {
    ProjectSummary {
        id,
        name: name.to_string(),
        server_count: 2
    }
}

#[test]
fn new_app_defaults() {
    let app = App::new(5);
    assert!(app.servers.is_empty());
    assert!(app.databases.is_empty());
    assert!(app.s3_storages.is_empty());
    assert!(app.k8s_clusters.is_empty());
    assert!(app.projects.is_empty());
    assert_eq!(app.selected, 0);
    assert_eq!(app.active_tab, ResourceTab::Servers);
    assert!(app.running);
    assert!(!app.show_help);
    assert_eq!(app.refresh_interval, Duration::from_secs(5));
}

#[test]
fn account_default() {
    let app = App::new(5);
    assert_eq!(app.account.account_id, 0.0);
    assert!(app.account.balance.is_empty());
    assert!(app.account.status.is_empty());
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
    assert_eq!(app.active_tab, ResourceTab::Servers);
    app.next_tab();
    assert_eq!(app.active_tab, ResourceTab::Databases);
    app.next_tab();
    assert_eq!(app.active_tab, ResourceTab::S3);
    app.next_tab();
    assert_eq!(app.active_tab, ResourceTab::Kubernetes);
    app.next_tab();
    assert_eq!(app.active_tab, ResourceTab::Projects);
    app.next_tab();
    assert_eq!(app.active_tab, ResourceTab::Servers);
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
fn update_databases_clamps_selection() {
    let mut app = App::new(5);
    app.databases = vec![
        make_database(1, "db1", "postgres"),
        make_database(2, "db2", "mysql"),
    ];
    app.selected = 5;
    app.update_databases(vec![make_database(3, "db3", "redis")]);
    assert_eq!(app.selected, 0);
    assert_eq!(app.databases.len(), 1);
}

#[test]
fn update_s3_clamps_selection() {
    let mut app = App::new(5);
    app.s3_storages = vec![make_s3(1, "bucket1")];
    app.selected = 5;
    app.update_s3(vec![make_s3(2, "bucket2")]);
    assert_eq!(app.selected, 0);
    assert_eq!(app.s3_storages.len(), 1);
}

#[test]
fn update_k8s_clamps_selection() {
    let mut app = App::new(5);
    app.k8s_clusters = vec![make_k8s(1, "cluster1")];
    app.selected = 5;
    app.update_k8s(vec![make_k8s(2, "cluster2")]);
    assert_eq!(app.selected, 0);
    assert_eq!(app.k8s_clusters.len(), 1);
}

#[test]
fn update_projects_clamps_selection() {
    let mut app = App::new(5);
    app.projects = vec![make_project(1, "proj1")];
    app.selected = 5;
    app.update_projects(vec![make_project(2, "proj2")]);
    assert_eq!(app.selected, 0);
    assert_eq!(app.projects.len(), 1);
}

#[test]
fn needs_refresh_after_interval() {
    let app = App::new(0);
    assert!(app.needs_refresh());
}

#[test]
fn tab_names_and_indices() {
    assert_eq!(ResourceTab::Servers.index(), 0);
    assert_eq!(ResourceTab::Databases.index(), 1);
    assert_eq!(ResourceTab::S3.index(), 2);
    assert_eq!(ResourceTab::Kubernetes.index(), 3);
    assert_eq!(ResourceTab::Projects.index(), 4);
    assert_eq!(ResourceTab::names().len(), 5);
}

#[test]
fn current_list_len_servers() {
    let mut app = App::new(5);
    app.servers = vec![make_server(1, "s1", "running")];
    assert_eq!(app.current_list_len(), 1);
}

#[test]
fn current_list_len_databases() {
    let mut app = App::new(5);
    app.active_tab = ResourceTab::Databases;
    app.databases = vec![make_database(1, "db1", "pg")];
    assert_eq!(app.current_list_len(), 1);
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
    assert_eq!(app.active_tab, ResourceTab::Databases);
}
