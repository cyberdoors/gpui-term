//! Simple Theme Integration Example
//!
//! This example demonstrates how to use gpui-term's built-in theme system
//! with a simple light/dark theme toggle.
//!
//! Controls:
//! - Click the theme buttons to switch between light and dark themes
//! - Cmd+Q/Ctrl+Q: Quit
//! - Cmd+C: Copy
//! - Cmd+V: Paste

use std::env;

use gpui::{
    App, AppContext, Application, Context, Entity, FocusHandle, Focusable, InteractiveElement,
    IntoElement, KeyBinding, MouseButton, MouseDownEvent, ParentElement, Render, Styled, Window,
    WindowOptions, actions, div, prelude::FluentBuilder, rgb,
};
use gpui_term::{Clear, Copy, Paste, SelectAll, TerminalBuilder, TerminalView, ThemeManager};

actions!(simple_theme, [Quit, ToggleToDark, ToggleToLight]);

/// Main application view containing the terminal and theme toggle buttons
struct ThemeDemo {
    terminal_view: Option<Entity<TerminalView>>,
    focus_handle: FocusHandle,
    current_theme: String,
}

impl ThemeDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            terminal_view: None,
            focus_handle: cx.focus_handle(),
            current_theme: "One Dark".to_string(),
        }
    }

    fn set_terminal(&mut self, terminal_view: Entity<TerminalView>) {
        self.terminal_view = Some(terminal_view);
    }

    fn switch_to_dark(&mut self, cx: &mut Context<Self>) {
        let theme_name = "One Dark";
        ThemeManager::global_mut(cx).set_theme(theme_name);

        if let Some(terminal_view) = &self.terminal_view {
            terminal_view.update(cx, |view, cx| {
                view.set_theme(theme_name, cx);
            });
        }

        self.current_theme = theme_name.to_string();
        cx.notify();
    }

    fn switch_to_light(&mut self, cx: &mut Context<Self>) {
        let theme_name = "One Light";
        ThemeManager::global_mut(cx).set_theme(theme_name);

        if let Some(terminal_view) = &self.terminal_view {
            terminal_view.update(cx, |view, cx| {
                view.set_theme(theme_name, cx);
            });
        }

        self.current_theme = theme_name.to_string();
        cx.notify();
    }

    fn render_theme_button(&self, label: String, is_active: bool) -> impl IntoElement {
        let bg_color = if is_active {
            rgb(0x0078d4) // Active blue
        } else {
            rgb(0x2a2a2a) // Inactive dark
        };

        let hover_color = if is_active {
            rgb(0x006cbe)
        } else {
            rgb(0x3a3a3a)
        };

        let text_color = if is_active {
            rgb(0xffffff)
        } else {
            rgb(0xa6a6a6)
        };

        div()
            .flex_1()
            .py_2()
            .bg(bg_color)
            .text_color(text_color)
            .rounded_md()
            .cursor_pointer()
            .hover(|style| style.bg(hover_color))
            .flex()
            .items_center()
            .justify_center()
            .child(label)
    }
}

impl Focusable for ThemeDemo {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ThemeDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_dark = self.current_theme.contains("Dark");
        let is_light = self.current_theme.contains("Light");

        div()
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(rgb(0x1a1a1a))
            .flex()
            .flex_col()
            .child(
                // Header with theme toggle
                div()
                    .w_full()
                    .p_4()
                    .bg(rgb(0x101010))
                    .border_b_1()
                    .border_color(rgb(0x3a3a3a))
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .text_color(rgb(0xd8d8d8))
                            .text_base()
                            .font_weight(gpui::FontWeight::BOLD)
                            .child("简单主题示例"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                div()
                                    .flex_1()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(
                                            |this, _event: &MouseDownEvent, _window, cx| {
                                                this.switch_to_dark(cx);
                                            },
                                        ),
                                    )
                                    .child(
                                        self.render_theme_button("🌙 暗色".to_string(), is_dark),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(
                                            |this, _event: &MouseDownEvent, _window, cx| {
                                                this.switch_to_light(cx);
                                            },
                                        ),
                                    )
                                    .child(
                                        self.render_theme_button("☀️ 亮色".to_string(), is_light),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .text_color(rgb(0x5a5a5a))
                            .text_xs()
                            .child(format!("当前主题: {} • Cmd+Q 退出", self.current_theme)),
                    ),
            )
            .child(
                // Terminal area
                div().flex_1().w_full().map(|el| {
                    if let Some(terminal_view) = &self.terminal_view {
                        el.child(terminal_view.clone())
                    } else {
                        el.flex()
                            .items_center()
                            .justify_center()
                            .child(div().text_color(rgb(0xa6a6a6)).child("正在加载终端..."))
                    }
                }),
            )
    }
}

fn main() {
    env_logger::init();

    Application::new().run(|cx| {
        // Initialize theme manager with default dark theme
        let theme_manager = ThemeManager::new(None);
        cx.set_global(theme_manager);

        // Register global actions
        cx.on_action(|_: &Quit, cx| cx.quit());

        // Register key bindings
        cx.bind_keys([
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("ctrl-q", Quit, None),
            KeyBinding::new("cmd-c", Copy, None),
            KeyBinding::new("cmd-v", Paste, None),
            KeyBinding::new("cmd-a", SelectAll, None),
            KeyBinding::new("cmd-k", Clear, None),
        ]);

        // Get shell configuration
        let shell = env::var("SHELL").unwrap_or_else(|_| {
            #[cfg(target_os = "windows")]
            {
                "powershell.exe".to_string()
            }
            #[cfg(not(target_os = "windows"))]
            {
                "/bin/bash".to_string()
            }
        });

        let working_directory = env::current_dir().ok();

        // Open main window
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds {
                    origin: gpui::Point {
                        x: gpui::px(100.0),
                        y: gpui::px(100.0),
                    },
                    size: gpui::Size {
                        width: gpui::px(1000.0),
                        height: gpui::px(700.0),
                    },
                })),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("简单主题示例".into()),
                    appears_transparent: false,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| {
                // Get window ID for terminal creation
                let window_id = window.window_handle().window_id().as_u64();

                // Create terminal asynchronously
                let terminal_task = TerminalBuilder::new(
                    working_directory.clone(),
                    Some(shell),
                    Default::default(),
                    Some(10000),
                    window_id,
                    cx,
                );

                // Create the main view
                let view = cx.new(|cx| ThemeDemo::new(cx));

                // Spawn terminal loading task
                let view_weak = view.downgrade();
                let window_handle = window.window_handle();
                cx.spawn(async move |cx| {
                    let builder = match terminal_task.await {
                        Ok(b) => b,
                        Err(e) => {
                            eprintln!("Failed to create terminal: {e}");
                            return;
                        }
                    };

                    let _ = cx.update_window(window_handle, |_, window, cx| {
                        let _ = view_weak.update(cx, |demo, cx| {
                            let terminal = cx.new(|cx| builder.subscribe(cx));
                            let terminal_view =
                                cx.new(|cx| TerminalView::new(terminal, window, cx));
                            demo.set_terminal(terminal_view);
                            cx.notify();
                        });
                    });
                })
                .detach();

                view
            },
        );
    });
}
