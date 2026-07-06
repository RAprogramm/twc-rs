<!--
SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
SPDX-License-Identifier: MIT
-->

<a id="top"></a>

<div align="center">

# twc-rs

**A fast, single-binary CLI _and_ interactive TUI dashboard for [Timeweb Cloud](https://timeweb.cloud).**

Servers, databases, S3, Kubernetes, balancers, domains, firewalls and more —
managed from one native binary. No Python, no `pip`, no virtualenv.

[![crates.io](https://img.shields.io/crates/v/twc-rs.svg?logo=rust&color=fc8d62)](https://crates.io/crates/twc-rs)
[![downloads](https://img.shields.io/crates/d/twc-rs.svg?color=brightgreen)](https://crates.io/crates/twc-rs)
[![CI](https://github.com/RAprogramm/twc-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/RAprogramm/twc-rs/actions/workflows/ci.yml)
[![Security](https://github.com/RAprogramm/twc-rs/actions/workflows/security.yml/badge.svg)](https://github.com/RAprogramm/twc-rs/actions/workflows/security.yml)
[![release-plz](https://github.com/RAprogramm/twc-rs/actions/workflows/release-plz.yml/badge.svg)](https://github.com/RAprogramm/twc-rs/actions/workflows/release-plz.yml)
[![license](https://img.shields.io/crates/l/twc-rs.svg?color=blue)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.96-blue.svg?logo=rust)](Cargo.toml)
[![platforms](https://img.shields.io/badge/platforms-linux%20%7C%20macos%20%7C%20windows-informational?logo=linux)](#supported-platforms)

**English** · [Русский](README.ru.md)

</div>

---

## Table of contents

- [Why twc-rs](#why-twc-rs)
- [The dashboard](#the-dashboard)
- [Install](#install)
  - [Direct downloads](#direct-downloads)
  - [Supported platforms](#supported-platforms)
- [Authenticate](#authenticate)
- [Usage](#usage)
- [Shell completions](#shell-completions)
- [Benchmarks](#benchmarks)
- [Building from source](#building-from-source)
- [License](#license)

## Why twc-rs

<a id="why-twc-rs"></a>

The official Timeweb CLI ([`timeweb-cloud/twc`](https://github.com/timeweb-cloud/twc))
is a Python application. `twc-rs` matches its command coverage and beats it on
speed, footprint, and experience — every number below is **measured, not
estimated** (see [docs/COMPARISON.md](docs/COMPARISON.md) for the full report
and the reproducible benchmark).

| | `twc-rs` (Rust) | Official `twc` (Python) |
|---|---|---|
| Cold start (`--version`) | **2.3 ms** | 357 ms |
| Cold start (`--help`) | **2.1 ms** | 347 ms |
| Peak memory (RSS) | **13.7 MB** | 59.2 MB |
| Footprint | 1 static binary, 15 MB stripped | 33 MB packages + a Python interpreter |
| Runtime deps | system libc only | Python + 15 PyPI packages |
| Command coverage | near-full parity | baseline |
| Interactive dashboard | **yes** — full TUI with live metrics | no |
| Create / delete from the dashboard | **yes** | no |
| Shell completions | bash, zsh, fish, powershell, elvish, **nushell** | bash, zsh, fish, powershell |
| Output formats | table, json, yaml, quiet | default, raw, json, yaml |
| Profiles (multi-account) | yes (`--profile`, switchable in the TUI) | yes |
| Languages | **English + Russian** (TUI & CLI) | English only |

Measured on an AMD Ryzen AI MAX+ 395 (Linux 7.1, rustc 1.96) against
`twc-cli` v2.15.2, 50 runs each. The Python tool pays ~350 ms of interpreter
and import startup before any application code runs.

<p align="right"><a href="#top">↑ back to top</a></p>

## The dashboard

<a id="the-dashboard"></a>

The headline feature the Python CLI does not have: a live, k9s-style TUI
(`twc-rs dashboard`).

| Key | Action |
|---|---|
| `h` / `l` | switch resource tabs |
| `j` / `k` | move selection |
| `Enter` | open the action menu / drill into a resource |
| `n` | create a new resource (where supported) |
| `/` | filter the current list |
| `Ctrl+K` | command palette — actions, theme, language, profile switch |
| `?` | help overlay |
| `Q` | quit |

- Context action menu per resource (reboot / shutdown / clone / delete) with a
  confirmation step for destructive actions.
- Create resources and **switch account profile** without leaving the dashboard.
- Live per-resource metrics (CPU / RAM / network sparklines), fetched off the
  UI thread so input never blocks on the network.
- Drill into a project to see the resources it contains; live event log.
- Customizable layout, hide-empty-tabs, 4 true-color themes and EN/RU — all
  persisted to the config file.

<p align="right"><a href="#top">↑ back to top</a></p>

## Install

<a id="install"></a>

Pick the channel that fits your platform. Every channel ships with the
interactive TUI dashboard **enabled by default**. For a lean, headless CLI,
build without it: `cargo install twc-rs --no-default-features --features auth`.

| Channel | Command |
|---|---|
| **crates.io** | `cargo install twc-rs` |
| **Installer** (Linux/macOS) | `curl -fsSL https://raw.githubusercontent.com/RAprogramm/twc-rs/main/install.sh \| sh` |
| **Arch (AUR)** | `paru -S twc-rs-bin` |
| **Debian/Ubuntu** | `sudo apt install ./twc-rs_<ver>_amd64.deb` |
| **Releases** | download an archive from [Releases](https://github.com/RAprogramm/twc-rs/releases), verify the `.sha256`, put `twc-rs` on your `PATH` |

### Direct downloads

Prebuilt, checksummed archives are attached to the
**[latest release](https://github.com/RAprogramm/twc-rs/releases/latest)** — pick
your platform (each archive has a matching `.sha256`):

| Platform | Archive |
|---|---|
| 🐧 Linux `x86_64` | [`twc-rs-*-x86_64-unknown-linux-gnu.tar.gz`](https://github.com/RAprogramm/twc-rs/releases/latest) |
| 🐧 Linux `aarch64` | [`twc-rs-*-aarch64-unknown-linux-gnu.tar.gz`](https://github.com/RAprogramm/twc-rs/releases/latest) |
| 🍎 macOS Intel | [`twc-rs-*-x86_64-apple-darwin.tar.gz`](https://github.com/RAprogramm/twc-rs/releases/latest) |
| 🍎 macOS Apple Silicon | [`twc-rs-*-aarch64-apple-darwin.tar.gz`](https://github.com/RAprogramm/twc-rs/releases/latest) |
| 🪟 Windows `x86_64` | [`twc-rs-*-x86_64-pc-windows-msvc.zip`](https://github.com/RAprogramm/twc-rs/releases/latest) |
| 📦 Debian/Ubuntu | [`twc-rs_*_amd64.deb`](https://github.com/RAprogramm/twc-rs/releases/latest) |

The one-line installer detects your OS/arch, downloads the matching tarball
from the latest GitHub release and installs to `~/.local/bin` (or
`/usr/local/bin` when writable). The `.deb` package is attached to every tagged
release automatically.

> `twc-rs` is **not** in the official Debian/Ubuntu (`apt install twc-rs`) or
> Arch (`pacman -S twc-rs`) repositories — those require distro maintainership.
> Use the AUR package, the `.deb`, the installer, or `cargo install`.

### Supported platforms

<a id="supported-platforms"></a>

Every tagged release ships prebuilt, checksummed binaries for:

| OS | Architectures |
|---|---|
| Linux (glibc) | `x86_64`, `aarch64` |
| macOS | `x86_64` (Intel), `aarch64` (Apple Silicon) |
| Windows | `x86_64` |

<p align="right"><a href="#top">↑ back to top</a></p>

## Authenticate

<a id="authenticate"></a>

```sh
twc-rs auth flow                       # guided browser flow, stored in the OS keyring
# or
twc-rs config set-token --token <TOKEN>
```

The token is resolved from the OS keyring, the config file, `--token`, or the
`TWC_TOKEN` environment variable, in that order. Multiple accounts are
supported via named profiles:

```sh
twc-rs config set-token --profile staging --token <TOKEN>
twc-rs --profile staging server list
```

<p align="right"><a href="#top">↑ back to top</a></p>

## Usage

<a id="usage"></a>

Every resource type is a subcommand; run `twc-rs <group> --help` to see its
actions and flags.

```sh
twc-rs server list                        # list cloud servers
twc-rs server info --id 12345             # server details
twc-rs database list -f json              # JSON output
twc-rs ssh attach --server 12345 --key 42 # attach an SSH key to a server
twc-rs project resources --id 678         # drill into a project
twc-rs apps logs my-api --today           # app runtime logs by name or ID (also --since, --tail)
twc-rs apps list-deploys my-api           # deploy history, newest first
twc-rs apps deploy-logs my-api            # build/deploy logs of the latest deploy
twc-rs doctor                             # detect conflicting installs in PATH
twc-rs dashboard                          # interactive TUI (k9s-style)
```

Resource groups: `server`, `database`, `s3`, `kubernetes`, `registry`,
`balancer`, `domain`, `firewall`, `apps`, `image`, `ip`, `vpc`, `ssh`,
`project`, `account`. The full command coverage versus the official CLI is in
[docs/COMPARISON.md](docs/COMPARISON.md).

Global flags:

| Flag | Env | Meaning |
|---|---|---|
| `-f, --format <table\|json\|yaml\|quiet>` | `TWC_OUTPUT` | output format (default `table`) |
| `-t, --token <TOKEN>` | `TWC_TOKEN` | API token override |
| `--profile <NAME>` | `TWC_PROFILE` | use a named profile for multi-account setups |

<p align="right"><a href="#top">↑ back to top</a></p>

## Shell completions

<a id="shell-completions"></a>

```sh
twc-rs completions nushell > ~/.config/nushell/completions/twc-rs.nu
twc-rs completions zsh     > ~/.zfunc/_twc-rs
twc-rs completions bash    > /etc/bash_completion.d/twc-rs
```

Supported shells: `bash`, `zsh`, `fish`, `powershell`, `elvish`, `nushell`. The
AUR package ships completions for `bash`, `zsh`, `fish` and `nushell` in the
standard vendor directories, so they work out of the box.

### Dynamic completions

Static scripts complete commands and flags; the **dynamic** engine also
completes live values — `twc-rs apps logs <TAB>` offers your actual apps by
name and ID, fetched from the API (silently skipped when offline). Register it
instead of (or on top of) the static script:

| Shell | Add to |
|---|---|
| bash | `echo 'source <(COMPLETE=bash twc-rs)' >> ~/.bashrc` |
| zsh | `echo 'source <(COMPLETE=zsh twc-rs)' >> ~/.zshrc` |
| fish | `echo 'COMPLETE=fish twc-rs \| source' >> ~/.config/fish/config.fish` |
| elvish | `echo 'eval (E:COMPLETE=elvish twc-rs \| slurp)' >> ~/.elvish/rc.elv` |
| powershell | `$env:COMPLETE = "powershell"; twc-rs \| Out-String \| Invoke-Expression; Remove-Item Env:\COMPLETE` in `$PROFILE` |

Nushell keeps the static script (the dynamic engine does not support it yet).


<p align="right"><a href="#top">↑ back to top</a></p>

## Benchmarks

<a id="benchmarks"></a>

All performance claims are reproducible. [docs/COMPARISON.md](docs/COMPARISON.md)
documents the test environment and methodology; rerun it with:

```sh
cargo build --release --features tui
python3 -m venv /tmp/twcbench && /tmp/twcbench/bin/pip install twc-cli
benches/compare.sh ./target/release/twc-rs /tmp/twcbench/bin/twc
```

Prefer CI? The **[Benchmarks](../../actions/workflows/benchmarks.yml)** workflow
runs the same comparison on demand (Actions → Benchmarks → *Run workflow*) and
prints the head-to-head table in the run summary. It is manual only — never on
push — so it never slows down regular CI.

<p align="right"><a href="#top">↑ back to top</a></p>

## Building from source

<a id="building-from-source"></a>

```sh
git clone https://github.com/RAprogramm/twc-rs && cd twc-rs
cargo build --release                   # full binary (TUI dashboard is on by default)
cargo install --path .                  # install from the checkout
# headless, no TUI:
cargo build --release --no-default-features --features auth
```

Requires Rust **1.96+**. The crate is linted under `clippy` pedantic + nursery
and the SDK is generated from the official OpenAPI spec via
[`timeweb-rs`](https://crates.io/crates/timeweb-rs).

<p align="right"><a href="#top">↑ back to top</a></p>

## License

<a id="license"></a>

MIT © RAprogramm

<p align="right"><a href="#top">↑ back to top</a></p>
