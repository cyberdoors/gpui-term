//! Basic terminal example for gpui-term.
//!
//! Demonstrates a minimal working terminal application using the gpui-term library.
//! This example shows how to:
//! - Create a GPUI application with key bindings
//! - Spawn a terminal with PTY connection
//! - Handle async terminal initialization
//! - Display a loading state while the terminal initializes

use std::collections::HashMap;
use std::env;

use gpui::{
    actions, div, rgb, App, AppContext, Application, Context, Entity, FocusHandle, Focusable,
    InteractiveElement, IntoElement, KeyBinding, ParentElement, Render, Styled, Window,
    WindowOptions,
};
use gpui_term::{Clear, Copy, Paste, SelectAll, Terminal, TerminalBuilder, TerminalView};

actions!(basic_terminal, [Quit]);

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.bind_keys([
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("cmd-c", Copy, Some("Terminal")),
            KeyBinding::new("cmd-v", Paste, Some("Terminal")),
            KeyBinding::new("cmd-a", SelectAll, Some("Terminal")),
            KeyBinding::new("cmd-k", Clear, Some("Terminal")),
        ]);

        cx.on_action(|_: &Quit, cx| {
            cx.quit();
        });

        let window_options = WindowOptions {
            titlebar: Some(gpui::TitlebarOptions {
                title: Some("gpui-term".into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        cx.open_window(window_options, |window, cx| {
            let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());

            let mut env: HashMap<String, String> = env::vars().collect();
            env.insert("TERM".to_string(), "xterm-256color".to_string());
            env.insert("COLORTERM".to_string(), "truecolor".to_string());
            env.insert("TERM_PROGRAM".to_string(), "gpui-term".to_string());

            let window_id = window.window_handle().window_id().as_u64();

            let terminal_task = TerminalBuilder::new(
                env::current_dir().ok(),
                Some(shell),
                env,
                None,
                window_id,
                cx,
            );

            let view = cx.new(|cx| {
                let focus_handle = cx.focus_handle();
                focus_handle.focus(window, cx);

                MainView {
                    terminal: None,
                    terminal_view: None,
                    focus_handle,
                }
            });

            let view_clone = view.downgrade();
            let window_handle = window.window_handle();
            cx.spawn(async move |mut cx| {
                let builder = match terminal_task.await {
                    Ok(b) => b,
                    Err(e) => {
                        eprintln!("Failed to create terminal: {e}");
                        return;
                    }
                };

                let _ = cx.update_window(window_handle, |_, window, cx| {
                    let _ = view_clone.update(cx, |this, cx| {
                        let terminal = cx.new(|cx| builder.subscribe(cx));
                        this.set_terminal(terminal, window, cx);
                    });
                });
            })
            .detach();

            view
        })
        .unwrap();
    });
}

struct MainView {
    terminal: Option<Entity<Terminal>>,
    terminal_view: Option<Entity<TerminalView>>,
    focus_handle: FocusHandle,
}

impl MainView {
    fn set_terminal(&mut self, terminal: Entity<Terminal>, window: &mut Window, cx: &mut Context<Self>) {
        let terminal_view = cx.new(|cx| TerminalView::new(terminal.clone(), window, cx));

        // Focus the terminal view
        let focus_handle = terminal_view.read(cx).focus_handle(cx);
        focus_handle.focus(window, cx);

        self.terminal = Some(terminal);
        self.terminal_view = Some(terminal_view);
        cx.notify();
    }
}

impl Render for MainView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = if let Some(terminal_view) = &self.terminal_view {
            div().size_full().child(terminal_view.clone())
        } else {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(0xffffff))
                .child("Loading terminal...")
        };

        div()
            .id("main-view")
            .size_full()
            .bg(rgb(0x1e1e1e))
            .track_focus(&self.focus_handle)
            .child(content)
    }
}

impl Focusable for MainView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
