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
    assert_eq!(app.active_tab, ResourceTab::Balancers);

    for _ in 0..14 {
        app.next_tab();
    }
    assert_eq!(app.active_tab, ResourceTab::Finances);
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
    for i in 0..65i32 {
        app.push_net_in(f64::from(i));
    }
    assert_eq!(app.net_in_history.len(), 60);
}

#[test]
fn push_net_out_rolling_window() {
    let mut app = App::new(5);
    for i in 0..65i32 {
        app.push_net_out(f64::from(i));
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
    assert_eq!(ResourceTab::Finances.index(), 19);
    assert_eq!(ResourceTab::names().len(), 20);
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
        code:      KeyCode::Char('Q'),
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

#[test]
fn handle_key_vim_k() {
    let mut app = App::new(5);
    app.servers = vec![
        make_server(1, "s1", "running"),
        make_server(2, "s2", "running"),
    ];
    app.selected = 1;
    let event = AppEvent::Key(crossterm::event::KeyEvent {
        code:      KeyCode::Char('k'),
        modifiers: crossterm::event::KeyModifiers::NONE,
        kind:      crossterm::event::KeyEventKind::Press,
        state:     crossterm::event::KeyEventState::NONE
    });
    let keep_going = crate::tui::event::handle_event(&mut app, event);
    assert!(keep_going);
    assert_eq!(app.selected, 0);
}

#[test]
fn handle_key_vim_j() {
    let mut app = App::new(5);
    app.servers = vec![
        make_server(1, "s1", "running"),
        make_server(2, "s2", "running"),
    ];
    let event = AppEvent::Key(crossterm::event::KeyEvent {
        code:      KeyCode::Char('j'),
        modifiers: crossterm::event::KeyModifiers::NONE,
        kind:      crossterm::event::KeyEventKind::Press,
        state:     crossterm::event::KeyEventState::NONE
    });
    let keep_going = crate::tui::event::handle_event(&mut app, event);
    assert!(keep_going);
    assert_eq!(app.selected, 1);
}

#[test]
fn handle_key_vim_h() {
    let mut app = App::new(5);
    let event = AppEvent::Key(crossterm::event::KeyEvent {
        code:      KeyCode::Char('h'),
        modifiers: crossterm::event::KeyModifiers::NONE,
        kind:      crossterm::event::KeyEventKind::Press,
        state:     crossterm::event::KeyEventState::NONE
    });
    let keep_going = crate::tui::event::handle_event(&mut app, event);
    assert!(keep_going);
    assert_eq!(app.active_tab, ResourceTab::Finances);
}

#[test]
fn handle_key_vim_l() {
    let mut app = App::new(5);
    let event = AppEvent::Key(crossterm::event::KeyEvent {
        code:      KeyCode::Char('l'),
        modifiers: crossterm::event::KeyModifiers::NONE,
        kind:      crossterm::event::KeyEventKind::Press,
        state:     crossterm::event::KeyEventState::NONE
    });
    let keep_going = crate::tui::event::handle_event(&mut app, event);
    assert!(keep_going);
    assert_eq!(app.active_tab, ResourceTab::Databases);
}

#[test]
fn handle_key_g() {
    let mut app = App::new(5);
    app.servers = vec![
        make_server(1, "s1", "running"),
        make_server(2, "s2", "running"),
    ];
    app.selected = 1;
    let event = AppEvent::Key(crossterm::event::KeyEvent {
        code:      KeyCode::Char('g'),
        modifiers: crossterm::event::KeyModifiers::NONE,
        kind:      crossterm::event::KeyEventKind::Press,
        state:     crossterm::event::KeyEventState::NONE
    });
    let keep_going = crate::tui::event::handle_event(&mut app, event);
    assert!(keep_going);
    assert_eq!(app.selected, 0);
}

#[test]
fn handle_key_dollar() {
    let mut app = App::new(5);
    app.servers = vec![
        make_server(1, "s1", "running"),
        make_server(2, "s2", "running"),
    ];
    app.selected = 0;
    let event = AppEvent::Key(crossterm::event::KeyEvent {
        code:      KeyCode::Char('$'),
        modifiers: crossterm::event::KeyModifiers::NONE,
        kind:      crossterm::event::KeyEventKind::Press,
        state:     crossterm::event::KeyEventState::NONE
    });
    let keep_going = crate::tui::event::handle_event(&mut app, event);
    assert!(keep_going);
    assert_eq!(app.selected, 1);
}

fn key(code: KeyCode) -> AppEvent {
    AppEvent::Key(crossterm::event::KeyEvent {
        code,
        modifiers: crossterm::event::KeyModifiers::NONE,
        kind: crossterm::event::KeyEventKind::Press,
        state: crossterm::event::KeyEventState::NONE
    })
}

#[test]
fn open_action_menu_on_servers_tab() {
    let mut app = App::new(5);
    app.servers = vec![make_server(7, "web", "On")];
    app.selected = 0;
    app.open_action_menu();
    let menu = app.action_menu().expect("menu should open");
    assert_eq!(menu.resource_id, 7);
    assert_eq!(menu.resource_name, "web");
    assert_eq!(menu.actions, ResourceTab::Servers.actions().to_vec());
    assert_eq!(menu.selected, 0);
}

#[test]
fn open_action_menu_noop_on_other_tab() {
    let mut app = App::new(5);
    app.active_tab = ResourceTab::Databases;
    app.servers = vec![make_server(7, "web", "On")];
    app.open_action_menu();
    assert!(!app.action_menu_open());
}

#[test]
fn menu_navigation_wraps() {
    let mut app = App::new(5);
    app.servers = vec![make_server(7, "web", "On")];
    app.open_action_menu();
    app.menu_previous();
    assert_eq!(
        app.action_menu().unwrap().selected,
        ResourceTab::Servers.actions().len() - 1
    );
    app.menu_next();
    assert_eq!(app.action_menu().unwrap().selected, 0);
}

#[test]
fn menu_select_non_destructive_dispatches_directly() {
    let mut app = App::new(5);
    app.servers = vec![make_server(7, "web", "On")];
    app.open_action_menu();
    app.menu_select();
    assert!(!app.action_menu_open());
    assert!(!app.awaiting_confirm());
    let dispatched = app.take_dispatch().expect("non-destructive dispatches");
    assert_eq!(dispatched.kind, ActionKind::Start);
    assert_eq!(dispatched.resource_id, 7);
}

#[test]
fn menu_select_destructive_requires_confirm() {
    let mut app = App::new(5);
    app.servers = vec![make_server(7, "web", "On")];
    app.open_action_menu();
    for _ in 0..ResourceTab::Servers.actions().len() - 1 {
        app.menu_next();
    }
    let current = {
        let menu = app.action_menu().unwrap();
        menu.actions[menu.selected]
    };
    assert_eq!(current, ActionKind::Delete);
    app.menu_select();
    assert!(!app.action_menu_open());
    assert!(app.awaiting_confirm());
    assert!(app.take_dispatch().is_none());

    app.confirm_action();
    let dispatched = app.take_dispatch().expect("confirm dispatches");
    assert_eq!(dispatched.kind, ActionKind::Delete);
}

#[test]
fn enter_opens_menu_then_runs_action() {
    let mut app = App::new(5);
    app.servers = vec![make_server(7, "web", "On")];

    crate::tui::event::handle_event(&mut app, key(KeyCode::Enter));
    assert!(app.action_menu_open());

    crate::tui::event::handle_event(&mut app, key(KeyCode::Char('j')));
    crate::tui::event::handle_event(&mut app, key(KeyCode::Enter));
    assert!(!app.action_menu_open());
    let dispatched = app.take_dispatch().expect("action dispatched");
    assert_eq!(dispatched.kind, ActionKind::Shutdown);
}

#[test]
fn menu_esc_closes_without_dispatch() {
    let mut app = App::new(5);
    app.servers = vec![make_server(7, "web", "On")];
    app.open_action_menu();
    crate::tui::event::handle_event(&mut app, key(KeyCode::Esc));
    assert!(!app.action_menu_open());
    assert!(app.take_dispatch().is_none());
}

#[test]
fn per_tab_action_sets() {
    assert_eq!(ResourceTab::Servers.actions().len(), 5);
    assert_eq!(ResourceTab::Databases.actions(), &[ActionKind::Delete]);
    assert_eq!(ResourceTab::Kubernetes.actions(), &[ActionKind::Delete]);
    assert!(ResourceTab::Projects.actions().is_empty());
    assert!(ResourceTab::Finances.actions().is_empty());
}

#[test]
fn action_menu_works_for_databases() {
    let mut app = App::new(5);
    app.active_tab = ResourceTab::Databases;
    app.databases = vec![make_database(42, "pg-prod", "postgres")];
    app.open_action_menu();
    let menu = app.action_menu().expect("menu opens for databases");
    assert_eq!(menu.tab, ResourceTab::Databases);
    assert_eq!(menu.resource_id, 42);
    assert_eq!(menu.resource_name, "pg-prod");
    assert_eq!(menu.actions, vec![ActionKind::Delete]);

    app.menu_select();
    assert!(app.awaiting_confirm());
    app.confirm_action();
    let dispatched = app.take_dispatch().expect("delete dispatched");
    assert_eq!(dispatched.tab, ResourceTab::Databases);
    assert_eq!(dispatched.kind, ActionKind::Delete);
    assert_eq!(dispatched.resource_id, 42);
}

#[test]
fn no_menu_on_action_less_tab() {
    let mut app = App::new(5);
    app.active_tab = ResourceTab::Projects;
    app.projects = vec![make_project(1, "proj")];
    app.open_action_menu();
    assert!(!app.action_menu_open());
}

#[test]
fn toggle_widget_flips_and_marks_dirty() {
    let mut app = App::new(5);
    assert!(app.is_widget_enabled("stats"));
    app.toggle_widget("stats");
    assert!(!app.is_widget_enabled("stats"));
    assert!(app.prefs_dirty);
    assert_eq!(app.hidden_widget_ids(), vec!["stats".to_string()]);
}

#[test]
fn apply_prefs_hides_widgets_and_sets_width() {
    let mut app = App::new(5);
    app.apply_prefs(
        &["account".to_string(), "token_info".to_string()],
        55,
        false
    );
    assert!(!app.is_widget_enabled("account"));
    assert!(!app.is_widget_enabled("token_info"));
    assert!(app.is_widget_enabled("stats"));
    assert_eq!(app.list_width_pct, 55);
}

#[test]
fn set_theme_marks_dirty() {
    let mut app = App::new(5);
    app.set_theme(crate::tui::themes::Theme::CatppuccinMocha);
    assert_eq!(app.theme, crate::tui::themes::Theme::CatppuccinMocha);
    assert!(app.prefs_dirty);
}

#[test]
fn palette_opens_with_context_commands() {
    let mut app = App::new(5);
    app.servers = vec![make_server(7, "web", "On")];
    app.open_palette();
    assert!(app.palette_open());
    let cp = app.palette.as_ref().unwrap();
    let titles: Vec<&str> = cp
        .filtered_commands()
        .iter()
        .map(|c| c.title.as_str())
        .collect();
    let reboot_title = format!("{} web", ActionKind::Reboot.display_label());
    assert!(titles.iter().any(|t| t.contains(reboot_title.as_str())));
    assert!(titles.iter().any(|t| t.starts_with("Theme:")));
    let stats_label = rust_i18n::t!("app.widget_stats");
    assert!(titles.iter().any(|t| t.contains(stats_label.as_ref())));
}

#[test]
fn palette_run_theme_command() {
    let mut app = App::new(5);
    app.open_palette();
    app.run_command("theme:catppuccin_latte");
    assert_eq!(app.theme, crate::tui::themes::Theme::CatppuccinLatte);
}

#[test]
fn palette_run_widget_toggle_command() {
    let mut app = App::new(5);
    app.run_command("widget:stats");
    assert!(!app.is_widget_enabled("stats"));
}

#[test]
fn palette_run_action_command_dispatches() {
    let mut app = App::new(5);
    app.servers = vec![make_server(7, "web", "On")];
    app.run_command("action:reboot");
    let dispatched = app.take_dispatch().expect("reboot dispatched via palette");
    assert_eq!(dispatched.kind, ActionKind::Reboot);
    assert_eq!(dispatched.resource_id, 7);
}

#[test]
fn palette_run_delete_requires_confirm() {
    let mut app = App::new(5);
    app.servers = vec![make_server(7, "web", "On")];
    app.run_command("action:delete");
    assert!(app.awaiting_confirm());
    assert!(app.take_dispatch().is_none());
}

#[test]
fn filter_narrows_list_and_maps_selection() {
    let mut app = App::new(5);
    app.servers = vec![
        make_server(1, "web-prod", "On"),
        make_server(2, "db-prod", "On"),
        make_server(3, "web-stage", "On"),
    ];
    assert_eq!(app.current_list_len(), 3);

    app.start_filter();
    for c in "web".chars() {
        app.filter_push(c);
    }
    assert_eq!(app.filtered_indices(), vec![0, 2]);
    assert_eq!(app.current_list_len(), 2);

    app.selected = 1;
    let (id, name) = app.selected_resource().expect("maps filtered selection");
    assert_eq!(id, 3);
    assert_eq!(name, "web-stage");

    app.filter_clear();
    assert_eq!(app.current_list_len(), 3);
    assert!(!app.filter_active());
}

#[test]
fn filter_resets_on_tab_change() {
    let mut app = App::new(5);
    app.servers = vec![make_server(1, "web", "On")];
    app.start_filter();
    app.filter_push('x');
    assert!(app.filter_active());
    app.next_tab();
    assert!(!app.filter_active());
    assert!(app.filter.is_empty());
}

#[test]
fn drill_request_only_on_projects() {
    let mut app = App::new(5);
    app.servers = vec![make_server(7, "web", "On")];
    app.request_drill();
    assert!(app.take_drill_request().is_none());

    app.active_tab = ResourceTab::Projects;
    app.projects = vec![make_project(3, "ratma")];
    assert!(app.can_drill());
    app.request_drill();
    let req = app.take_drill_request().expect("drill requested");
    assert_eq!(req, (ResourceTab::Projects, 3, "ratma".to_string()));
}

#[test]
fn drill_view_navigation() {
    use crate::tui::app::{DrillItem, DrillView};
    let mut app = App::new(5);
    app.open_drill(DrillView {
        title:    "Project 'x'".to_string(),
        items:    vec![
            DrillItem {
                kind:   "Server".to_string(),
                name:   "a".to_string(),
                detail: String::new()
            },
            DrillItem {
                kind:   "Database".to_string(),
                name:   "b".to_string(),
                detail: String::new()
            },
        ],
        selected: 0
    });
    assert!(app.drill_open());
    app.drill_next();
    assert_eq!(app.drill_view().unwrap().selected, 1);
    app.drill_next();
    assert_eq!(app.drill_view().unwrap().selected, 1);
    app.drill_previous();
    assert_eq!(app.drill_view().unwrap().selected, 0);
    app.close_drill();
    assert!(!app.drill_open());
}

#[test]
fn log_caps_at_200_entries() {
    let mut app = App::new(5);
    for i in 0..250 {
        app.log(crate::tui::app::LogLevel::Info, format!("event {i}"));
    }
    assert_eq!(app.logs.len(), 200);
    assert_eq!(app.logs.back().unwrap().text, "event 249");
}

#[test]
fn force_refresh_sets_request_flag() {
    let mut app = App::new(5);
    assert!(!app.refresh_requested);
    app.force_refresh();
    assert!(app.refresh_requested);
}

#[test]
fn apply_data_logs_load_errors_once() {
    let mut app = App::new(5);
    let data = DashboardData {
        account:           AccountInfo::default(),
        servers:           Vec::new(),
        databases:         Vec::new(),
        s3_storages:       Vec::new(),
        k8s_clusters:      Vec::new(),
        projects:          Vec::new(),
        balancers:         Vec::new(),
        registries:        Vec::new(),
        domains:           Vec::new(),
        firewalls:         Vec::new(),
        floating_ips:      Vec::new(),
        images:            Vec::new(),
        network_drives:    Vec::new(),
        vpcs:              Vec::new(),
        dedicated_servers: Vec::new(),
        mails:             Vec::new(),
        apps:              Vec::new(),
        ai_agents:         Vec::new(),
        knowledge_bases:   Vec::new(),
        ssh_keys:          Vec::new(),
        finances:          Vec::new(),
        error_message:     None,
        status_message:    None,
        load_errors:       vec!["databases".to_string()]
    };
    app.apply_data(data.clone());
    let after_first = app.logs.len();
    assert!(app.logs.iter().any(|e| e.text.contains("databases")));
    app.apply_data(data);
    assert_eq!(app.logs.len(), after_first);
}
