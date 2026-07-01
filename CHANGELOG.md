# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.6](https://github.com/RAprogramm/twc-rs/compare/v0.3.5...v0.3.6) - 2026-07-01

### Documentation

- Use the native GitHub status badge for release-plz ([#27](https://github.com/RAprogramm/twc-rs/pull/27)) ([b3a3efe](https://github.com/RAprogramm/twc-rs/commit/b3a3efe62fb081bf7c91d3a9b67bb8d6121ea9ad))


## [0.3.5](https://github.com/RAprogramm/twc-rs/compare/v0.3.4...v0.3.5) - 2026-07-01

### Fixed

- Activate nushell completions via the vendor autoload dir in the AUR package ([#25](https://github.com/RAprogramm/twc-rs/pull/25)) ([e28aa0c](https://github.com/RAprogramm/twc-rs/commit/e28aa0c1fb15bceb9d5c8de283277a4082067457))


## [0.3.4](https://github.com/RAprogramm/twc-rs/compare/v0.3.3...v0.3.4) - 2026-07-01

### Added

- TUI by default + locale-aware AUR completions scriptlet ([#23](https://github.com/RAprogramm/twc-rs/pull/23)) ([0927fba](https://github.com/RAprogramm/twc-rs/commit/0927fba5497cbc5b1f46d80f85b5ab9f9ab3fc06))


## [0.3.2](https://github.com/RAprogramm/twc-rs/compare/v0.3.1...v0.3.2) - 2026-07-01

### Documentation

- Repair duplicated changelog header and add the 0.3.1 entry ([#15](https://github.com/RAprogramm/twc-rs/pull/15)) ([c3d2324](https://github.com/RAprogramm/twc-rs/commit/c3d2324128517a5b1393837890a6ec5c373183ac))


## [0.3.1](https://github.com/RAprogramm/twc-rs/compare/v0.3.0...v0.3.1) - 2026-06-30

### Fixed

- Switch HTTP stack to rustls to drop the OpenSSL C dependency, fixing the aarch64-unknown-linux-gnu release build ([#11](https://github.com/RAprogramm/twc-rs/pull/11))

## [0.3.0](https://github.com/RAprogramm/twc-rs/compare/v0.2.1...v0.3.0) - 2026-06-30

### Fixed

- Close ssh-key parity gap and fix database/apps preset deserialization ([#9](https://github.com/RAprogramm/twc-rs/pull/9)) ([5896245](https://github.com/RAprogramm/twc-rs/commit/5896245253750f6a179a3674f350f78fb03d7023))

- Replace unsound serde_yml with maintained serde_yaml_ng ([edbfc8c](https://github.com/RAprogramm/twc-rs/commit/edbfc8cd469a4c359f01baf76b2656f8fa053185))


### Documentation

- Add Security workflow badge to the README header ([c09fdd6](https://github.com/RAprogramm/twc-rs/commit/c09fdd6c5108787de53aab37ba914ad93f684cbd))


## [0.2.1](https://github.com/RAprogramm/twc-rs/compare/v0.2.0...v0.2.1) - 2026-06-30

### Fixed

- Let release-plz own the release notes, only attach assets ([f02f00d](https://github.com/RAprogramm/twc-rs/commit/f02f00dc813a479d892a88588ad355edf9ba7e52))
