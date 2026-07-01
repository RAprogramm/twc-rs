use super::*;

#[test]
fn default_is_table() {
    let fmt = OutputFormat::default();
    assert_eq!(fmt, OutputFormat::Table);
}

#[test]
fn parse_table() {
    assert_eq!(OutputFormat::parse("table").unwrap(), OutputFormat::Table);
}

#[test]
fn parse_tbl() {
    assert_eq!(OutputFormat::parse("tbl").unwrap(), OutputFormat::Table);
}

#[test]
fn parse_table_case_insensitive() {
    assert_eq!(OutputFormat::parse("TABLE").unwrap(), OutputFormat::Table);
    assert_eq!(OutputFormat::parse("Table").unwrap(), OutputFormat::Table);
}

#[test]
fn parse_json() {
    assert_eq!(OutputFormat::parse("json").unwrap(), OutputFormat::Json);
}

#[test]
fn parse_js() {
    assert_eq!(OutputFormat::parse("js").unwrap(), OutputFormat::Json);
}

#[test]
fn parse_json_case_insensitive() {
    assert_eq!(OutputFormat::parse("JSON").unwrap(), OutputFormat::Json);
}

#[test]
fn parse_quiet() {
    assert_eq!(OutputFormat::parse("quiet").unwrap(), OutputFormat::Quiet);
}

#[test]
fn parse_q() {
    assert_eq!(OutputFormat::parse("q").unwrap(), OutputFormat::Quiet);
}

#[test]
fn parse_quiet_case_insensitive() {
    assert_eq!(OutputFormat::parse("QUIET").unwrap(), OutputFormat::Quiet);
}

#[test]
fn parse_invalid_format() {
    let err = OutputFormat::parse("xml").unwrap_err();
    assert!(err.contains("unknown output format: xml"));
    assert!(err.contains("expected table, json, yaml, or quiet"));
}

#[test]
fn parse_empty_string() {
    let err = OutputFormat::parse("").unwrap_err();
    assert!(err.contains("unknown output format"));
}

#[test]
fn display_table() {
    assert_eq!(OutputFormat::Table.to_string(), "table");
}

#[test]
fn display_json() {
    assert_eq!(OutputFormat::Json.to_string(), "json");
}

#[test]
fn display_quiet() {
    assert_eq!(OutputFormat::Quiet.to_string(), "quiet");
}

#[test]
fn equality_and_clone() {
    let a = OutputFormat::Json;
    let b = a;
    assert_eq!(a, b);
    let c = OutputFormat::Table;
    assert_ne!(a, c);
}

#[test]
fn parse_yaml_format() {
    assert_eq!(OutputFormat::parse("yaml").unwrap(), OutputFormat::Yaml);
    assert_eq!(OutputFormat::parse("yml").unwrap(), OutputFormat::Yaml);
}

#[test]
fn display_yaml() {
    assert_eq!(OutputFormat::Yaml.to_string(), "yaml");
}

#[derive(Tabled, Serialize)]
struct Row {
    name:  &'static str,
    value: u32
}

#[test]
fn render_table_contains_headers_and_values() {
    let rows = vec![
        Row {
            name:  "alpha",
            value: 1
        },
        Row {
            name:  "beta",
            value: 2
        },
    ];
    let out = render_table(&rows);
    assert!(out.contains("name"));
    assert!(out.contains("value"));
    assert!(out.contains("alpha"));
    assert!(out.contains("beta"));
}

#[test]
fn serialized_json_returns_pretty_json() {
    let row = Row {
        name:  "alpha",
        value: 7
    };
    let out = serialized(OutputFormat::Json, &row)
        .expect("json returns Some")
        .expect("serialization succeeds");
    assert!(out.contains("\"name\": \"alpha\""));
    assert!(out.contains("\"value\": 7"));
}

#[test]
fn serialized_yaml_returns_yaml() {
    let row = Row {
        name:  "beta",
        value: 9
    };
    let out = serialized(OutputFormat::Yaml, &row)
        .expect("yaml returns Some")
        .expect("serialization succeeds");
    assert!(out.contains("name: beta"));
    assert!(out.contains("value: 9"));
}

#[test]
fn serialized_table_and_quiet_return_none() {
    let row = Row {
        name:  "gamma",
        value: 0
    };
    assert!(serialized(OutputFormat::Table, &row).is_none());
    assert!(serialized(OutputFormat::Quiet, &row).is_none());
}

struct FailingSerialize;

impl Serialize for FailingSerialize {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        Err(serde::ser::Error::custom("intentional serialization failure"))
    }
}

#[test]
fn serialized_json_propagates_error() {
    let result = serialized(OutputFormat::Json, &FailingSerialize).expect("json returns Some");
    assert!(result.is_err());
}

#[test]
fn serialized_yaml_propagates_error() {
    let result = serialized(OutputFormat::Yaml, &FailingSerialize).expect("yaml returns Some");
    assert!(result.is_err());
}
