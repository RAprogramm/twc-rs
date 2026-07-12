# Dashboard (TUI)

The headline feature the official Python CLI does not have: a full cloud
console in the terminal — a full-height sidebar (Create hub, projects,
services, settings) with an adaptive card-grid content pane.

```sh
twc-rs dashboard
```

![twc-rs dashboard demo](https://raw.githubusercontent.com/RAprogramm/twc-rs/main/docs/demo/dashboard.gif)

## Keys

| Key | Action |
|---|---|
| `↑` `↓` / `k` `j` | move through the sidebar, cards or detail rows |
| `←` `→` / `h` `l` | move across the card grid |
| `Enter` | focus the content pane / open details / press a button |
| `Esc` | step back: details → content → sidebar |
| `y` / `c` | copy the highlighted detail value (OSC 52 clipboard) |
| `n` | create a new resource (where supported) |
| `/` | filter the current list |
| `Ctrl+K` | command palette — actions, theme, language, profile switch |
| `p` | switch account profile |
| `r` | refresh |
| `?` | help overlay |
| `Q` | quit |

## Features

- Instant warm start: the last snapshot renders immediately, then every
  endpoint streams its fresh data in independently — no blank screens.
- Interactive resource details at web-panel depth: action buttons (backup,
  redeploy, delete, …) on top, every value copyable, live CPU/RAM
  sparklines, a confirmation step for destructive actions.
- Create hub with descriptions of every product; create resources and
  switch the account profile without leaving the dashboard.
- Settings as cards: four true-color themes previewed with their palettes,
  EN/RU, layout toggles — all persisted to the config file.
- Drill into a project to see the resources it contains; live event log.
