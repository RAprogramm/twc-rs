# Installation

Every channel ships with the TUI dashboard enabled by default.

| Channel | Command |
|---|---|
| crates.io | `cargo install twc-rs` |
| Installer (Linux/macOS) | `curl -fsSL https://raw.githubusercontent.com/RAprogramm/twc-rs/main/install.sh \| sh` |
| Arch (AUR) | `paru -S twc-rs-bin` |
| Debian/Ubuntu | `sudo apt install ./twc-rs_<ver>_amd64.deb` |
| Releases | download from [GitHub Releases](https://github.com/RAprogramm/twc-rs/releases), verify the `.sha256` |

For a lean, headless CLI without the TUI:

```sh
cargo install twc-rs --no-default-features --features auth
```

## Supported platforms

| OS | Architectures |
|---|---|
| Linux (glibc) | `x86_64`, `aarch64` |
| macOS | `x86_64`, `aarch64` (Apple Silicon) |
| Windows | `x86_64` |

Run `twc-rs doctor` after installing to detect conflicting copies in `PATH`.
