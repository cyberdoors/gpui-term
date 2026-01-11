# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**gpui-term** is a GPUI-based terminal emulator built on top of Alacritty's terminal emulation library. The project provides a reusable terminal component for GPUI applications with full PTY support, mouse handling, and text selection.

## Build & Run Commands

```bash
# Build the library
cargo build

# Build with optimizations
cargo build --release

# Run the basic terminal example (Agent Term UI)
cargo run --example basic

# Run with release optimizations
cargo run --example basic --release

# Check for compilation errors
cargo check

# Run tests (if any exist)
cargo test
```

## Architecture

### Workspace Structure

This is a Cargo workspace with three main components:

1. **`crates/gpui_term`** - The core terminal library
2. **`examples/basic`** - Agent Term example application with a floating sidebar UI
3. **Root workspace** - Coordinates shared dependencies

### Core Architecture

The terminal system is built in layers, bridging GPUI's UI framework with Alacritty's terminal emulation:

#### 1. Terminal Entity (`terminal.rs`)
- **`Terminal`** - Main entity wrapping `Arc<FairMutex<Term<ZedListener>>>`
- Manages PTY communication via `Notifier` and `EventLoop`
- Handles input (keyboard, mouse, paste), scrolling, and selection
- Processes events through an internal queue (`VecDeque<InternalEvent>`)
- Events are batched in 4ms windows to reduce UI update overhead

**Event Flow:**
```
PTY → EventLoop → ZedListener (channel) → Terminal::process_event()
  → Terminal::events queue → Terminal::sync() → TerminalContent
```

#### 2. Terminal View (`terminal_view.rs`)
- **`TerminalView`** - GPUI entity that wraps a `Terminal` entity
- Implements `Render` and `Focusable` traits
- Handles GPUI input events (key down, mouse events, scroll wheel)
- Provides actions: `Copy`, `Paste`, `Clear`, `SelectAll`, `ScrollLineUp/Down`, `ScrollPageUp/Down`
- Subscribes to terminal events (`Wakeup`, `Bell`, `TitleChanged`, `BlinkChanged`, etc.)
- Delegates rendering to `TerminalElement`

#### 3. Terminal Element (`terminal_element.rs`)
- **`TerminalElement`** - Implements GPUI's `Element` trait for rendering
- **Batching optimization:** Adjacent cells with identical styles are combined into single text runs
- Handles color conversion from Alacritty (`Rgb`) to GPUI (`Hsla`)
- Manages cell flags: bold, italic, underline, strikethrough, dim
- Renders cursor with support for block, bar, and underline shapes
- Three-phase rendering: `request_layout` → `prepaint` → `paint`

#### 4. Mappings (`mappings/`)
- **`keys.rs`** - Translates GPUI keystrokes to terminal escape sequences
- **`mouse.rs`** - Handles mouse coordinate translation and reporting protocols

#### 5. Terminal Builder (`terminal.rs`)
- **`TerminalBuilder`** - Factory for creating terminals with PTY subscription
- Async terminal creation via `TerminalBuilder::new()` returns `Task<Result<TerminalBuilder>>`
- Call `.subscribe(cx)` to wire up the event loop and get a `Terminal`

### Key Data Structures

- **`TerminalBounds`** - Manages terminal dimensions (pixel bounds ↔ grid cells)
- **`TerminalContent`** - Snapshot of terminal state for rendering (cells, cursor, selection, mode)
- **`ZedListener`** - Bridges Alacritty's `EventListener` to GPUI via unbounded channel
- **`InternalEvent`** - Queued events for terminal state updates (resize, scroll, selection, copy)

### GPUI Integration

- Uses GPUI's `Entity` system for terminal lifecycle management
- Terminal creation is async because PTY setup requires I/O operations
- The event loop runs in a GPUI spawn task (`cx.spawn`) for continuous event processing
- Focus management via `FocusHandle` for keyboard input routing
- Mouse events are translated to grid coordinates for Alacritty's selection system

### Dependencies

- **GPUI** - Sourced from Zed's git repository (main UI framework)
- **Alacritty Terminal** - v0.25 (terminal emulation, VTE parsing, PTY management)
- **portable-pty** - v0.9 (cross-platform PTY interface, currently unused in Terminal but available)
- **smol** - v2.0 (async runtime used by event loop)

Note: `gpui_component` is disabled due to API incompatibility with current GPUI version (see workspace Cargo.toml).

## Important Implementation Notes

### Terminal Creation Pattern

Always follow this async pattern:

```rust
let terminal_task = TerminalBuilder::new(
    working_directory,
    shell,
    env_vars,
    max_scroll_history_lines,
    window_id,
    cx,
);

cx.spawn(async move |cx| {
    let builder = terminal_task.await?;
    let terminal = cx.new(|cx| builder.subscribe(cx));
    // Now you can create TerminalView from the terminal entity
})
```

### Input Handling

- **Special keys** (arrows, F-keys, ctrl combinations) → `Terminal::try_keystroke()` returns bool
- **Regular text input** → `Terminal::input_text()` for character input
- The view layer (`TerminalView::on_key_down`) calls `try_keystroke()` first; if false, GPUI's input handler chain continues

### Synchronization

Call `Terminal::sync()` before rendering to process queued internal events and update `last_content`. The `TerminalView` does this automatically in its render method.

### Event Batching

The event loop batches Alacritty events in 4ms windows to minimize UI updates (see `Terminal::subscribe` implementation around line 366-421). This is critical for performance when the terminal has high throughput.

### Mouse Mode

The terminal respects Alacritty's mouse mode (`TermMode::MOUSE_MODE`). When enabled and shift is not pressed, mouse events are sent to the PTY instead of being handled for selection.

### Selection Updates

Selection is managed through `InternalEvent::SetSelection` and `InternalEvent::UpdateSelection` to ensure atomicity during the sync phase.

## Rust Edition

This project uses Rust edition **2024** (see workspace Cargo.toml). Ensure your toolchain supports this edition.
