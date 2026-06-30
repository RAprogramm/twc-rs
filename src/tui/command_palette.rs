// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Command palette overlay — a VSCode-style fuzzy command launcher.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph}
};

use crate::tui::themes::Palette;

/// A single invocable command shown in the palette.
pub struct Command {
    /// Stable identifier used by the caller to dispatch the action.
    pub id:    String,
    /// Human-readable command title displayed on the left.
    pub title: String,
    /// Short right-aligned hint such as a keybinding or category.
    pub hint:  String
}

/// State for the command palette overlay.
///
/// Holds the full command set, the current fuzzy query, the indices of the
/// commands matching that query, and the highlighted selection.
pub struct CommandPalette {
    /// Current fuzzy query string typed by the user.
    pub query:    String,
    commands:     Vec<Command>,
    filtered:     Vec<usize>,
    /// Index into `filtered` of the highlighted command.
    pub selected: usize
}

impl CommandPalette {
    /// Creates a palette over `commands` with an empty query.
    ///
    /// All commands match initially and the first one is selected.
    #[must_use]
    pub fn new(commands: Vec<Command>) -> Self {
        let filtered = (0..commands.len()).collect();
        Self {
            query: String::new(),
            commands,
            filtered,
            selected: 0
        }
    }

    /// Replaces the command set, resetting the query and selection.
    #[allow(dead_code)]
    pub fn set_commands(&mut self, commands: Vec<Command>) {
        self.commands = commands;
        self.query.clear();
        self.selected = 0;
        self.refilter();
    }

    /// Appends `c` to the query, refilters, and clamps the selection.
    pub fn push_char(&mut self, c: char) {
        self.query.push(c);
        self.refilter();
        self.clamp_selected();
    }

    /// Removes the last query character, refilters, and clamps the selection.
    pub fn backspace(&mut self) {
        self.query.pop();
        self.refilter();
        self.clamp_selected();
    }

    /// Moves the selection down within the filtered list, wrapping.
    pub fn next(&mut self) {
        if self.filtered.is_empty() {
            self.selected = 0;
            return;
        }
        self.selected = (self.selected + 1) % self.filtered.len();
    }

    /// Moves the selection up within the filtered list, wrapping.
    pub fn previous(&mut self) {
        if self.filtered.is_empty() {
            self.selected = 0;
            return;
        }
        self.selected = if self.selected == 0 {
            self.filtered.len() - 1
        } else {
            self.selected - 1
        };
    }

    /// Returns the currently highlighted command, if any.
    #[must_use]
    pub fn selected_command(&self) -> Option<&Command> {
        self.filtered
            .get(self.selected)
            .and_then(|&idx| self.commands.get(idx))
    }

    /// Returns references to the matching commands in filtered order.
    #[must_use]
    pub fn filtered_commands(&self) -> Vec<&Command> {
        self.filtered
            .iter()
            .filter_map(|&idx| self.commands.get(idx))
            .collect()
    }

    /// Recomputes `filtered` from the current query.
    ///
    /// An empty query matches everything. Otherwise a command matches when the
    /// query is a case-insensitive subsequence of its title or its hint.
    fn refilter(&mut self) {
        if self.query.is_empty() {
            self.filtered = (0..self.commands.len()).collect();
            return;
        }
        let needle = self.query.to_lowercase();
        self.filtered = self
            .commands
            .iter()
            .enumerate()
            .filter(|(_, cmd)| {
                is_subsequence(&needle, &cmd.title.to_lowercase())
                    || is_subsequence(&needle, &cmd.hint.to_lowercase())
            })
            .map(|(idx, _)| idx)
            .collect();
    }

    /// Ensures `selected` points inside the filtered range.
    fn clamp_selected(&mut self) {
        if self.filtered.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len() - 1;
        }
    }
}

