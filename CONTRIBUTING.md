# Contributing to twc-rs

Thanks for your interest in improving twc-rs. This guide covers the local setup,
the development loop, commit conventions, and how to add a new command.

## Prerequisites

- **Rust 1.96+** (the declared MSRV). Install via [rustup](https://rustup.rs/).
- **Nightly rustfmt** for formatting: `rustup toolchain install nightly --component rustfmt`.
- **[`just`](https://github.com/casey/just)** task runner: `cargo install just`.

Optional:

- `cargo install cargo-llvm-cov` for coverage (`just cov`).
- `cargo install cargo-deny` to run the supply-chain checks locally (`cargo deny check`).

## Development loop

Run `just` to see all available recipes. The common ones:

```sh
just build        # cargo build --all-features
just run -- ...    # run the CLI with arguments
just dash          # run the TUI dashboard
just test          # cargo test --all-features
just fmt           # cargo +nightly fmt --all
just lint          # cargo clippy --all-features --all-targets -- -D warnings
just check         # fmt-check + lint + test — the full pre-push gate
just msrv          # cargo +1.96.0 check --all-features
```

Always run `just check` before pushing. CI enforces the same bar
(formatting, clippy with `-D warnings`, tests, and MSRV), so a green
`just check` means a green CI.

## Commit conventions

Commits and PR titles follow [Conventional Commits](https://www.conventionalcommits.org/).
They drive the changelog and the automatic version bump performed by release-plz:

| Prefix      | Meaning                              | Version effect |
| ----------- | ------------------------------------ | -------------- |
| `feat:`     | New user-facing capability           | minor          |
| `fix:`      | Bug fix                              | patch          |
| `perf:`     | Performance improvement              | patch          |
| `refactor:` | Internal change, no behavior change  | patch          |
| `docs:`     | Documentation only                   | none           |
| `test:`     | Tests only                           | none           |
| `chore:`    | Tooling, deps, CI (skipped in log)   | none           |

Append `!` (e.g. `feat!:`) or a `BREAKING CHANGE:` footer for a major bump.
Dependabot uses the `chore` prefix so dependency PRs stay out of the changelog.

## Architecture overview

twc-rs is a single binary crate. Features: `default = ["auth"]` plus `tui`;
the full build uses `--all-features`.

- `src/cli.rs` — the clap CLI. The top-level `Commands` enum holds one variant
  per resource group, each wrapping a per-group subcommand enum
  (e.g. `ServerCommands`, `ProjectCommands`).
- `src/main.rs` — argument parsing and dispatch. It matches on `Commands` and
  calls the relevant handler/command function, passing the resolved
  `OutputFormat` and API `Configuration`.
- `src/commands/<group>.rs` — the implementation for each group
  (e.g. `src/commands/projects.rs`), exposing `pub async fn` entry points like
  `list`, `create`, `delete`. Larger groups have a `src/commands/<group>/`
  directory.
- `src/output.rs` — output rendering. The global `-f/--format` flag selects
  `table` (default), `json`, `yaml`, or `quiet`. Use `output::serialized(format, &value)`
  for the machine-readable formats and `output::render_table(&rows)` for tables.
- `locales/*.yml` — i18n strings (rust-i18n). User-facing text goes through the
  `t!("...")` macro with `en` and `ru` entries. `en` is the fallback.

The Timeweb Cloud SDK comes from the [`timeweb-rs`](https://crates.io/crates/timeweb-rs)
crate. It is OpenAPI-generated: fix spec defects in that crate's spec normalizer,
**never** by hand-editing generated code.

## Adding a new command

1. **Declare the CLI surface** in `src/cli.rs`. Add a variant to the group's
   subcommand enum (or a new group variant in `Commands` plus its enum), with
   `///` doc comments on every variant and argument — clap turns these into the
   `--help` text.
2. **Implement it** in `src/commands/<group>.rs` as a `pub async fn` that takes
   the API `Configuration` and `OutputFormat`, calls the `timeweb_rs` API, and
   renders results via `src/output.rs`.
3. **Wire up dispatch** in `src/main.rs`: match the new variant and call your
   function, forwarding `format`.
4. **Add locale strings** to `locales/*.yml` for any user-facing message, with
   both `en` and `ru` values, and use them via `t!(...)`.
5. **Run `just check`** and fix anything it reports.

## Rust comment rules

Outside `#[cfg(test)] mod tests`, only doc comments are allowed:

- `///` — item documentation.
- `//!` — module documentation.

Plain `//` line comments and `/* */` block comments are **not** allowed in
function bodies or inline. If you feel the need to explain *why* inside a body,
that is a signal to rename or extract a well-named `///`-documented function.

## License

By contributing, you agree that your contributions are licensed under the
project's MIT license.
