// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! CPU micro-benchmarks for the offline hot paths of twc-rs: CLI argument
//! parsing, JWT claim decoding, table rendering, JSON/YAML serialization and
//! config (de)serialization. Run with `cargo bench --bench core`, or under
//! `CodSpeed` with `cargo codspeed build && cargo codspeed run`.

use clap::Parser;
use divan::{Bencher, black_box};
use serde::Serialize;
use tabled::Tabled;
use twc_rs::{
    cli::Cli,
    config::AppConfig,
    jwt::JwtPayload,
    output::{OutputFormat, render_table, serialized}
};

fn main() {
    divan::main();
}

/// A representative list-command row, mirroring the server list table.
#[derive(Tabled, Serialize, Clone)]
struct Row {
    #[tabled(rename = "ID")]
    id:       String,
    #[tabled(rename = "Name")]
    name:     String,
    #[tabled(rename = "Status")]
    status:   String,
    #[tabled(rename = "CPU")]
    cpu:      String,
    #[tabled(rename = "RAM (MB)")]
    ram:      String,
    #[tabled(rename = "OS")]
    os:       String,
    #[tabled(rename = "Location")]
    location: String
}

fn rows(n: usize) -> Vec<Row> {
    (0..n)
        .map(|i| Row {
            id:       (1_000_000 + i).to_string(),
            name:     format!("web-server-{i:04}"),
            status:   if i % 3 == 0 { "off" } else { "on" }.to_string(),
            cpu:      (1 + i % 8).to_string(),
            ram:      (1024 * (1 + i % 16)).to_string(),
            os:       "ubuntu 24.04".to_string(),
            location: ["ru-1", "ru-2", "nl-1", "kz-1"][i % 4].to_string()
        })
        .collect()
}

const SIZES: [usize; 3] = [10, 100, 1000];

mod cli_parse {
    use super::{Cli, Parser, black_box};

    #[divan::bench]
    fn server_list() {
        black_box(
            Cli::try_parse_from(black_box(["twc-rs", "server", "list", "--limit", "50"]))
                .expect("valid arguments")
        );
    }

    #[divan::bench]
    fn server_create() {
        black_box(
            Cli::try_parse_from(black_box([
                "twc-rs",
                "--format",
                "json",
                "server",
                "create",
                "--name",
                "web-01",
                "--preset-id",
                "4795",
                "--os-id",
                "99",
                "--ssh-key",
                "1",
                "--ssh-key",
                "2",
                "--availability-zone",
                "spb-1"
            ]))
            .expect("valid arguments")
        );
    }
}

mod jwt {
    use super::{JwtPayload, black_box};

    const TOKEN: &str = "eyJhbGciOiJSUzUxMiIsInR5cCI6IkpXVCJ9.\
        eyJzdWIiOiJhYjEyMzQ1NiIsInNjb3BlIjoicmVhZCB3cml0ZSIsImlhdCI6MTcwMDAwMDAwMCwiZXhwIjo0MTAyNDQ0ODAwfQ.\
        c2lnbmF0dXJl";

    #[divan::bench]
    fn parse() -> JwtPayload {
        JwtPayload::parse(black_box(TOKEN))
    }
}

mod output {
    use super::{Bencher, OutputFormat, SIZES, black_box, render_table, rows, serialized};

    #[divan::bench(args = SIZES)]
    fn table(bencher: Bencher, n: usize) {
        let data = rows(n);
        bencher.bench_local(|| render_table(black_box(&data)));
    }

    #[divan::bench(args = SIZES)]
    fn json(bencher: Bencher, n: usize) {
        let data = rows(n);
        bencher.bench_local(|| serialized(OutputFormat::Json, black_box(&data)));
    }

    #[divan::bench(args = SIZES)]
    fn yaml(bencher: Bencher, n: usize) {
        let data = rows(n);
        bencher.bench_local(|| serialized(OutputFormat::Yaml, black_box(&data)));
    }

    #[divan::bench]
    fn parse_format() {
        for s in ["table", "JSON", "yml", "q", "bogus"] {
            black_box(OutputFormat::parse(black_box(s))).ok();
        }
    }
}

mod config {
    use super::{AppConfig, Bencher, black_box};

    fn sample() -> AppConfig {
        let mut cfg = AppConfig {
            token: Some("eyJhbGciOiJSUzUxMiJ9.e30.sig".to_string()),
            default_region: Some("ru-1".to_string()),
            ..AppConfig::default()
        };
        for i in 0..20 {
            cfg.profiles
                .insert(format!("profile-{i}"), format!("token-{i:032}"));
        }
        cfg
    }

    #[divan::bench]
    fn serialize_toml(bencher: Bencher) {
        let cfg = sample();
        bencher.bench_local(|| toml::to_string(black_box(&cfg)));
    }

    #[divan::bench]
    fn deserialize_toml(bencher: Bencher) {
        let text = toml::to_string(&sample()).expect("serialize sample config");
        bencher.bench_local(|| toml::from_str::<AppConfig>(black_box(&text)));
    }
}