/// Returns whether `needle` is a subsequence of `haystack`.
///
/// Both inputs are expected to already be lowercased by the caller.
fn is_subsequence(needle: &str, haystack: &str) -> bool {
    let mut chars = haystack.chars();
    for nc in needle.chars() {
        loop {
            match chars.next() {
                Some(hc) if hc == nc => break,
                Some(_) => {}
                None => return false
            }
        }
    }
    true
}

/// Renders the command palette as a centered modal over `area`.
///
/// Draws a one-line query box showing `> {query}` with a cursor block, then the
/// filtered command list with titles on the left and right-aligned dim hints.
/// The selected row is bold and prefixed with a `▶` marker in the accent color.
pub fn render(frame: &mut Frame, area: Rect, palette: &Palette, cp: &CommandPalette) {
    let items = cp.filtered_commands();

    let width = 60u16.min(area.width.saturating_sub(4)).max(1);
    let max_height = area.height.saturating_sub(4).max(3);
    let item_count = u16::try_from(items.len()).unwrap_or(u16::MAX);
    let height = item_count.saturating_add(3).min(max_height).max(3);

    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let modal = Rect {
        x,
        y,
        width,
        height
    };

    frame.render_widget(Clear, modal);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Command ",
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD)
        ))
        .border_style(Style::default().fg(palette.accent))
        .style(Style::default().bg(palette.bg));

    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let query_area = Rect {
        x:      inner.x,
        y:      inner.y,
        width:  inner.width,
        height: 1
    };
    let query_line = Line::from(vec![
        Span::styled("> ", Style::default().fg(palette.accent)),
        Span::styled(cp.query.clone(), Style::default().fg(palette.fg)),
        Span::styled("█", Style::default().fg(palette.accent)),
    ]);
    frame.render_widget(Paragraph::new(query_line), query_area);

    let list_height = inner.height.saturating_sub(1);
    if list_height == 0 {
        return;
    }
    let list_area = Rect {
        x:      inner.x,
        y:      inner.y + 1,
        width:  inner.width,
        height: list_height
    };

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(idx, cmd)| {
            let is_selected = idx == cp.selected;
            build_item_line(cmd, is_selected, inner.width, palette)
        })
        .collect();

    let mut state = ListState::default();
    if items.is_empty() {
        state.select(None);
    } else {
        state.select(Some(cp.selected.min(items.len() - 1)));
    }

    let list = List::new(list_items).highlight_style(Style::default());
    frame.render_stateful_widget(list, list_area, &mut state);
}

/// Builds one styled list row for `cmd`.
fn build_item_line<'a>(
    cmd: &'a Command,
    is_selected: bool,
    width: u16,
    palette: &Palette
) -> ListItem<'a> {
    let marker = if is_selected { "▶ " } else { "  " };
    let title_color = if is_selected {
        palette.accent
    } else {
        palette.fg
    };
    let mut title_style = Style::default().fg(title_color);
    if is_selected {
        title_style = title_style.add_modifier(Modifier::BOLD);
    }

    let used = marker.chars().count() + cmd.title.chars().count() + cmd.hint.chars().count();
    let total = width as usize;
    let pad = total.saturating_sub(used).max(1);

    let line = Line::from(vec![
        Span::styled(marker, Style::default().fg(palette.accent)),
        Span::styled(cmd.title.as_str(), title_style),
        Span::raw(" ".repeat(pad)),
        Span::styled(cmd.hint.as_str(), Style::default().fg(palette.dim)),
    ]);

    let mut item = ListItem::new(line);
    if is_selected {
        item = item.style(Style::default().bg(selected_row_bg(palette)));
    }
    item
}

