use clap::Parser;

use super::*;

fn parse(args: &[&str]) -> Cli {
    Cli::try_parse_from(
        std::iter::once("twc-rs").chain(args.iter().copied()),
    )
    .expect("should parse successfully")
}

#[test]
fn default_format_is_table() {
    let cli = parse(&["config", "show"]);
    assert_eq!(cli.format, "table");
}

#[test]
fn format_flag_json() {
    let cli = parse(&["--format", "json", "config", "show"]);
    assert_eq!(cli.format, "json");
}

#[test]
fn format_short_flag() {
    let cli = parse(&["-f", "quiet", "config", "show"]);
    assert_eq!(cli.format, "quiet");
}

#[test]
fn format_json_alias() {
    let cli = parse(&["--format", "js", "config", "show"]);
    assert_eq!(cli.format, "js");
}

#[test]
fn format_quiet_alias() {
    let cli = parse(&["--format", "q", "config", "show"]);
    assert_eq!(cli.format, "q");
}

#[test]
fn token_flag() {
    let cli = parse(&["--token", "abc123", "config", "show"]);
    assert_eq!(cli.token.as_deref(), Some("abc123"));
}

#[test]
fn token_short_flag() {
    let cli = parse(&["-t", "xyz", "config", "show"]);
    assert_eq!(cli.token.as_deref(), Some("xyz"));
}

#[test]
fn no_token_flag() {
    let cli = parse(&["config", "show"]);
    assert!(cli.token.is_none());
}

#[test]
fn server_list_subcommand() {
    let cli = parse(&["server", "list"]);
    assert!(matches!(
        cli.command,
        Commands::Server(ServerCommands::List { .. })
    ));
}

#[test]
fn server_list_with_limit_offset() {
    let cli = parse(&["server", "list", "-l", "10", "-o", "5"]);
    match &cli.command {
        Commands::Server(ServerCommands::List { limit, offset }) => {
            assert_eq!(*limit, Some(10));
            assert_eq!(*offset, Some(5));
        }
        _ => panic!("expected ServerCommands::List"),
    }
}

#[test]
fn server_list_with_only_limit() {
    let cli = parse(&["server", "list", "-l", "20"]);
    match &cli.command {
        Commands::Server(ServerCommands::List { limit, offset }) => {
            assert_eq!(*limit, Some(20));
            assert_eq!(*offset, None);
        }
        _ => panic!("expected ServerCommands::List"),
    }
}

#[test]
fn server_list_with_only_offset() {
    let cli = parse(&["server", "list", "-o", "10"]);
    match &cli.command {
        Commands::Server(ServerCommands::List { limit, offset }) => {
            assert_eq!(*limit, None);
            assert_eq!(*offset, Some(10));
        }
        _ => panic!("expected ServerCommands::List"),
    }
}

#[test]
fn server_info_subcommand() {
    let cli = parse(&["server", "info", "--id", "42"]);
    match &cli.command {
        Commands::Server(ServerCommands::Info { id }) => {
            assert_eq!(*id, 42);
        }
        _ => panic!("expected ServerCommands::Info"),
    }
}

#[test]
fn server_info_short_id() {
    let cli = parse(&["server", "info", "-i", "99"]);
    match &cli.command {
        Commands::Server(ServerCommands::Info { id }) => {
            assert_eq!(*id, 99);
        }
        _ => panic!("expected ServerCommands::Info"),
    }
}

#[test]
fn server_delete_subcommand() {
    let cli = parse(&["server", "delete", "--id", "7"]);
    match &cli.command {
        Commands::Server(ServerCommands::Delete { id }) => {
            assert_eq!(*id, 7);
        }
        _ => panic!("expected ServerCommands::Delete"),
    }
}

#[test]
fn server_reboot_subcommand() {
    let cli = parse(&["server", "reboot", "-i", "3"]);
    match &cli.command {
        Commands::Server(ServerCommands::Reboot { id }) => {
            assert_eq!(*id, 3);
        }
        _ => panic!("expected ServerCommands::Reboot"),
    }
}

#[test]
fn ssh_list_subcommand() {
    let cli = parse(&["ssh", "list"]);
    assert!(matches!(cli.command, Commands::Ssh(SshCommands::List)));
}

