# Usage

Every resource type is a subcommand; run `twc-rs <group> --help` to see its
actions and flags — the help screens are colorized.

![twc-rs apps list demo](https://raw.githubusercontent.com/RAprogramm/twc-rs/main/docs/demo/apps-list.gif)

```sh
twc-rs server list                        # list cloud servers
twc-rs server info --id 12345             # server details
twc-rs database list -f json              # JSON output
twc-rs ssh attach --server 12345 --key 42 # attach an SSH key to a server
twc-rs project resources --id 678         # drill into a project
twc-rs apps logs my-api --today           # app runtime logs by name or ID
twc-rs apps list-deploys my-api           # deploy history, newest first
twc-rs doctor                             # detect conflicting installs in PATH
twc-rs dashboard                          # interactive TUI (k9s-style)
```

Per-app commands accept either the app **name** or its numeric **ID**.

## Global flags

| Flag | Env | Meaning |
|---|---|---|
| `-f, --format <table\|json\|yaml\|quiet>` | `TWC_OUTPUT` | output format (default `table`) |
| `-t, --token <TOKEN>` | `TWC_TOKEN` | API token override |
| `--profile <NAME>` | `TWC_PROFILE` | named profile for multi-account setups |

The full, always-current list of commands is in the
[CLI reference](cli/index.md).
