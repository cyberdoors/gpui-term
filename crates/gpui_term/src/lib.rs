mod mappings;
mod terminal;
mod config;
mod terminal_element;
mod terminal_view;

pub use config::{TerminalConfig, TerminalTheme, TerminalThemeConfig};
pub use terminal::{
    Event, IndexedCell, Terminal, TerminalBounds, TerminalBuilder, TerminalContent, ZedListener,
};
pub use terminal_element::{TerminalElement, TextStyle, convert_color};
pub use terminal_view::{
    Clear, Copy, Paste, ScrollLineDown, ScrollLineUp, ScrollPageDown, ScrollPageUp, SelectAll,
    TerminalView,
};
