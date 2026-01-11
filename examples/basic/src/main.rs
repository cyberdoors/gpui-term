//! Agent Term - A GPUI terminal with a floating sidebar overlay.
//!
//! This example focuses on reproducing Agent Term's current visual layout:
//! - Floating, rounded sidebar with inset + shadow
//! - Main terminal content padded to avoid the sidebar
//! - Transparent/blurred window background

use std::collections::HashMap;
use std::env;

use gpui::{
    App, AppContext, Application, BoxShadow, Context, Entity, FocusHandle, Focusable,
    InteractiveElement, IntoElement, KeyBinding, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, ParentElement, Pixels, Render, SharedString, StatefulInteractiveElement, Styled,
    Window, WindowBackgroundAppearance, WindowOptions, actions, div, hsla, point, prelude::*, px,
    rgb, rgba,
};
use gpui_term::{
    Clear, Copy, Paste, SelectAll, Terminal, TerminalBuilder, TerminalConfig, TerminalView,
    TextStyle,
};

actions!(agent_term, [Quit, ToggleSidebar]);

// Layout (mirrors the Tauri UI tokens / App.tsx layout math)
const SIDEBAR_INSET: f32 = 8.0;
const SIDEBAR_GAP: f32 = 16.0;
const SIDEBAR_MIN_WIDTH: f32 = 200.0;
const SIDEBAR_MAX_WIDTH: f32 = 420.0;
const SIDEBAR_HEADER_LEFT_PADDING: f32 = 68.0;

// Colors (approximate the current Agent Term Tauri tokens)
const TEXT_PRIMARY: u32 = 0xd8d8d8;
const TEXT_SUBTLE: u32 = 0xa6a6a6;
const TEXT_FAINT: u32 = 0x5a5a5a;

const SURFACE_ROOT: u32 = 0x000000;
const SURFACE_SIDEBAR: u32 = 0x202020;
const BORDER_SOFT: u32 = 0x3a3a3a;

const SURFACE_ROOT_ALPHA: f32 = 0.05;
const SURFACE_SIDEBAR_ALPHA: f32 = 0.32;
const BORDER_SOFT_ALPHA: f32 = 0.50;

const ENABLE_BLUR: bool = true;

fn platform_keybindings() -> Vec<KeyBinding> {
    let mut bindings = vec![
        KeyBinding::new("ctrl-shift-c", Copy, Some("Terminal")),
        KeyBinding::new("ctrl-shift-v", Paste, Some("Terminal")),
    ];

    #[cfg(target_os = "macos")]
    {
        bindings.extend([
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("cmd-b", ToggleSidebar, None),
            KeyBinding::new("cmd-c", Copy, Some("Terminal")),
            KeyBinding::new("cmd-v", Paste, Some("Terminal")),
            KeyBinding::new("cmd-a", SelectAll, Some("Terminal")),
            KeyBinding::new("cmd-k", Clear, Some("Terminal")),
        ]);
    }

    #[cfg(not(target_os = "macos"))]
    {
        bindings.extend([
            KeyBinding::new("ctrl-shift-q", Quit, None),
            KeyBinding::new("ctrl-shift-b", ToggleSidebar, None),
            KeyBinding::new("ctrl-shift-a", SelectAll, Some("Terminal")),
            KeyBinding::new("ctrl-shift-k", Clear, Some("Terminal")),
        ]);
    }

    bindings
}

#[cfg(windows)]
fn find_on_path(executable: &str) -> Option<String> {
    let path = env::var_os("PATH")?;
    for dir in env::split_paths(&path) {
        let candidate = dir.join(executable);
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().into_owned());
        }
    }
    None
}

#[cfg(windows)]
fn find_pwsh() -> Option<String> {
    if let Some(path) = find_on_path("pwsh.exe") {
        return Some(path);
    }

    let roots = ["ProgramW6432", "ProgramFiles", "ProgramFiles(x86)"];
    for key in roots {
        if let Ok(root) = env::var(key) {
            for suffix in ["PowerShell\\7\\pwsh.exe", "PowerShell\\7-preview\\pwsh.exe"] {
                let path = std::path::PathBuf::from(&root).join(suffix);
                if path.is_file() {
                    return Some(path.to_string_lossy().into_owned());
                }
            }
        }
    }

    if let Ok(root) = env::var("LOCALAPPDATA") {
        for suffix in [
            "Microsoft\\PowerShell\\7\\pwsh.exe",
            "Microsoft\\PowerShell\\7-preview\\pwsh.exe",
        ] {
            let path = std::path::PathBuf::from(&root).join(suffix);
            if path.is_file() {
                return Some(path.to_string_lossy().into_owned());
            }
        }
    }

    None
}