#[test]
fn ssh_delete_subcommand() {
    let cli = parse(&["ssh", "delete", "--id", "99"]);
    match &cli.command {
        Commands::Ssh(SshCommands::Delete { id }) => {
            assert_eq!(*id, 99);
        }
        _ => panic!("expected SshCommands::Delete"),
    }
}

#[test]
fn project_list_subcommand() {
    let cli = parse(&["project", "list"]);
    assert!(matches!(
        cli.command,
        Commands::Project(ProjectCommands::List)
    ));
}

#[test]
fn project_create_subcommand() {
    let cli = parse(&["project", "create", "--name", "my-project"]);
    match &cli.command {
        Commands::Project(ProjectCommands::Create { name, description }) => {
            assert_eq!(name, "my-project");
            assert!(description.is_none());
        }
        _ => panic!("expected ProjectCommands::Create"),
    }
}

#[test]
fn project_create_with_description() {
    let cli = parse(&[
        "project", "create", "-n", "web", "-d", "Production web app",
    ]);
    match &cli.command {
        Commands::Project(ProjectCommands::Create { name, description }) => {
            assert_eq!(name, "web");
            assert_eq!(description.as_deref(), Some("Production web app"));
        }
        _ => panic!("expected ProjectCommands::Create"),
    }
}

#[test]
fn project_delete_subcommand() {
    let cli = parse(&["project", "delete", "--id", "5"]);
    match &cli.command {
        Commands::Project(ProjectCommands::Delete { id }) => {
            assert_eq!(*id, 5);
        }
        _ => panic!("expected ProjectCommands::Delete"),
    }
}

#[test]
fn config_show_subcommand() {
    let cli = parse(&["config", "show"]);
    assert!(matches!(
        cli.command,
        Commands::Config(ConfigCommands::Show)
    ));
}

#[test]
fn config_set_token_subcommand() {
    let cli = parse(&["config", "set-token", "--token", "secret"]);
    match &cli.command {
        Commands::Config(ConfigCommands::SetToken { token }) => {
            assert_eq!(token, "secret");
        }
        _ => panic!("expected ConfigCommands::SetToken"),
    }
}

#[test]
fn config_set_token_short_flag() {
    let cli = parse(&["config", "set-token", "-t", "abc"]);
    match &cli.command {
        Commands::Config(ConfigCommands::SetToken { token }) => {
            assert_eq!(token, "abc");
        }
        _ => panic!("expected ConfigCommands::SetToken"),
    }
}

#[test]
fn invalid_subcommand_fails() {
    let result = Cli::try_parse_from(["twc-rs", "nonexistent"]);
    assert!(result.is_err());
}

#[test]
fn server_requires_subcommand() {
    let result = Cli::try_parse_from(["twc-rs", "server"]);
    assert!(result.is_err());
}

#[test]
fn ssh_requires_subcommand() {
    let result = Cli::try_parse_from(["twc-rs", "ssh"]);
    assert!(result.is_err());
}

#[test]
fn project_requires_subcommand() {
    let result = Cli::try_parse_from(["twc-rs", "project"]);
    assert!(result.is_err());
}

#[test]
fn config_requires_subcommand() {
    let result = Cli::try_parse_from(["twc-rs", "config"]);
    assert!(result.is_err());
}

#[test]
fn server_info_requires_id() {
    let result =
        Cli::try_parse_from(["twc-rs", "server", "info"]);
    assert!(result.is_err());
}

#[test]
fn ssh_delete_requires_id() {
    let result =
        Cli::try_parse_from(["twc-rs", "ssh", "delete"]);
    assert!(result.is_err());
}

#[test]
fn project_create_requires_name() {
    let result =
        Cli::try_parse_from(["twc-rs", "project", "create"]);
    assert!(result.is_err());
}

#[test]
fn global_flags_before_subcommand() {
    let cli = parse(&["-t", "tok", "-f", "json", "config", "show"]);
    assert_eq!(cli.token.as_deref(), Some("tok"));
    assert_eq!(cli.format, "json");
    assert!(matches!(
        cli.command,
        Commands::Config(ConfigCommands::Show)
    ));
}

#[test]
fn help_returns_error_with_help_displayed() {
    let result = Cli::try_parse_from(["twc-rs", "--help"]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    let help_text = err.render().to_string();
    assert!(help_text.contains("Timeweb Cloud CLI"));
    assert!(help_text.contains("server"));
    assert!(help_text.contains("ssh"));
    assert!(help_text.contains("project"));
    assert!(help_text.contains("config"));
}
