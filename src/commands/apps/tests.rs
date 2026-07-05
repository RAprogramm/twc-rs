use chrono::Local;

use super::*;

fn lines(raw: &[&str]) -> Vec<String> {
    raw.iter().map(|s| (*s).to_owned()).collect()
}

fn utc_bound(raw: &str) -> Option<DateTime<Local>> {
    DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|ts| ts.with_timezone(&Local))
}

#[test]
fn numeric_selector_detects_ids() {
    assert!(is_numeric_selector("219311"));
    assert!(!is_numeric_selector("aura_api_mos"));
    assert!(!is_numeric_selector("219311a"));
    assert!(!is_numeric_selector(""));
}

#[test]
fn match_app_selector_resolves_unique_name() {
    let apps = vec![
        ("189492".to_owned(), "rori-shop-api".to_owned()),
        ("219311".to_owned(), "aura_api_mos".to_owned()),
    ];
    assert_eq!(match_app_selector(&apps, "aura_api_mos").unwrap(), "219311");
}

#[test]
fn match_app_selector_rejects_unknown_and_ambiguous_names() {
    let apps = vec![
        ("1".to_owned(), "twin".to_owned()),
        ("2".to_owned(), "twin".to_owned()),
    ];
    assert!(match_app_selector(&apps, "absent").is_err());
    assert!(match_app_selector(&apps, "twin").is_err());
}

#[test]
fn line_timestamp_parses_rfc3339_prefix() {
    let ts = line_timestamp("2026-07-05T10:00:00Z app started");
    assert!(ts.is_some());
    assert!(line_timestamp("plain continuation line").is_none());
    assert!(line_timestamp("").is_none());
}

#[test]
fn line_timestamp_parses_ansi_wrapped_prefix() {
    let line = "\u{1b}[2m2026-07-05T07:13:30.019889Z\u{1b}[0m \u{1b}[33m WARN\u{1b}[0m message";
    assert!(line_timestamp(line).is_some());
}

#[test]
fn line_timestamp_parses_json_embedded_timestamp() {
    let line = r#"{"timestamp":"2026-07-05T23:10:58.424430Z","level":"INFO","message":"ok"}"#;
    let ts = line_timestamp(line).expect("embedded timestamp must parse");
    assert_eq!(ts.to_rfc3339(), "2026-07-05T23:10:58.424430+00:00");
}

#[test]
fn line_timestamp_parses_json_with_offset_timestamp() {
    let line = r#"{"level":"warn","time":"2026-07-05T12:30:00+03:00","msg":"x"}"#;
    assert!(line_timestamp(line).is_some());
}

#[test]
fn embedded_rfc3339_ignores_date_like_noise() {
    assert!(embedded_rfc3339("build 2026-07-05 finished in 12s").is_none());
    assert!(embedded_rfc3339("id 1234-56-78T90 garbage").is_none());
    assert!(embedded_rfc3339("").is_none());
}

#[test]
fn filter_applies_since_to_json_lines() {
    let input = lines(&[
        r#"{"timestamp":"2026-07-05T09:00:00Z","message":"old"}"#,
        r#"{"timestamp":"2026-07-05T23:00:00Z","message":"new"}"#
    ]);
    let bound = utc_bound("2026-07-05T14:00:00Z");
    let kept = filter_log_lines(&input, bound, None);
    assert_eq!(kept.len(), 1);
    assert!(kept[0].contains("new"));
}

#[test]
fn strip_ansi_removes_csi_sequences() {
    let line = "\u{1b}[2m2026-07-05T07:13:30Z\u{1b}[0m \u{1b}[33mWARN\u{1b}[0m ok";
    assert_eq!(strip_ansi(line), "2026-07-05T07:13:30Z WARN ok");
    assert_eq!(strip_ansi("no escapes"), "no escapes");
}

#[test]
fn resolve_since_accepts_date_and_rfc3339() {
    assert!(resolve_since(Some("2026-07-05"), false).unwrap().is_some());
    assert!(
        resolve_since(Some("2026-07-05T12:30:00+03:00"), false)
            .unwrap()
            .is_some()
    );
    assert!(resolve_since(None, false).unwrap().is_none());
    assert!(resolve_since(None, true).unwrap().is_some());
    assert!(resolve_since(Some("yesterday"), false).is_err());
}

#[test]
fn filter_keeps_everything_without_bounds() {
    let input = lines(&[
        "2026-07-05T10:00:00Z a",
        "no stamp",
        "2026-07-05T11:00:00Z b"
    ]);
    assert_eq!(filter_log_lines(&input, None, None), input);
}

#[test]
fn filter_applies_since_and_inherits_timestamps() {
    let input = lines(&[
        "2026-07-04T23:59:00Z old entry",
        "old continuation",
        "2026-07-05T08:00:00Z fresh entry",
        "fresh continuation"
    ]);
    let bound = utc_bound("2026-07-05T00:00:00Z");
    let kept = filter_log_lines(&input, bound, None);
    assert_eq!(
        kept,
        lines(&["2026-07-05T08:00:00Z fresh entry", "fresh continuation"])
    );
}

#[test]
fn filter_drops_unstamped_prefix_when_bounded() {
    let input = lines(&["orphan line", "2026-07-05T08:00:00Z entry"]);
    let bound = utc_bound("2026-07-05T00:00:00Z");
    let kept = filter_log_lines(&input, bound, None);
    assert_eq!(kept, lines(&["2026-07-05T08:00:00Z entry"]));
}

#[test]
fn filter_tail_takes_last_lines_after_since() {
    let input = lines(&[
        "2026-07-05T08:00:00Z one",
        "2026-07-05T09:00:00Z two",
        "2026-07-05T10:00:00Z three"
    ]);
    let kept = filter_log_lines(&input, None, Some(2));
    assert_eq!(
        kept,
        lines(&["2026-07-05T09:00:00Z two", "2026-07-05T10:00:00Z three"])
    );
    assert!(filter_log_lines(&input, None, Some(0)).is_empty());
    assert_eq!(filter_log_lines(&input, None, Some(10)), input);
}
