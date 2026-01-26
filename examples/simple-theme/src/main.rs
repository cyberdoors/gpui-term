//! Simple Theme Integration Example with SSH Support
//!
//! This example demonstrates how to use gpui-term with SSH connections.
//!
//! Controls:
//! - Click the SSH Connect button to connect to a remote server
//! - Cmd+Q/Ctrl+Q: Quit
//! - Cmd+C: Copy
//! - Cmd+V: Paste

use gpui::{
    App, AppContext, Application, Context, Entity, FocusHandle, Focusable, InteractiveElement,
    IntoElement, KeyBinding, MouseButton, ParentElement, Render, Styled, Window, WindowOptions,
    actions, div, prelude::FluentBuilder, rgb,
};
use gpui_term::{
    Clear, Copy, Paste, SelectAll, Terminal, TerminalBuilder, TerminalView, ThemeManager,
};

actions!(simple_theme, [Quit]);

/// SSH connection state
#[derive(Clone, Debug, PartialEq)]
enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

/// Main application view containing the terminal and SSH connection UI
struct ThemeDemo {
    terminal_view: Option<Entity<TerminalView>>,
    terminal: Option<Entity<Terminal>>,
    focus_handle: FocusHandle,
    // SSH connection fields
    ssh_host: String,
    ssh_user: String,
    ssh_password: String,
    ssh_port: String,
    connection_state: ConnectionState,
    password_sent: bool,
}

impl ThemeDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            terminal_view: None,
            terminal: None,
            focus_handle: cx.focus_handle(),
            ssh_host: "47.79.92.52".to_string(),
            ssh_user: "root".to_string(),
            ssh_password: "Liang123.aliyun".to_string(),
            ssh_port: "22".to_string(),
            connection_state: ConnectionState::Disconnected,
            password_sent: false,
        }
    }

    fn set_terminal(&mut self, terminal: Entity<Terminal>, terminal_view: Entity<TerminalView>) {
        self.terminal = Some(terminal);
        self.terminal_view = Some(terminal_view);
        self.connection_state = ConnectionState::Connecting;
    }

    fn connect_ssh(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.ssh_host.is_empty() || self.ssh_user.is_empty() {
            return;
        }

        self.connection_state = ConnectionState::Connecting;
        self.password_sent = false;

        // Build SSH command
        let port = self.ssh_port.parse::<u16>().unwrap_or(22);
        let ssh_command = if port == 22 {
            format!(
                "ssh -o StrictHostKeyChecking=no {}@{}",
                self.ssh_user, self.ssh_host
            )
        } else {
            format!(
                "ssh -o StrictHostKeyChecking=no -p {} {}@{}",
                port, self.ssh_user, self.ssh_host
            )
        };

        let window_id = window.window_handle().window_id().as_u64();

        // Create terminal with SSH command as shell
        let terminal_task = TerminalBuilder::new(
            None,
            Some(ssh_command),
            Default::default(),
            Some(10000),
            window_id,
            cx,
        );

        let view = cx.entity().downgrade();
        let window_handle = window.window_handle();

        cx.spawn(async move |_, cx| {
            let builder = match terminal_task.await {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Failed to create SSH terminal: {e}");
                    return;
                }
            };

            let _ = cx.update_window(window_handle, |_, window, cx| {
                let _ = view.update(cx, |demo, cx| {
                    let terminal = cx.new(|cx| builder.subscribe(cx));
                    let terminal_view =
                        cx.new(|cx| TerminalView::new(terminal.clone(), window, cx));
                    demo.set_terminal(terminal, terminal_view);
                    cx.notify();
                });
            });
        })
        .detach();

        cx.notify();
    }

    fn disconnect(&mut self, cx: &mut Context<Self>) {
        self.terminal_view = None;
        self.terminal = None;
        self.connection_state = ConnectionState::Disconnected;
        self.password_sent = false;
        cx.notify();
    }

    fn check_and_send_password(&mut self, cx: &mut Context<Self>) {
        if self.password_sent || self.ssh_password.is_empty() {
            return;
        }

        if let Some(terminal) = &self.terminal {
            terminal.update(cx, |term, _cx| {
                let content = term.last_content();
                let text: String = content.cells.iter().map(|cell| cell.c).collect();
                let lower = text.to_lowercase();

                // Check for password prompt
                if lower.contains("password:") || lower.contains("password for") {
                    // Send password with newline
                    let password_with_newline = format!("{}\r", self.ssh_password);
                    term.input_text(&password_with_newline);
                    self.password_sent = true;
                    self.connection_state = ConnectionState::Connected;
                }
            });
        }
    }

    fn render_input_field(
        &self,
        label: &str,
        value: &str,
        placeholder: &str,
        is_password: bool,
    ) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .text_color(rgb(0xa6a6a6))
                    .text_xs()
                    .child(label.to_string()),
            )
            .child(
                div()
                    .px_3()
                    .py_2()
                    .bg(rgb(0x2a2a2a))
                    .border_1()
                    .border_color(rgb(0x3a3a3a))
                    .rounded_md()
                    .text_color(rgb(0xd8d8d8))
                    .text_sm()
                    .child(if value.is_empty() {
                        div()
                            .text_color(rgb(0x5a5a5a))
                            .child(placeholder.to_string())
                    } else if is_password {
                        div().child("•".repeat(value.len()))
                    } else {
                        div().child(value.to_string())
                    }),
            )
    }

    fn render_button(&self, label: &str, enabled: bool) -> impl IntoElement {
        let bg = if enabled {
            rgb(0x0078d4)
        } else {
            rgb(0x3a3a3a)
        };
        let hover_bg = if enabled {
            rgb(0x006cbe)
        } else {
            rgb(0x3a3a3a)
        };
        let text_color = if enabled {
            rgb(0xffffff)
        } else {
            rgb(0x6a6a6a)
        };

        div()
            .px_4()
            .py_2()
            .bg(bg)
            .text_color(text_color)
            .rounded_md()
            .cursor_pointer()
            .hover(|s| s.bg(hover_bg))
            .flex()
            .items_center()
            .justify_center()
            .child(label.to_string())
    }
}

