# TUI Dashboard Plan — June 2026

## 1. Professional Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│ twc-rs v0.1.0  │  Balance: 1,234.56 RUB  │  Token: 185d left      │
├─────────────────────────────────────────────────────────────────────┤
│ [▶ Servers] Databases  S3  Kubernetes  Projects  Balancers  ...    │
├─────────────────────────────────────────────────────────────────────┤
│ Projects: [📊 All] [🔥 production] [🧪 staging] [📦 backup] [+ New] │
├──────────────────────┬──────────────────────────────────────────────┤
│ Resources            │  Details                                     │
│ ───────────────────  │  ─────────────────────────────────────────  │
│ ▶ production-web-01  │  Name: production-web-01                    │
│   staging-db-01      │  ID: 12345                                   │
│   backup-s3-01       │  Status: Running                             │
│                      │  CPU: 4 cores @ 3.6 GHz                     │
│                      │  RAM: 8192 MB                               │
│                      │  Disk: 50 GB SSD                            │
│                      │  IP: 185.172.128.45                         │
│                      │  Location: ru-1 (Moscow)                    │
│                      │                                              │
│                      │  Actions: [Start] [Stop] [Reboot] [Delete]  │
├──────────────────────┴─────────────────────────────────────────────┤
│ k/j ↑↓  h/l ←→  Tab cycle  g first  $ last  r refresh  ? help  Q quit│
└─────────────────────────────────────────────────────────────────────┘
```

## 2. Vim Mode Navigation (Default)

| Клавиша | Действие |
|---------|----------|
| k / ↑ | Move selection up |
| j / ↓ | Move selection down |
| h / ← | Switch to previous resource tab |
| l / → | Switch to next resource tab |
| Tab | Cycle resource tabs |
| g | Go to first item |
| $ | Go to last item |
| r | Refresh all data |
| w | Toggle widget visibility |
| t | Cycle theme |
| Enter | Open resource detail view (popup) |
| Esc | Close popup, return to list |
| Q | Quit (big Q, not q) |
| ? | Toggle help overlay |

## 3. Custom Dashboards

User can create multiple dashboards with different widgets:

```toml
# ~/.config/twc-rs/dashboards.toml

[[dashboards]]
name = "production"
widgets = ["account", "resource_list", "details", "actions", "stats"]

[[dashboards]]
name = "monitoring"
widgets = ["account", "stats", "token_info"]

