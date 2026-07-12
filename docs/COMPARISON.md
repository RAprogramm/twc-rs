<!--
SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
SPDX-License-Identifier: MIT
-->

# twc-rs vs the official Timeweb CLI — measured comparison

A head-to-head against the official Python CLI
[`timeweb-cloud/twc`](https://github.com/timeweb-cloud/twc). Every performance
number below is produced live by [`benches/compare.sh`](../benches/compare.sh)
— nothing here is hand-written or estimated. Re-run it yourself to reproduce.

## Test environment

| | |
|---|---|
| CPU | AMD Ryzen AI MAX+ 395 (32 threads) |
| Memory | 62 GiB |
| OS / kernel | Linux 7.1.2 (x86_64) |
| Rust toolchain | stable Rust (release build, `--features tui`, stripped) |
| Official CLI | `twc-cli` v2.15.2, in a clean virtualenv on Python 3.14.6 |
| Sampling | 50 runs per measurement, mean reported, one warm-up discarded |
| Startup metric | `--version` / `--help` (no network — pure process start) |
| Memory metric | peak RSS via `getrusage(RUSAGE_CHILDREN)` |

Reproduce:

```sh
cargo build --release --features tui
python3 -m venv /tmp/twcbench && /tmp/twcbench/bin/pip install twc-cli
benches/compare.sh ./target/release/twc-rs /tmp/twcbench/bin/twc
```

## Performance — twc-rs wins on every axis

| Metric | twc-rs | official `twc` | Advantage |
|---|---|---|---|
| Cold start (`--version`) | **2.3 ms** | 357.3 ms | **≈155× faster** |
| Cold start (`--help`) | **2.1 ms** | 347.4 ms | **≈165× faster** |
| Peak memory (RSS) | **13.7 MB** | 59.2 MB | **≈4.3× less** |
| On-disk size | **15 MB**, one static binary | 33 MB of packages + a Python interpreter | smaller, self-contained |
| Runtime dependencies | system libc only | 15 PyPI packages + CPython | none to install |
| Install | copy one file | `pip` + Python toolchain | no runtime needed |

The Python tool spends ~350 ms loading the interpreter and importing its
dependency tree before any application code runs; `twc-rs` is a native binary,
so the same invocation completes in single-digit milliseconds.

## Capabilities only twc-rs has

| Feature | twc-rs | official |
|---|---|---|
| Interactive TUI dashboard (k9s-style) | yes | no |
| Live per-resource metrics (CPU / RAM / network) | yes | no |
| Create / delete resources from the dashboard | yes | no |
| Switch account profile from the dashboard | yes | no |
| Localization — English + Russian (CLI & TUI) | yes | English only |
| Named profiles (`--profile`) | yes | yes |
| Installation self-check (`doctor` finds conflicting copies in `PATH`) | yes | no |
| Address apps by name, not just numeric ID | yes | no |
| Dynamic shell completion of live resource values (app names/IDs from the API) | yes | no |
| Shell completions | bash, zsh, fish, powershell, elvish, **nushell** | bash, zsh, fish, powershell |

## Command coverage

Near-complete parity with the official CLI across all resource groups
(accounting for naming: `ssh-key`≈`ssh`, `storage`≈`s3`, `cluster`≈`kubernetes`,
`balancer backend`≈`balancer ip-*`):

| Group | Status |
|---|---|
| account | full — status, finances (`show`), access restrictions |
| apps | full — list, info, create, delete, presets, repositories, vcs-providers, logs (`--tail`/`--since`/`--today`), list-deploys, deploy-logs; per-app commands accept the app name or ID |
| balancer | full — list/info/create/update/delete, rules, IPs (backends), presets |
| database | full — list/info/create/update/delete, backups, users, presets, types, instances |
| domain | full — list/info/add/delete, DNS records, subdomains, name servers |
| firewall | full — groups, rules, resource link/unlink |
| image | full — list, info, create, set, delete, upload |
| ip | full — list, info, create, attach, detach, set, delete |
| kubernetes | full — clusters, node groups, nodes, addons, presets, versions, network drivers |
| project | full — list, create, set, delete, resources |
| server | list, info, create, set, clone, delete, reboot, start, shutdown, reset-password, resize, reinstall, disk, ip, history, backup-list, backup-create, set-nat-mode, set-boot-mode, list-presets/os/software/configurators |
| ssh-key | full — list, add (upload), info, edit, delete, attach/detach key to a server |
| storage (s3) | full — buckets, users, subdomains, transfer, presets, genconfig |
| vpc | full — list, info, create, set, delete, ports |

The single uncovered command is `server vnc` — Timeweb's API has no VNC
endpoint at all (it is absent from the OpenAPI spec), so there is nothing to
call. `image upload` was missing only because the upstream spec omitted the
request body; that defect was fixed in the [`timeweb-rs`](https://crates.io/crates/timeweb-rs)
normalizer (0.3.0) and the command now works. Custom server configurators
(CPU/RAM/disk arrays) remain behind the common preset path. The TUI dashboard
additionally exposes delete / power / clone management for every integer-id
resource.

## Engineering quality

| | |
|---|---|
| Lints | clean under `clippy` **pedantic + nursery**, no `#[allow]` crutches |
| Tasks | non-blocking UI — data fetched off the render thread |
| SDK | generated from the official OpenAPI spec via [`timeweb-rs`](https://crates.io/crates/timeweb-rs); spec defects fixed in a normalizer pass, never by hand-editing generated code |
| Deprecations | zero — migrated to the current v2/cluster endpoints |