impl Focusable for ThemeDemo {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ThemeDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Check for password prompt if connecting
        if self.connection_state == ConnectionState::Connecting && !self.password_sent {
            self.check_and_send_password(cx);
        }

        let is_connected = self.terminal_view.is_some();
        let can_connect = !self.ssh_host.is_empty() && !self.ssh_user.is_empty();

        div()
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(rgb(0x1a1a1a))
            .flex()
            .flex_col()
            .child(
                // Header with SSH connection controls
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
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_color(rgb(0xd8d8d8))
                                    .text_base()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .child("SSH 终端"),
                            )
                            .child(
                                div()
                                    .text_color(match self.connection_state {
                                        ConnectionState::Disconnected => rgb(0x6a6a6a),
                                        ConnectionState::Connecting => rgb(0xf0ad4e),
                                        ConnectionState::Connected => rgb(0x5cb85c),
                                    })
                                    .text_xs()
                                    .child(match self.connection_state {
                                        ConnectionState::Disconnected => "未连接",
                                        ConnectionState::Connecting => "连接中...",
                                        ConnectionState::Connected => "已连接",
                                    }),
                            ),
                    )
                    .when(!is_connected, |el| {
                        el.child(
                            // SSH input fields
                            div()
                                .flex()
                                .gap_3()
                                .child(div().flex_1().child(self.render_input_field(
                                    "主机",
                                    &self.ssh_host,
                                    "47.79.92.52",
                                    false,
                                )))
                                .child(div().w(gpui::px(80.0)).child(self.render_input_field(
                                    "端口",
                                    &self.ssh_port,
                                    "22",
                                    false,
                                )))
                                .child(div().flex_1().child(self.render_input_field(
                                    "用户名",
                                    &self.ssh_user,
                                    "root",
                                    false,
                                )))
                                .child(div().flex_1().child(self.render_input_field(
                                    "密码",
                                    &self.ssh_password,
                                    "Liang123.aliyun",
                                    true,
                                ))),
                        )
                        .child(
                            div()
                                .flex()
                                .gap_2()
                                .child(
                                    div()
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _, window, cx| {
                                                this.connect_ssh(window, cx);
                                            }),
                                        )
                                        .when(can_connect, |el| {
                                            el.child(self.render_button("连接", true))
                                        })
                                        .when(!can_connect, |el| {
                                            el.child(self.render_button("连接", false))
                                        }),
                                )
                                .child(
                                    div()
                                        .text_color(rgb(0x5a5a5a))
                                        .text_xs()
                                        .flex()
                                        .items_center()
                                        .child("点击输入框后使用键盘输入"),
                                ),
                        )
                    })
                    .when(is_connected, |el| {
                        el.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_3()
                                .child(div().text_color(rgb(0xa6a6a6)).text_sm().child(format!(
                                    "{}@{}:{}",
                                    self.ssh_user, self.ssh_host, self.ssh_port
                                )))
                                .child(
                                    div()
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _, _, cx| {
                                                this.disconnect(cx);
                                            }),
                                        )
                                        .child(
                                            div()
                                                .px_3()
                                                .py_1()
                                                .bg(rgb(0xd9534f))
                                                .text_color(rgb(0xffffff))
                                                .text_sm()
                                                .rounded_md()
                                                .cursor_pointer()
                                                .hover(|s| s.bg(rgb(0xc9302c)))
                                                .child("断开"),
                                        ),
                                ),
                        )
                    }),
            )
            .child(
                // Terminal area
                div().flex_1().w_full().map(|el| {
                    if let Some(terminal_view) = &self.terminal_view {
                        el.child(terminal_view.clone())
                    } else {
                        el.flex().items_center().justify_center().child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap_2()
                                .child(div().text_color(rgb(0x5a5a5a)).text_xl().child("🔒"))
                                .child(
                                    div()
                                        .text_color(rgb(0xa6a6a6))
                                        .child("输入 SSH 连接信息后点击连接"),
                                ),
                        )
                    }
                }),
            )
            // Handle keyboard input for the form fields
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, _, cx| {
                if this.terminal_view.is_some() {
                    return; // Let terminal handle input
                }

                let key = &event.keystroke.key;

                // Handle backspace
                if key == "backspace" {
                    // Remove last char from the focused field (simple: just ssh_host for now)
                    if !this.ssh_password.is_empty() {
                        this.ssh_password.pop();
                    } else if !this.ssh_user.is_empty() {
                        this.ssh_user.pop();
                    } else if !this.ssh_port.is_empty() {
                        this.ssh_port.pop();
                    } else if !this.ssh_host.is_empty() {
                        this.ssh_host.pop();
                    }
                    cx.notify();
                    return;
                }

                // Handle tab to switch fields
                if key == "tab" {
                    // Cycle through fields (simplified)
                    cx.notify();
                    return;
                }

                // Handle enter to connect
                if key == "enter" {
                    // Will be handled by connect button
                    return;
                }

                // Handle regular character input
                if let Some(key_char) = &event.keystroke.key_char {
                    let ch = key_char.as_str();
                    // Simple field focus: fill in order
                    if this.ssh_host.len() < 50
                        && this.ssh_user.is_empty()
                        && this.ssh_password.is_empty()
                    {
                        this.ssh_host.push_str(ch);
                    } else if this.ssh_user.len() < 30 && this.ssh_password.is_empty() {
                        this.ssh_user.push_str(ch);
                    } else if this.ssh_password.len() < 50 {
                        this.ssh_password.push_str(ch);
                    }
                    cx.notify();
                }
            }))
    }
}

fn main() {
    env_logger::init();

    Application::new().run(|cx: &mut App| {
        // Initialize theme manager
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
                    title: Some("SSH 终端".into()),
                    appears_transparent: false,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_window, cx| cx.new(|cx| ThemeDemo::new(cx)),
        );
    });
}
