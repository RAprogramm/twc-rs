use assert_cmd::Command;
use predicates::prelude::*;

fn twc() -> Command {
    let mut cmd = Command::cargo_bin("twc-rs").unwrap();
    cmd.env_remove("TWC_TOKEN");
    cmd.env_remove("TWC_OUTPUT");
    cmd
}

#[test]
fn help_exits_zero() {
    twc()
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn help_contains_about_text() {
    twc()
        .arg("--help")
        .assert()
        .stdout(predicate::str::contains("Timeweb Cloud CLI"));
}

#[test]
fn help_contains_subcommands() {
    let output = twc().arg("--help").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("server"));
    assert!(stdout.contains("ssh"));
    assert!(stdout.contains("project"));
    assert!(stdout.contains("config"));
}

#[test]
fn version_exits_zero() {
    twc()
        .arg("--version")
        .assert()
        .success();
}

#[test]
fn version_contains_version_string() {
    twc()
        .arg("--version")
        .assert()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn config_show_no_config_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    twc()
        .env("XDG_CONFIG_HOME", dir.path())
        .args(["config", "show"])
        .assert()
        .success();
}

#[test]
fn config_show_output_contains_token_set() {
    let dir = tempfile::tempdir().unwrap();
    let output = twc()
        .env("XDG_CONFIG_HOME", dir.path())
        .args(["config", "show"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Token set:"));
}

#[test]
fn config_show_output_contains_config_path() {
    let dir = tempfile::tempdir().unwrap();
    let output = twc()
        .env("XDG_CONFIG_HOME", dir.path())
        .args(["config", "show"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Config path:"));
}

#[test]
fn config_set_token_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    twc()
        .env("XDG_CONFIG_HOME", dir.path())
        .args(["config", "set-token", "--token", "test-token-123"])
        .assert()
        .success();
}

#[test]
fn config_set_token_shows_saved() {
    let dir = tempfile::tempdir().unwrap();
    let output = twc()
        .env("XDG_CONFIG_HOME", dir.path())
        .args(["config", "set-token", "-t", "abc"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Token saved."));
}

#[test]
fn config_set_token_then_show_reflects_it() {
    let dir = tempfile::tempdir().unwrap();
    twc()
        .env("XDG_CONFIG_HOME", dir.path())
        .args(["config", "set-token", "--token", "my-tok"])
        .assert()
        .success();
    let output = twc()
        .env("XDG_CONFIG_HOME", dir.path())
        .args(["config", "show"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Token set:   true"));
}

#[test]
fn server_list_without_token_errors() {
    let dir = tempfile::tempdir().unwrap();
    twc()
        .env("XDG_CONFIG_HOME", dir.path())
        .args(["server", "list"])
        .assert()
        .failure();
}

#[test]
fn server_list_invalid_token_errors() {
    let dir = tempfile::tempdir().unwrap();
    let output = twc()
        .env("XDG_CONFIG_HOME", dir.path())
        .args(["server", "list", "--token", "invalid-token"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Error"));
}

#[test]
fn ssh_list_without_token_errors() {
    let dir = tempfile::tempdir().unwrap();
    twc()
        .env("XDG_CONFIG_HOME", dir.path())
        .args(["ssh", "list"])
        .assert()
        .failure();
}

#[test]
fn project_list_without_token_errors() {
    let dir = tempfile::tempdir().unwrap();
    twc()
        .env("XDG_CONFIG_HOME", dir.path())
        .args(["project", "list"])
        .assert()
        .failure();
}

#[test]
fn invalid_subcommand_exits_nonzero() {
    twc()
        .args(["nonexistent"])
        .assert()
        .failure();
}

#[test]
fn format_json_flag_accepted() {
    let dir = tempfile::tempdir().unwrap();
    twc()
        .env("XDG_CONFIG_HOME", dir.path())
        .args(["--format", "json", "config", "show"])
        .assert()
        .success();
}

#[test]
fn format_invalid_flag_errors() {
    let dir = tempfile::tempdir().unwrap();
    twc()
        .env("XDG_CONFIG_HOME", dir.path())
        .args(["--format", "xml", "config", "show"])
        .assert()
        .failure();
}

#[test]
fn server_info_without_token_errors() {
    let dir = tempfile::tempdir().unwrap();
    twc()
        .env("XDG_CONFIG_HOME", dir.path())
        .args(["server", "info", "--id", "1"])
        .assert()
        .failure();
}

#[test]
fn ssh_add_without_token_errors() {
    let dir = tempfile::tempdir().unwrap();
    twc()
        .env("XDG_CONFIG_HOME", dir.path())
        .args(["ssh", "add", "--name", "test"])
        .assert()
        .failure();
}

#[test]
fn project_create_without_token_errors() {
    let dir = tempfile::tempdir().unwrap();
    twc()
        .env("XDG_CONFIG_HOME", dir.path())
        .args(["project", "create", "--name", "test"])
        .assert()
        .failure();
}

#[test]
fn server_delete_without_token_errors() {
    let dir = tempfile::tempdir().unwrap();
    twc()
        .env("XDG_CONFIG_HOME", dir.path())
        .args(["server", "delete", "--id", "1"])
        .assert()
        .failure();
}

#[test]
fn server_reboot_without_token_errors() {
    let dir = tempfile::tempdir().unwrap();
    twc()
        .env("XDG_CONFIG_HOME", dir.path())
        .args(["server", "reboot", "--id", "1"])
        .assert()
        .failure();
}

#[test]
fn ssh_delete_without_token_errors() {
    let dir = tempfile::tempdir().unwrap();
    twc()
        .env("XDG_CONFIG_HOME", dir.path())
        .args(["ssh", "delete", "--id", "1"])
        .assert()
        .failure();
}

#[test]
fn project_delete_without_token_errors() {
    let dir = tempfile::tempdir().unwrap();
    twc()
        .env("XDG_CONFIG_HOME", dir.path())
        .args(["project", "delete", "--id", "1"])
        .assert()
        .failure();
}

#[test]
fn config_show_no_subcommand_errors() {
    twc()
        .args(["config"])
        .assert()
        .failure();
}

#[test]
fn server_no_subcommand_errors() {
    twc()
        .args(["server"])
        .assert()
        .failure();
}

#[test]
fn ssh_no_subcommand_errors() {
    twc()
        .args(["ssh"])
        .assert()
        .failure();
}

#[test]
fn project_no_subcommand_errors() {
    twc()
        .args(["project"])
        .assert()
        .failure();
}
