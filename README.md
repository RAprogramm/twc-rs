<!--
SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
SPDX-License-Identifier: MIT
-->

# twc-rs

A fast, single-binary CLI **and interactive TUI dashboard** for managing
[Timeweb Cloud](https://timeweb.cloud) infrastructure — servers, databases,
S3, Kubernetes, balancers, domains, firewalls and more.

Written in Rust. No Python, no `pip`, no virtualenv — just one static binary.

## Why twc-rs

The official Timeweb CLI ([`timeweb-cloud/twc`](https://github.com/timeweb-cloud/twc))
is a Python application. `twc-rs` matches its command coverage and beats it on
speed, footprint, and experience — every number below is measured, not
estimated (see [docs/COMPARISON.md](docs/COMPARISON.md) for the full report and
the reproducible benchmark).

| | `twc-rs` (Rust) | Official `twc` (Python) |
|---|---|---|
| Cold start (`--version`) | **2.3 ms** | 357 ms |
| Cold start (`--help`) | **2.1 ms** | 347 ms |
| Peak memory (RSS) | **13.7 MB** | 59.2 MB |
| Footprint | 1 static binary, 15 MB stripped | 33 MB packages + a Python interpreter |
| Runtime deps | system libc only | Python + 15 PyPI packages |
| Command coverage | near-full parity | baseline |
| Interactive dashboard | **yes** — full TUI with live metrics | no |
| Shell completions | bash, zsh, fish, powershell, elvish, **nushell** | bash, zsh, fish, powershell |
| Output formats | table, json, yaml, quiet | default, raw, json, yaml |
| Profiles (multi-account) | yes (`--profile`) | yes |
| Languages | **English + Russian** (TUI & CLI) | English only |

Measured on an AMD Ryzen AI MAX+ 395 (Linux 7.1, rustc 1.96) against
`twc-cli` v2.15.2, 50 runs each. The Python tool pays ~350 ms of interpreter
and import startup before any application code runs.

### The dashboard (`twc-rs dashboard`)

The headline feature the Python CLI does not have: a live, k9s-style TUI.

- Flat, discoverable navigation: `h/l` switch tabs, `j/k` move, `Enter` opens
  actions or drills into a resource, `/` filters the list, `Q` quits.
- `Ctrl+K` command palette — fuzzy-run actions, toggle widgets, switch theme.
- Context action menu per resource (reboot / shutdown / clone / delete, with a
  confirmation step for destructive actions).
- Drill into a project to see the resources it contains.
- Live event log surfacing actions and load failures.
- Customizable layout (toggle widgets) and 4 themes (Gruvbox, Catppuccin),
  persisted to the config file.
- Sparkline metrics, status chips, skeleton loaders — all true-color.
- Data is fetched off the UI thread, so input never blocks on the network.

## Install

```sh
cargo install --path .          # from a checkout
# or build:
cargo build --release           # target/release/twc-rs
```

The interactive dashboard needs the `tui` feature:

```sh
cargo build --release --features tui
```

## Authenticate

```sh
twc-rs auth flow                # guided browser flow, stores in the OS keyring
# or
twc-rs config set-token --token <TOKEN>
```

The token is read from the OS keyring, the config file, `--token`, or the
`TWC_TOKEN` environment variable, in that order.

## Usage

```sh
twc-rs server list
twc-rs database info --id 12345
twc-rs project list -f json
twc-rs dashboard                # interactive TUI
```

Global flags: `-f, --format <table|json|quiet>` (env `TWC_OUTPUT`) and
`-t, --token <TOKEN>` (env `TWC_TOKEN`).

### Shell completions

```sh
twc-rs completions nushell > ~/.config/nushell/completions/twc-rs.nu
twc-rs completions zsh   > ~/.zfunc/_twc-rs
twc-rs completions bash  > /etc/bash_completion.d/twc-rs
```

## License

MIT © RAprogramm
