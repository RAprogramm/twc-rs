<!--
SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
SPDX-License-Identifier: MIT
-->

# twc-rs vs the official Timeweb CLI — measured comparison

A head-to-head against the official Python CLI
[`timeweb-cloud/twc`](https://github.com/timeweb-cloud/twc). Every performance
number below was produced live with [`benches/compare.sh`](../benches/compare.sh)
— nothing here is estimated.

## Methodology

| | |
|---|---|
| Host | AMD Ryzen AI MAX+ 395, 32 threads, Linux 7.1 (x86_64) |
| twc-rs | release build, `--features tui`, stripped binary |
| Official | `twc-cli` v2.15.2 in a clean venv, Python 3.14 |
| Runs | 50 per measurement, mean reported; one warm-up run discarded |
| Startup metric | `--version` and `--help` (no network, pure process start) |
| Memory | peak RSS via `getrusage(RUSAGE_CHILDREN)` |

Reproduce:

```sh
cargo build --release --features tui
python3 -m venv /tmp/twcbench && /tmp/twcbench/bin/pip install twc-cli
benches/compare.sh ./target/release/twc-rs /tmp/twcbench/bin/twc
```

## Performance — twc-rs wins on every axis

| Metric | twc-rs | official `twc` | Advantage |
|---|---|---|---|
| Cold start (`--version`) | **2.3 ms** | 348.2 ms | **151× faster** |
| Cold start (`--help`) | **2.0 ms** | 344.8 ms | **172× faster** |
| Peak memory (RSS) | **14.0 MB** | 59.1 MB | **4.2× less** |
| On-disk size | **14 MB**, one file | 33 MB of packages + a Python interpreter | smaller, self-contained |
| Runtime dependencies | 12 system libraries | 15 PyPI packages + CPython | fewer, no virtualenv |
| Install | copy one binary | `pip` + Python toolchain | no runtime needed |

The Python tool pays ~300 ms of interpreter and import cost before any
application code runs; `twc-rs` is a native binary, so the same operations
finish in single-digit milliseconds.

## Capabilities only twc-rs has

| Feature | twc-rs | official |
|---|---|---|
| Interactive TUI dashboard (k9s-style) | yes | no |
| Live per-resource metrics (CPU/RAM/network) | yes | no |
| Localization (English + Russian, CLI & TUI) | yes | English only |
| Named profiles (`--profile`) | yes | yes |
| Shell completions | bash, zsh, fish, powershell, elvish, **nushell** | bash, zsh, fish, powershell |

## Command coverage — honest status

The official CLI is currently deeper on raw command count. Mapping subcommands
(accounting for naming differences: `ssh-key`≈`ssh`, `storage`≈`s3`,
`cluster`≈`kubernetes`):

| Group | Covered / official | Notable gaps in twc-rs |
|---|---|---|
| domain | 6 / 6 | — |
| firewall | 5 / 5 | — |
| balancer | 7 / 8 | backend management |
| cluster | 9 / 10 | list-network-drivers |
| storage (s3) | 7 / 8 | genconfig |
| database | 8 / 10 | instance, list-types |
| account | 2 / 3 | access restrictions |
| project | 3 / 5 | resource, set |
| ssh-key | 3 / 6 | edit, get, new |
| apps | 1 / 7 | create, delete, get, presets, repositories, vcs-providers |
| image | 2 / 6 | create, get, set, upload |
| ip | 2 / 7 | attach, detach, create, get, set |
| vpc | 2 / 6 | create, port, set, show |
| server | 4 / 24 | create, clone, resize, reinstall, disk, ip, vnc, backup, power, boot/nat mode, reset-root-password, history, listings |
| **Total** | **~61 / 111 (~55%)** | server depth is the largest gap |

These gaps are tracked and being closed; the SDK already exposes the
corresponding endpoints, so each is additive. This document is updated as
coverage lands.
