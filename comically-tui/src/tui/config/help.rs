use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Text},
    widgets::{Clear, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

use crate::tui::{
    utils::{popup_block, themed_block},
    Theme,
};

pub struct Keybinding {
    pub key: &'static str,
    pub action: &'static str,
    pub docs: &'static str,
}

pub struct HelpState {
    pub keybindings: Vec<Keybinding>,
    pub list_state: ListState,
}

impl HelpState {
    pub fn new() -> Self {
        let keybindings = vec![
            Keybinding {
                key: "↑/↓/j/k",
                action: "navigate files",
                docs: "move up and down through the file list in the left pane. arrow keys or vim-style navigation both work",
            },
            Keybinding {
                key: "space",
                action: "toggle file selection",
                docs: "select or deselect the current focused file in the left pane. selected files show [✓] and will be processed when you press enter",
            },
            Keybinding {
                key: "a",
                action: "toggle all files",
                docs: "select or deselect all files at once. if all files are currently selected, this deselects all. otherwise selects all",
            },
            Keybinding {
                key: "enter",
                action: "start processing",
                docs: "begin converting all selected files with current settings. files are saved to the output directory with the chosen format",
            },
            Keybinding {
                key: "m",
                action: "reading direction",
                docs: "toggle between reading modes:\n\n• left to right: standard western comics\n• right to left: manga style\n\naffects page order in output files",
            },
            Keybinding {
                key: "s",
                action: "spread splitter",
                docs: "cycle through double-page handling:\n\n• none: keep spreads as-is\n• split: cut spreads into separate pages\n• rotate: rotate spreads 90° for vertical viewing\n• rotate & split: show twice - rotated then split",
            },
            Keybinding {
                key: "c",
                action: "auto crop",
                docs: "toggle automatic margin removal. when enabled, detects and removes blank space around page content for better screen fit",
            },
            Keybinding {
                key: "f",
                action: "output format",
                docs: "cycle through output formats:\n\n• azw3/mobi: amazon kindle format\n• epub: standard e-book format\n• cbz: comic book archive (zip)\n\nnote: mobi forces jpeg image format",
            },
            Keybinding {
                key: "i",
                action: "image format",
                docs: "cycle compression formats:\n\n• jpeg: lossy, smaller files\n• png: lossless, larger files\n• webp: modern, good compression\n\ndisabled for mobi output",
            },
            Keybinding {
                key: "u",
                action: "quality/compression",
                docs: "select quality setting for adjustment\n\n• jpeg/webp: quality 0-100\n• png: fast/default/best compression\n\nuse ←/→ arrows to adjust value",
            },
            Keybinding {
                key: "b",
                action: "brightness",
                docs: "select brightness for adjustment\n\nrange: -100 to +100\n• negative values: darker image\n• positive values: brighter image\n\nuse ←/→ arrows to adjust",
            },
            Keybinding {
                key: "g",
                action: "gamma",
                docs: "select gamma correction for adjustment\n\nrange: 0.1 to 3.0\n• < 1.0: lower contrast, lifted shadows\n• > 1.0: higher contrast, deeper blacks\n• = 1.0: no adjustment\n\nuse ←/→ arrows to adjust",
            },
            Keybinding {
                key: "←/→",
                action: "adjust values",
                docs: "decrease/increase selected setting (quality, brightness, or gamma)\n\nhold shift for fine adjustments:\n• quality: ±1 instead of ±5\n• brightness: ±1 instead of ±5\n• gamma: ±0.05 instead of ±0.1",
            },
            Keybinding {
                key: "d",
                action: "device presets",
                docs: "open device selector to choose from common e-reader presets. automatically sets optimal dimensions for your target device",
            },
            Keybinding {
                key: "o",
                action: "margin color",
                docs: "cycle margin fill when image doesn't fill screen:\n\n• none: preserve original aspect ratio\n• black: fill empty space with black\n• white: fill empty space with white",
            },
            Keybinding {
                key: "p",
                action: "load preview",
                docs: "load preview of selected file with current settings applied. updates when settings change. useful for testing before batch processing",
            },
            Keybinding {
                key: "h",
                action: "toggle help",
                docs: "show or hide this help menu. press h or esc to close",
            },
            Keybinding {
                key: "t",
                action: "toggle theme",
                docs: "switch between light and dark color themes",
            },
            Keybinding {
                key: "q",
                action: "quit",
                docs: "exit the application. any unsaved settings will be lost",
            },
            Keybinding {
                key: "esc",
                action: "cancel/close",
                docs: "context sensitive:\n• close modal dialogs\n• deselect adjustment fields\n• cancel current operation",
            },
        ];

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            keybindings,
            list_state,
        }
    }

    pub fn select_next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.keybindings.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn select_previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.keybindings.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}

pub fn render_help_popup(area: Rect, buf: &mut Buffer, theme: &Theme, help_state: &mut HelpState) {
    let popup_width = (area.width * 4 / 5).min(100);
    let popup_height = (area.height * 9 / 10).min(40);

    let popup_x = area.left() + (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = area.top() + (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    Clear.render(popup_area, buf);

    let block = popup_block("help", theme).title(Line::from("[esc/h to close]").right_aligned());
    let inner = block.inner(popup_area);
    block.render(popup_area, buf);

    // Split the area into left (keybindings list) and right (documentation)
    let [list_area, docs_area] =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).areas(inner);

    // Render keybindings list
    let items: Vec<ListItem> = help_state
        .keybindings
        .iter()
        .map(|kb| {
            let content = format!("{:<12} {}", kb.key, kb.action);
            ListItem::new(content).style(theme.content)
        })
        .collect();

    let list = List::new(items)
        .block(themed_block(Some("keybindings"), theme))
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::REVERSED),
        )
        .highlight_symbol("> ");

    StatefulWidget::render(list, list_area, buf, &mut help_state.list_state);

    // Render documentation for selected keybinding
    if let Some(selected) = help_state.list_state.selected() {
        if let Some(keybinding) = help_state.keybindings.get(selected) {
            let docs_block = themed_block(Some(keybinding.action), theme);
            let docs_inner = docs_block.inner(docs_area);
            docs_block.render(docs_area, buf);

            let docs_text = Text::from(keybinding.docs);
            let docs_paragraph = Paragraph::new(docs_text)
                .style(Style::default().fg(theme.content))
                .wrap(ratatui::widgets::Wrap { trim: true });

            docs_paragraph.render(docs_inner, buf);
        }
    }
}
