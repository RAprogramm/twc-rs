set shell := ["sh", "-cu"]

# List available recipes.
default:
    @just --list

# Build the binary with all features.
build:
    cargo build --all-features

# Run the CLI, forwarding any extra arguments.
run *ARGS:
    cargo run -- {{ARGS}}

# Run the interactive TUI dashboard.
dash:
    cargo run --features tui -- dashboard

# Run the test suite with all features.
test:
    cargo test --all-features

# Format all code with the nightly toolchain.
fmt:
    cargo +nightly fmt --all

# Check formatting without writing changes.
fmt-check:
    cargo +nightly fmt --all --check

# Lint with clippy; warnings are errors.
lint:
    cargo clippy --all-features --all-targets -- -D warnings

# Full pre-push gate: formatting, lints, and tests.
check: fmt-check lint test

# Generate a coverage report (requires cargo-llvm-cov).
cov:
    cargo llvm-cov --all-features --workspace

# Print a shell completion script for SHELL (bash, zsh, fish, ...).
completions SHELL:
    cargo run -- completions {{SHELL}}

# Verify the crate builds on the declared MSRV.
msrv:
    cargo +1.96.0 check --all-features
