# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.10.0](https://github.com/RAprogramm/twc-rs/compare/v0.9.1...v0.10.0) - 2026-07-11

### Added

- Enrich app details and auto-select first non-empty tab (#117) ([#116](https://github.com/RAprogramm/twc-rs/issues/116)) ([5154c14](https://github.com/RAprogramm/twc-rs/commit/5154c14140008d651dab7b322294acea74b676ff))


## [0.9.0](https://github.com/RAprogramm/twc-rs/compare/v0.8.1...v0.9.0) - 2026-07-06

### Added

- Add update command with install-channel detection (#109) ([#108](https://github.com/RAprogramm/twc-rs/issues/108)) ([8ccca35](https://github.com/RAprogramm/twc-rs/commit/8ccca3598ec438ead1fed577b003b6bbe829a866))


## [0.8.1](https://github.com/RAprogramm/twc-rs/compare/v0.8.0...v0.8.1) - 2026-07-06

### Added

- GitHub Pages book with docgen-generated CLI reference (#102) ([#101](https://github.com/RAprogramm/twc-rs/issues/101)) ([de1b99e](https://github.com/RAprogramm/twc-rs/commit/de1b99e0636773a71d6e7f7c7a71c61d5406bce1))


### Fixed

- Classify changelog entries despite issue-number commit prefix (#105) ([#104](https://github.com/RAprogramm/twc-rs/issues/104)) ([7e4fa07](https://github.com/RAprogramm/twc-rs/commit/7e4fa07a93a61cd071652349ad5c9bf3583fd87c))


## [0.8.0](https://github.com/RAprogramm/twc-rs/compare/v0.7.0...v0.8.0) - 2026-07-06

### Other

- #99 docs: embed demo GIFs into README sections ([#100](https://github.com/RAprogramm/twc-rs/pull/100)) ([d17c8d4](https://github.com/RAprogramm/twc-rs/commit/d17c8d4385445b3e5303883328b14f6c83a2ffc6))

- #97 docs: refresh examples for name selectors and dynamic completion ([#98](https://github.com/RAprogramm/twc-rs/pull/98)) ([de85486](https://github.com/RAprogramm/twc-rs/commit/de85486f78532ceff568f8588e31fb02f132ce88))

- #93 feat: dynamic shell completion serving live app names and IDs ([#96](https://github.com/RAprogramm/twc-rs/pull/96)) ([daf88a2](https://github.com/RAprogramm/twc-rs/commit/daf88a2c0cc05db61f028bc3a2b733d749034c36))

- #92 feat: report last log timestamp when date filter yields nothing ([#95](https://github.com/RAprogramm/twc-rs/pull/95)) ([8844fed](https://github.com/RAprogramm/twc-rs/commit/8844fed50612079383bf37a9736bf442659a9964))

- #91 feat: address apps by name or numeric ID via positional selector ([#94](https://github.com/RAprogramm/twc-rs/pull/94)) ([c9a4a6e](https://github.com/RAprogramm/twc-rs/commit/c9a4a6e67df645b72d82a90ed396a372799a1165))

- #88 fix: stamp JSON log lines via embedded RFC 3339 timestamp fallback ([#89](https://github.com/RAprogramm/twc-rs/pull/89)) ([6a6012c](https://github.com/RAprogramm/twc-rs/commit/6a6012c0ea3cfd014e87b1c12fcd364db34405fc))

- #86 feat: colorize help output and group global flags last ([#87](https://github.com/RAprogramm/twc-rs/pull/87)) ([3497410](https://github.com/RAprogramm/twc-rs/commit/3497410bfa46692278b758e15bccd3b7b693fd82))

- #84 docs: document apps logs, list-deploys, deploy-logs and doctor commands ([#85](https://github.com/RAprogramm/twc-rs/pull/85)) ([36a3117](https://github.com/RAprogramm/twc-rs/commit/36a3117584fca4eef1c36c11a0ff87c98b2c3d20))


## [0.7.0](https://github.com/RAprogramm/twc-rs/compare/v0.6.0...v0.7.0) - 2026-07-05

### Other

- #79 feat: add doctor command detecting conflicting installations in PATH ([#80](https://github.com/RAprogramm/twc-rs/pull/80)) ([a5d2d80](https://github.com/RAprogramm/twc-rs/commit/a5d2d8023dc3a13dd73ab68620018532d0d702b5))


## [0.6.0](https://github.com/RAprogramm/twc-rs/compare/v0.5.1...v0.6.0) - 2026-07-05

### Other

- #77 feat: add apps logs, list-deploys and deploy-logs commands ([#78](https://github.com/RAprogramm/twc-rs/pull/78)) ([da6d17c](https://github.com/RAprogramm/twc-rs/commit/da6d17c5b9ed0429329f730cbb4e2860699d46b1))

- #75 fix: adopt untracked completion files via backup array ([#76](https://github.com/RAprogramm/twc-rs/pull/76)) ([447fc9d](https://github.com/RAprogramm/twc-rs/commit/447fc9db86b00a6a3b89a4859706dadc3e67467d))

- #73 fix: drop pre_upgrade scriptlet and document one-time AUR upgrade step ([#74](https://github.com/RAprogramm/twc-rs/pull/74)) ([5fc2d56](https://github.com/RAprogramm/twc-rs/commit/5fc2d56ecea15c47550d3772870a99477c3cbc7b))

- #70 fix: install completions into shell vendor paths and package LICENSE ([#71](https://github.com/RAprogramm/twc-rs/pull/71)) ([6c6fc19](https://github.com/RAprogramm/twc-rs/commit/6c6fc193b719cdd451145d2fff793ce85c1dd50d))


## [0.5.1](https://github.com/RAprogramm/twc-rs/compare/v0.5.0...v0.5.1) - 2026-07-03

### Other

- #67 fix: map real database engine into the CLI list table ([1014454](https://github.com/RAprogramm/twc-rs/commit/1014454bcf6329bbd3d8d09cc3ee62a8258de35f))


## [0.5.0](https://github.com/RAprogramm/twc-rs/compare/v0.4.0...v0.5.0) - 2026-07-03

### Other

- #65 refactor: move table row types into per-resource rows modules ([52ccd04](https://github.com/RAprogramm/twc-rs/commit/52ccd047d85950224bcf6097d0aaa6f4375fed97))

- #63 refactor: split details renderers and ui overlays into submodules ([0a074f6](https://github.com/RAprogramm/twc-rs/commit/0a074f65b65d8e8043e2d78c1f0499f9fe392cb6))

- #61 refactor: split cli.rs into per-resource subcommand modules ([cb3cc16](https://github.com/RAprogramm/twc-rs/commit/cb3cc16954f4bd136af915f7b369d2876f3d35cb))

- #59 refactor: split tui app state into focused submodules ([f3f5d66](https://github.com/RAprogramm/twc-rs/commit/f3f5d663161436a3bbabf641b68cb5a30ffa8f10))

- #57 refactor: split main.rs into auth_cli, cli_dispatch and dashboard modules ([ecb762c](https://github.com/RAprogramm/twc-rs/commit/ecb762ce311af5eb281100a4ec83ce6607ecbbbb))

- #54 fix: split ssh key body reading from upload so tests never block on stdin ([c67f501](https://github.com/RAprogramm/twc-rs/commit/c67f5018afdfa7e5e0e3bc6310b8a760ee5be22b))


## [0.4.0](https://github.com/RAprogramm/twc-rs/compare/v0.3.8...v0.4.0) - 2026-07-03

### Other

- #52 feat: count and drill project apps via timeweb-rs 0.5 ([9db2eeb](https://github.com/RAprogramm/twc-rs/commit/9db2eebbf089d9ebbf3a3986a517a85073490b63))

- #50 fix: count all project resource types instead of servers only ([144d823](https://github.com/RAprogramm/twc-rs/commit/144d823dca5fd07003d5f87d3eb97eba640d8a1c))

- #42 feat: show active profile in header, add profile switcher key, fix stale token info after switch ([94f1ce5](https://github.com/RAprogramm/twc-rs/commit/94f1ce5b0da5dce06939b6fbded238a671c193c6))

- #41 feat: database backup action, delete for domains, firewall, floating ips, images, network drives and vpcs, feedback on unsupported enter and create ([4703948](https://github.com/RAprogramm/twc-rs/commit/4703948fa66e44f58f4f7038b60b8d1ceb61393c))

- #46 fix: remove databases restart command that deleted the cluster ([e31d555](https://github.com/RAprogramm/twc-rs/commit/e31d55559d8c6bc6f202e9dd0707309c143531ee))

- #40 fix: correct stats timestamp format, surface stats errors in events log, refresh sparklines every 30s ([ae11eee](https://github.com/RAprogramm/twc-rs/commit/ae11eeedb4776e85064d468616a4cb40c3fc0580))

- #39 fix: paginate all dashboard list fetches and surface load failures instead of fake empty states ([d487c81](https://github.com/RAprogramm/twc-rs/commit/d487c81b7bdbfb4953af7e3f6b7742d93393fa45))

- #37 fix: floating ip binding info, real account status, status-aware chip colors, localized list title ([40c9e0b](https://github.com/RAprogramm/twc-rs/commit/40c9e0b743087e6af3535781d1453e6bafb80b65))

- #37 fix: honest dashboard fields for k8s, registry, firewall, vpc, dedicated, mail, apps, ai agents, knowledge bases and real project server counts ([1665a98](https://github.com/RAprogramm/twc-rs/commit/1665a987f9e8c925f913d4a7abb35d7d48f782ef))

- #37 fix: map real server ip and disk, database size, s3 objects in dashboard ([019c594](https://github.com/RAprogramm/twc-rs/commit/019c594a4540a04c690e8ffefa07abdd955bf649))


## [0.3.8](https://github.com/RAprogramm/twc-rs/compare/v0.3.7...v0.3.8) - 2026-07-01

### Documentation

- Add funding info ([#36](https://github.com/RAprogramm/twc-rs/pull/36)) ([3bd63e6](https://github.com/RAprogramm/twc-rs/commit/3bd63e6b32d3dfb083b0d446230600fc3f2a25b5))

- Point documentation link to docs.rs ([#35](https://github.com/RAprogramm/twc-rs/pull/35)) ([eb26e48](https://github.com/RAprogramm/twc-rs/commit/eb26e48c811236e23842987e51232ed3db289a6d))


### Other

- 32 ([#33](https://github.com/RAprogramm/twc-rs/pull/33)) ([856758e](https://github.com/RAprogramm/twc-rs/commit/856758e828060a4acf91eab8de6e34becc96a1ee))


## [0.3.7](https://github.com/RAprogramm/twc-rs/compare/v0.3.6...v0.3.7) - 2026-07-01

### Documentation

- Add direct download links and a Russian README with cross-links ([#30](https://github.com/RAprogramm/twc-rs/pull/30)) ([8bda9b4](https://github.com/RAprogramm/twc-rs/commit/8bda9b43edc5214c2d6da836216ecc0ee9654c0a))


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