/// Background color used to highlight the selected palette row.
const fn selected_row_bg(palette: &Palette) -> Color {
    palette.border
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<Command> {
        vec![
            Command {
                id:    "refresh".to_string(),
                title: "Refresh Data".to_string(),
                hint:  "r".to_string()
            },
            Command {
                id:    "quit".to_string(),
                title: "Quit Application".to_string(),
                hint:  "Q".to_string()
            },
            Command {
                id:    "theme".to_string(),
                title: "Switch Theme".to_string(),
                hint:  "t".to_string()
            },
        ]
    }

    #[test]
    fn new_filters_all() {
        let cp = CommandPalette::new(sample());
        assert_eq!(cp.filtered_commands().len(), 3);
        assert_eq!(cp.selected, 0);
        assert!(cp.query.is_empty());
    }

    #[test]
    fn push_char_filters_subsequence() {
        let mut cp = CommandPalette::new(sample());
        cp.push_char('q');
        let titles: Vec<&str> = cp
            .filtered_commands()
            .iter()
            .map(|c| c.title.as_str())
            .collect();
        assert_eq!(titles, vec!["Quit Application"]);
    }

    #[test]
    fn push_char_subsequence_non_contiguous() {
        let mut cp = CommandPalette::new(sample());
        cp.push_char('r');
        cp.push_char('d');
        let titles: Vec<&str> = cp
            .filtered_commands()
            .iter()
            .map(|c| c.title.as_str())
            .collect();
        assert_eq!(titles, vec!["Refresh Data"]);
    }

    #[test]
    fn matches_on_hint() {
        let mut cp = CommandPalette::new(sample());
        cp.push_char('Q');
        assert_eq!(cp.filtered_commands().len(), 1);
        assert_eq!(cp.selected_command().unwrap().id, "quit");
    }

    #[test]
    fn backspace_restores() {
        let mut cp = CommandPalette::new(sample());
        cp.push_char('q');
        assert_eq!(cp.filtered_commands().len(), 1);
        cp.backspace();
        assert_eq!(cp.filtered_commands().len(), 3);
    }

    #[test]
    fn next_wraps() {
        let mut cp = CommandPalette::new(sample());
        cp.next();
        assert_eq!(cp.selected, 1);
        cp.next();
        assert_eq!(cp.selected, 2);
        cp.next();
        assert_eq!(cp.selected, 0);
    }

    #[test]
    fn previous_wraps() {
        let mut cp = CommandPalette::new(sample());
        cp.previous();
        assert_eq!(cp.selected, 2);
        cp.previous();
        assert_eq!(cp.selected, 1);
    }

    #[test]
    fn selected_command_correct() {
        let mut cp = CommandPalette::new(sample());
        cp.next();
        assert_eq!(cp.selected_command().unwrap().id, "quit");
    }

    #[test]
    fn empty_query_shows_all() {
        let mut cp = CommandPalette::new(sample());
        cp.push_char('z');
        assert_eq!(cp.filtered_commands().len(), 0);
        cp.backspace();
        assert_eq!(cp.filtered_commands().len(), 3);
    }

    #[test]
    fn selected_clamped_after_refilter() {
        let mut cp = CommandPalette::new(sample());
        cp.selected = 2;
        cp.push_char('q');
        assert_eq!(cp.selected, 0);
        assert_eq!(cp.selected_command().unwrap().id, "quit");
    }

    #[test]
    fn no_match_has_no_selected_command() {
        let mut cp = CommandPalette::new(sample());
        cp.push_char('z');
        assert!(cp.selected_command().is_none());
    }

    #[test]
    fn set_commands_resets() {
        let mut cp = CommandPalette::new(sample());
        cp.push_char('q');
        cp.set_commands(vec![Command {
            id:    "only".to_string(),
            title: "Only One".to_string(),
            hint:  "o".to_string()
        }]);
        assert!(cp.query.is_empty());
        assert_eq!(cp.selected, 0);
        assert_eq!(cp.filtered_commands().len(), 1);
    }

    #[test]
    fn case_insensitive() {
        let mut cp = CommandPalette::new(sample());
        cp.push_char('S');
        cp.push_char('W');
        assert_eq!(cp.selected_command().unwrap().id, "theme");
    }
}