[[dashboards]]
name = "management"
widgets = ["account", "resource_list", "details", "actions", "project_manager"]
```

## 4. Sorting, Searching, Filtering

### Sorting
- Click column header to sort (A-Z, Z-A)
- Default sort: by name ascending
- Available sorts: name, status, created_at, size

### Searching
- `/` — open search bar
- Type to filter resources
- `Enter` — select first match
- `Esc` — close search bar

### Filtering
- Filter by status: running, stopped, error
- Filter by region: ru-1, ru-2, pl-1, kz-1, nl-1
- Filter by project: production, staging, backup
- Filter by date: today, week, month

## 5. Safe Operations

### No Accidental Deletes
- Delete requires confirmation: `y` to confirm, `n` to cancel
- Destructive actions show warning in red
- Soft delete first, permanent delete after 24h

### Confirmation Flow
```
┌─────────────────────────────────────────────────────────────────────┐
│ Delete Server?                                                     │
│ ─────────────────────────────────────────────────────────────────  │
│ Are you sure you want to delete "production-web-01"?              │
│ This action cannot be undone.                                     │
│                                                                     │
│ [y] Yes, delete  [n] No, cancel                                   │
└─────────────────────────────────────────────────────────────────────┘
```

## 6. Loading States

### Spinner in Loading Area
- When Tab is pressed, immediately switch to the new tab
- Show spinner in the content area while loading
- Spinner uses Unicode dots: ◜◠◝◞◡◟
- Loading indicator shows where data is being fetched

### Loading States
```
┌─────────────────────────────────────────────────────────────────────┐
│ Resources            │  Loading...                                  │
│ ───────────────────  │  ◜ Loading resources...                     │
│ ▶ production-web-01  │  ─────────────────────────────────────────  │
│   staging-db-01      │  (spinner animation)                        │
│   backup-s3-01       │                                              │
│                      │                                              │
│                      │                                              │
└──────────────────────┴─────────────────────────────────────────────┘
```

### Tab Switching
- Press Tab → immediately switch tab
- Show spinner in content area
- Fetch data in background
- Update content when data is ready
- Show error message if fetch fails

## 7. Widget System

### Widget Trait
```rust
pub trait Widget: Send {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn enabled(&self) -> bool;
    fn toggle(&mut self);
    fn render(&self, frame: &mut Frame, area: Rect, app: &App);
}
```

### Available Widgets
- `account` — Account info (ID, Balance, Status)
- `token_info` — Token expiration info
- `resource_list` — Resource list (scrollable)
- `details` — Resource details
- `actions` — Contextual actions
- `stats` — CPU/RAM/Network usage charts
- `project_manager` — Project filter tabs
- `help` — Help overlay
- `search` — Search bar
- `filter` — Filter panel

## 8. File Structure

```
src/
├── main.rs
├── cli.rs
├── error.rs
├── config.rs
├── output.rs
├── jwt.rs
├── commands/
│   ├── mod.rs
│   ├── servers.rs
│   ├── databases.rs
│   ├── s3.rs
│   ├── kubernetes.rs
│   ├── balancers.rs
│   ├── registry.rs
│   ├── domains.rs
│   ├── firewall.rs
│   ├── floating_ip.rs
│   ├── images.rs
│   ├── network_drives.rs
│   ├── vpc.rs
│   ├── dedicated_servers.rs
│   ├── mail.rs
│   ├── apps.rs
│   ├── ai_agents.rs
│   ├── knowledge_bases.rs
│   ├── account.rs
│   ├── projects.rs
│   └── ssh_keys.rs
└── tui/
    ├── mod.rs
    ├── app.rs
    ├── event.rs
    ├── ui.rs
    ├── themes/
    │   └── mod.rs
    └── widgets/
        ├── mod.rs
        ├── account.rs
        ├── token_info.rs
        ├── resource_list.rs
        ├── details.rs
        ├── actions.rs
        ├── stats.rs
        ├── project_manager.rs
        ├── resource_tabs.rs
        ├── help.rs
        ├── search.rs
        ├── filter.rs
        └── spinner.rs
```

## 9. Implementation Order

1. ResourceTabs widget — показать resource tabs в UI
2. Actions widget — контекстные действия
3. Обновить event.rs — vim keys + popup
4. Обновить ui.rs — новый layout
5. Обновить app.rs — detail_popup
6. Search widget — поиск ресурсов
7. Filter widget — фильтрация ресурсов
8. Custom dashboards — настройка виджетов
9. Safe operations — подтверждение действий
10. Loading states — spinner в области загрузки

## 10. Dependencies

### Existing
- ratatui — TUI framework
- crossterm — terminal control
- tokio-stream — event stream
- tabled — table output
- dialoguer — prompts
- colored — colors
- open — browser
- tiny_http — HTTP server
- keyring — keyring
- base64 — base64 encoding
- chrono — date/time

### New (if needed)
- None — all functionality can be implemented with existing dependencies

## 11. Testing

### Unit Tests
- Widget trait implementation
- WidgetRegistry
- ResourceTabs widget
- Actions widget
- Search widget
- Filter widget
- ProjectManager widget
- Account widget
- TokenInfo widget
- Stats widget
- Help widget
- Spinner widget
- App state management
- Event handling
- Theme switching
- Dashboard configuration

### Integration Tests
- Full TUI flow
- Tab switching
- Resource list navigation
- Detail view
- Search functionality
- Filter functionality
- Dashboard switching
- Theme switching
- Help overlay
- Loading states
- Error handling

## 12. Benchmarking

### Metrics
- Time to first render
- Time to load resources
- Time to switch tabs
- Time to search
- Time to filter
- Memory usage
- CPU usage

### Tools
- criterion — benchmarking framework
- tokio-console — async runtime monitoring
- flamegraph — profiling

## 13. Future Enhancements

### Phase 1 (June 2026)
- Core TUI functionality
- Vim mode navigation
- Resource tabs
- Project filters
- Search and filter
- Custom dashboards
- Safe operations

### Phase 2 (July 2026)
- Real-time updates
- WebSocket support
- Multi-user support
- Plugin system
- Theme editor
- Dashboard sharing

### Phase 3 (August 2026)
- Mobile support
- Remote management
- AI-powered insights
- Automated operations
- Integration with CI/CD
- Multi-cloud support
