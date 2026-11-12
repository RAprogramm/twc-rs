<!--
SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
SPDX-License-Identifier: MIT
-->

# Security Policy

## Reporting a Vulnerability

Please report security vulnerabilities privately through
[GitHub security advisories](https://github.com/RAprogramm/twc-rs/security/advisories/new).
Do not open a public issue for unfixed vulnerabilities. We aim to respond
within 5 business days and to publish a fix or guidance within 30 days,
prioritising by severity.

## Supported Versions

Only the current major release line is supported with security fixes.
Run `twc-rs --version` and upgrade to the latest release before reporting.

## Automated Checks

`cargo deny check` (advisories, bans, licenses, sources) runs on every pull
request, every push to `main`, and on a weekly schedule
([`security.yml`](.github/workflows/security.yml)). The RustSec advisory
database is therefore consulted at least weekly even without code changes.

## Advisory Triage Policy

Every RustSec advisory hitting the dependency graph is triaged as follows:

1. **Vulnerability** (unsoundness, memory safety, injection, etc.): never
   ignored. Upgrade the dependency, or replace it if no fixed version
   exists. Releases are blocked until resolved.
2. **Unmaintained** (no security impact today, upstream stopped maintaining):
   upgrade to a maintained replacement when one exists. If no upgrade path
   exists at the workspace level (the fix is pending upstream), the advisory
   may be listed under `[advisories] ignore` in `deny.toml` only together
   with:
   - a tracking issue stating the exact removal conditions, and
   - a link to that issue in the ignore comment.
3. **License/source violations**: fixed by dependency removal or an explicit,
   reviewed allowance in `deny.toml`.

Every ignore entry is auditable and self-expiring: the scheduled
`ignore-expiry` job re-runs the advisories check with the ignores stripped
and reports on the tracking issue the moment an ignore becomes obsolete.