fn platform_shell() -> Option<String> {
    #[cfg(windows)]
    {
        if let Ok(shell) = env::var("SHELL") {
            return Some(shell);
        }

        if let Some(pwsh) = find_pwsh() {
            return Some(pwsh);
        }

        if let Ok(root) = env::var("SystemRoot") {
            let mut path = std::path::PathBuf::from(root);
            path.push("System32");
            path.push("WindowsPowerShell");
            path.push("v1.0");
            path.push("powershell.exe");
            return Some(path.to_string_lossy().into_owned());
        }

        return Some("powershell".to_string());
    }

    #[cfg(not(windows))]
    {
        if let Ok(shell) = env::var("SHELL") {
            return Some(shell);
        }
        if std::path::Path::new("/bin/zsh").exists() {
            Some("/bin/zsh".to_string())
        } else {
            Some("/bin/bash".to_string())
        }
    }
}

fn rgba_u32(rgb: u32, alpha: f32) -> u32 {
    let a = (alpha.clamp(0.0, 1.0) * 255.0).round() as u32;
    (rgb << 8) | a
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.bind_keys(platform_keybindings());

        cx.on_action(|_: &Quit, cx| cx.quit());

        let background_appearance = if ENABLE_BLUR {
            WindowBackgroundAppearance::Blurred
        } else {
            WindowBackgroundAppearance::Opaque
        };

        let window_options = WindowOptions {
            titlebar: Some(gpui::TitlebarOptions {
                title: Some("Agent Term".into()),
                appears_transparent: true,
                traffic_light_position: Some(gpui::point(px(16.0), px(16.0))),
                ..Default::default()
            }),
            window_background: background_appearance,
            ..Default::default()
        };

        cx.open_window(window_options, |window, cx| {
            window.set_background_appearance(background_appearance);

            let terminal_config =
                TerminalConfig::load_or_create().unwrap_or_else(|_| TerminalConfig::default());
            let text_style = TextStyle::from_config(&terminal_config);

            let shell = platform_shell();
            let mut env_vars: HashMap<String, String> = env::vars().collect();
            env_vars.insert("TERM".to_string(), "xterm-256color".to_string());
            env_vars.insert("COLORTERM".to_string(), "truecolor".to_string());

            let window_id = window.window_handle().window_id().as_u64();
            let terminal_task = TerminalBuilder::new(
                env::current_dir().ok(),
                shell,
                env_vars,
                None,
                window_id,
                cx,
            );

            let view = cx.new(|cx| {
                let focus_handle = cx.focus_handle();
                focus_handle.focus(window, cx);

                AgentTermApp {
                    terminal: None,
                    terminal_view: None,
                    focus_handle,
                    sidebar_visible: true,
                    sidebar_width: 250.0,
                    resizing_sidebar: false,
                    resize_start_x: Pixels::ZERO,
                    resize_start_width: 250.0,
                    text_style,
                    projects: vec![
                        Project {
                            name: "Agent Term".into(),
                            expanded: true,
                            sessions: vec![TerminalSession {
                                name: "adityasharma@Ma...".into(),
                                active: true,
                            }],
                        },
                        Project {
                            name: "Frontier OS".into(),
                            expanded: true,
                            sessions: vec![],
                        },
                    ],
                }
            });

            let view_clone = view.downgrade();
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
                    let _ = view_clone.update(cx, |app, cx| {
                        let terminal = cx.new(|cx| builder.subscribe(cx));
                        app.set_terminal(terminal, window, cx);
                    });
                });
            })
            .detach();

            view
        })
        .unwrap();
    });
}

#[derive(Clone)]
struct Project {
    name: SharedString,
    expanded: bool,
    sessions: Vec<TerminalSession>,
}

#[derive(Clone)]
struct TerminalSession {
    name: SharedString,
    active: bool,
}

struct AgentTermApp {
    terminal: Option<Entity<Terminal>>,
    terminal_view: Option<Entity<TerminalView>>,
    focus_handle: FocusHandle,
    sidebar_visible: bool,
    sidebar_width: f32,
    resizing_sidebar: bool,
    resize_start_x: Pixels,
    resize_start_width: f32,
    text_style: TextStyle,
    projects: Vec<Project>,
}

