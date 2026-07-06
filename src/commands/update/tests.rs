use std::path::Path;

use super::*;

#[test]
fn version_parsing_handles_v_prefix_and_garbage() {
    assert_eq!(parse_version("0.8.1"), Some((0, 8, 1)));
    assert_eq!(parse_version("v1.2.3"), Some((1, 2, 3)));
    assert_eq!(parse_version("not-a-version"), None);
    assert_eq!(parse_version("1.2"), None);
}

#[test]
fn newer_comparison_is_strict() {
    assert!(is_newer("0.9.0", "0.8.1"));
    assert!(is_newer("1.0.0", "0.99.99"));
    assert!(!is_newer("0.8.1", "0.8.1"));
    assert!(!is_newer("0.8.0", "0.8.1"));
    assert!(!is_newer("garbage", "0.8.1"));
}

#[test]
fn channel_detection_by_path() {
    let home = Path::new("/home/user");
    assert_eq!(
        channel_from_path(Path::new("/home/user/.cargo/bin/twc-rs"), Some(home)),
        InstallChannel::Cargo
    );
    assert_eq!(
        channel_from_path(Path::new("/home/user/.local/bin/twc-rs"), Some(home)),
        InstallChannel::Installer
    );
    assert_eq!(
        channel_from_path(Path::new("/usr/local/bin/twc-rs"), Some(home)),
        InstallChannel::Installer
    );
    assert_eq!(
        channel_from_path(Path::new("/usr/bin/twc-rs"), Some(home)),
        InstallChannel::Unknown
    );
    assert_eq!(
        channel_from_path(Path::new("/opt/twc-rs"), None),
        InstallChannel::Unknown
    );
}

#[test]
fn update_plan_for_deb_and_unknown_instructs() {
    let exe = Path::new("/usr/bin/twc-rs");
    assert!(matches!(
        update_plan(InstallChannel::Deb, "0.9.0", exe),
        UpdatePlan::Instruct(_)
    ));
    assert!(matches!(
        update_plan(InstallChannel::Unknown, "0.9.0", exe),
        UpdatePlan::Instruct(_)
    ));
}

#[test]
fn update_plan_for_cargo_runs_cargo_install() {
    let exe = Path::new("/home/user/.cargo/bin/twc-rs");
    match update_plan(InstallChannel::Cargo, "0.9.0", exe) {
        UpdatePlan::Run(argv) => assert_eq!(argv, ["cargo", "install", "twc-rs"]),
        UpdatePlan::Instruct(_) => panic!("cargo channel must run a command")
    }
}
