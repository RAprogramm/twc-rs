# twc-rs

**A fast, single-binary CLI and interactive TUI dashboard for
[Timeweb Cloud](https://timeweb.cloud).**

Servers, databases, S3, Kubernetes, balancers, domains, firewalls, apps and
more — managed from one native binary. No Python, no `pip`, no virtualenv.

![twc-rs dashboard demo](https://raw.githubusercontent.com/RAprogramm/twc-rs/main/docs/demo/dashboard.gif)

## Highlights

| | |
|---|---|
| Cold start | ~2 ms (the official Python CLI needs ~350 ms) |
| Footprint | one static 15 MB binary, no runtime dependencies |
| Dashboard | full TUI console: sidebar, card grids, interactive details, live metrics |
| Coverage | near-full parity with the official CLI |
| Completions | bash, zsh, fish, powershell, elvish, nushell — plus dynamic live-value completion |
| Languages | English and Russian, CLI and TUI |

The [CLI reference](cli/index.md) section of this book is generated directly
from the command definitions on every deploy, so it always matches the
released code.