impl AgentTermApp {
    fn set_terminal(
        &mut self,
        terminal: Entity<Terminal>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let text_style = self.text_style.clone();
        let terminal_view = cx.new(|cx| {
            TerminalView::new_with_style(terminal.clone(), text_style, window, cx)
        });
        let focus_handle = terminal_view.read(cx).focus_handle(cx);
        focus_handle.focus(window, cx);

        self.terminal = Some(terminal);
        self.terminal_view = Some(terminal_view);
        cx.notify();
    }

    fn toggle_sidebar(&mut self, _: &ToggleSidebar, _window: &mut Window, cx: &mut Context<Self>) {
        self.sidebar_visible = !self.sidebar_visible;
        cx.notify();
    }

    fn start_sidebar_resize(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.resizing_sidebar = true;
        self.resize_start_x = event.position.x;
        self.resize_start_width = self.sidebar_width;
        cx.notify();
    }

    fn stop_sidebar_resize(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.resizing_sidebar {
            self.resizing_sidebar = false;
            cx.notify();
        }
    }

    fn update_sidebar_resize(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.resizing_sidebar || !event.dragging() {
            return;
        }

        let delta = event.position.x - self.resize_start_x;
        let next_width =
            (self.resize_start_width + delta / px(1.0)).clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH);
        if (next_width - self.sidebar_width).abs() > 0.1 {
            self.sidebar_width = next_width;
            cx.notify();
        }
    }

    fn sidebar_shadow() -> Vec<BoxShadow> {
        vec![
            BoxShadow {
                color: hsla(0., 0., 0., 0.25),
                offset: point(px(0.0), px(18.0)),
                blur_radius: px(45.0),
                spread_radius: px(0.0),
            },
            BoxShadow {
                color: hsla(0., 0., 0., 0.15),
                offset: point(px(0.0), px(6.0)),
                blur_radius: px(18.0),
                spread_radius: px(0.0),
            },
        ]
    }

    fn render_sidebar_shell(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("sidebar-shell")
            .absolute()
            .left(px(SIDEBAR_INSET))
            .top(px(SIDEBAR_INSET))
            .bottom(px(SIDEBAR_INSET))
            .w(px(self.sidebar_width))
            .relative()
            .child(
                div()
                    .id("sidebar-wrapper")
                    .size_full()
                    .rounded(px(16.0))
                    .overflow_hidden()
                    .bg(rgba(rgba_u32(SURFACE_SIDEBAR, SURFACE_SIDEBAR_ALPHA)))
                    .border_1()
                    .border_color(rgba(rgba_u32(BORDER_SOFT, BORDER_SOFT_ALPHA)))
                    .shadow(Self::sidebar_shadow())
                    .child(self.render_sidebar_content(cx)),
            )
            .child(
                div()
                    .id("sidebar-resizer")
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left(px(self.sidebar_width - 3.0))
                    .w(px(6.0))
                    .rounded(px(999.0))
                    .bg(gpui::transparent_black())
                    .cursor_col_resize()
                    .hover(|s| s.bg(rgba(rgba_u32(TEXT_PRIMARY, 0.20))))
                    .on_mouse_down(MouseButton::Left, cx.listener(Self::start_sidebar_resize))
                    .on_mouse_up(MouseButton::Left, cx.listener(Self::stop_sidebar_resize))
                    .on_mouse_move(cx.listener(Self::update_sidebar_resize)),
            )
    }

    fn render_sidebar_content(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("sidebar-content")
            .size_full()
            .flex()
            .flex_col()
            .child(self.render_sidebar_header())
            .child(self.render_add_project())
            .child(self.render_project_tree())
    }

    fn render_sidebar_header(&self) -> impl IntoElement {
        div()
            .h(px(44.0))
            .pl(px(SIDEBAR_HEADER_LEFT_PADDING))
            .pr(px(16.0))
            .flex()
            .items_center()
            .justify_between()
            .border_b_1()
            .border_color(rgba(rgba_u32(BORDER_SOFT, BORDER_SOFT_ALPHA)))
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(TEXT_PRIMARY))
                    .child("AGENT TERM"),
            )
            .child(
                div()
                    .flex()
                    .gap(px(12.0))
                    .child(icon_button("icons/search.svg"))
                    .child(icon_button("icons/tag.svg"))
                    .child(icon_button("icons/settings.svg")),
            )
    }

    fn render_add_project(&self) -> impl IntoElement {
        div().px(px(16.0)).py(px(12.0)).child(
            div()
                .text_sm()
                .text_color(rgb(TEXT_SUBTLE))
                .cursor_pointer()
                .hover(|s| s.text_color(rgb(TEXT_PRIMARY)))
                .child("+ Add Project"),
        )
    }

    fn render_project_tree(&self) -> impl IntoElement {
        let mut tree = div()
            .id("project-tree")
            .flex_1()
            .overflow_y_scroll()
            .px(px(8.0));

        for project in &self.projects {
            tree = tree.child(self.render_project(project));
        }

        tree
    }

    fn render_project(&self, project: &Project) -> impl IntoElement {
        div()
            .py(px(4.0))
            .child(
                // Project header
                div()
                    .px(px(8.0))
                    .py(px(6.0))
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .cursor_pointer()
                    .rounded(px(4.0))
                    .hover(|s| s.bg(rgba(0xffffff10)))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(TEXT_SUBTLE))
                            .child(if project.expanded { "▼" } else { "▶" }),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(rgb(TEXT_PRIMARY))
                            .child(project.name.clone()),
                    ),
            )
            .when(project.expanded, |el| {
                let mut sessions_container = div().pl(px(16.0));

                if project.sessions.is_empty() {
                    sessions_container = sessions_container.child(
                        div()
                            .px(px(8.0))
                            .py(px(4.0))
                            .text_sm()
                            .text_color(rgb(TEXT_FAINT))
                            .child("No terminals"),
                    );
                } else {
                    for session in &project.sessions {
                        sessions_container = sessions_container.child(self.render_session(session));
                    }
                }

                el.child(sessions_container)
            })
    }

    fn render_session(&self, session: &TerminalSession) -> impl IntoElement {
        div()
            .px(px(8.0))
            .py(px(4.0))
            .flex()
            .items_center()
            .justify_between()
            .rounded(px(4.0))
            .cursor_pointer()
            .when(session.active, |s| s.bg(rgba(0xffffff10)))
            .hover(|s| s.bg(rgba(0xffffff15)))
            .group("session")
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .w(px(12.0))
                            .h(px(12.0))
                            .rounded(px(2.0))
                            .border_1()
                            .border_color(rgb(TEXT_SUBTLE)),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(TEXT_PRIMARY))
                            .max_w(px(140.0))
                            .overflow_x_hidden()
                            .child(session.name.clone()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .opacity(0.0)
                    .group_hover("session", |s| s.opacity(1.0))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(TEXT_SUBTLE))
                            .px(px(4.0))
                            .cursor_pointer()
                            .hover(|s| s.text_color(rgb(TEXT_PRIMARY)))
                            .child("···"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(TEXT_SUBTLE))
                            .px(px(4.0))
                            .cursor_pointer()
                            .hover(|s| s.text_color(rgb(TEXT_PRIMARY)))
                            .child("×"),
                    ),
            )
    }

    fn render_terminal_container(&self) -> impl IntoElement {
        let content_left = if self.sidebar_visible {
            self.sidebar_width + SIDEBAR_INSET + SIDEBAR_GAP
        } else {
            0.0
        };

        div()
            .id("terminal-container")
            .absolute()
            .top_0()
            .right_0()
            .bottom_0()
            .left(px(content_left))
            .flex()
            .flex_col()
            .when_some(self.terminal_view.as_ref(), |el, tv| {
                el.child(
                    div()
                        .flex_1()
                        .overflow_hidden()
                        .py(px(16.0))
                        .px(px(8.0))
                        .child(tv.clone()),
                )
            })
            .when(self.terminal_view.is_none(), |el| {
                el.flex()
                    .items_center()
                    .justify_center()
                    .child(div().text_color(rgb(TEXT_FAINT)).child("Loading terminal…"))
            })
    }
}

fn icon_button(_icon_path: &str) -> impl IntoElement {
    // Placeholder - using text for now since we don't have icons
    div()
        .w(px(20.0))
        .h(px(20.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(4.0))
        .cursor_pointer()
        .text_color(rgb(TEXT_SUBTLE))
        .hover(|s| s.text_color(rgb(TEXT_PRIMARY)).bg(rgba(0xffffff10)))
        .child("•")
}

impl Render for AgentTermApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("agent-term-app")
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .relative()
            .bg(rgba(rgba_u32(SURFACE_ROOT, SURFACE_ROOT_ALPHA)))
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::toggle_sidebar))
            .on_mouse_move(cx.listener(Self::update_sidebar_resize))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::stop_sidebar_resize))
            .child(self.render_terminal_container())
            .when(self.sidebar_visible, |el| {
                el.child(self.render_sidebar_shell(cx))
            })
    }
}

impl Focusable for AgentTermApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
