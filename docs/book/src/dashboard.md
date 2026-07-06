# Dashboard (TUI)

The headline feature the official Python CLI does not have: a live,
k9s-style TUI.

```sh
twc-rs dashboard
```

![twc-rs dashboard demo](https://raw.githubusercontent.com/RAprogramm/twc-rs/main/docs/demo/dashboard.gif)

## Keys

| Key | Action |
|---|---|
| `h` / `l` | switch resource tabs |
| `j` / `k` | move selection |
| `Enter` | open the action menu / drill into a resource |
| `n` | create a new resource (where supported) |
| `/` | filter the current list |
| `Ctrl+K` | command palette — actions, theme, language, profile switch |
| `?` | help overlay |
| `Q` | quit |

## Features

- Context action menu per resource (reboot / shutdown / clone / delete) with
  a confirmation step for destructive actions.
- Create resources and switch the account profile without leaving the
  dashboard.
- Live per-resource metrics (CPU / RAM / network sparklines) fetched off the
  UI thread, so input never blocks on the network.
- Drill into a project to see its resources; live event log.
- Customizable layout, hide-empty-tabs, four true-color themes and EN/RU —
  all persisted to the config file.
