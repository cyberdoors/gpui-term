//! Agent Term - A terminal application with project management sidebar
//!
//! This example demonstrates a layered UI with:
//! - Left sidebar with project tree and terminal sessions
//! - Main terminal area with colorful status bar
//! - Blur and transparency effects

use std::collections::HashMap;
use std::env;

use gpui::{
    actions, div, prelude::*, px, rgb, rgba, App, AppContext, Application, Context, Entity,
    FocusHandle, Focusable, InteractiveElement, IntoElement, KeyBinding, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, Window, WindowBackgroundAppearance,
    WindowOptions,
};
use gpui_term::{Clear, Copy, Paste, SelectAll, Terminal, TerminalBuilder, TerminalView};

actions!(agent_term, [Quit, ToggleSidebar]);

// Colors
const SIDEBAR_BG: u32 = 0x1a1d2e;
const MAIN_BG: u32 = 0x1e2235;
const HEADER_BG: u32 = 0x151825;
const TEXT_PRIMARY: u32 = 0xc8ccd4;
const TEXT_SECONDARY: u32 = 0x6b7280;
const TEXT_MUTED: u32 = 0x4b5563;
const ACCENT_CYAN: u32 = 0x5eead4;
const ACCENT_ORANGE: u32 = 0xfbbf24;
const ACCENT_GREEN: u32 = 0x4ade80;
const ACCENT_PURPLE: u32 = 0xc084fc;
const BORDER_COLOR: u32 = 0x2d3348;

const BACKGROUND_OPACITY: f32 = 0.92;
const ENABLE_BLUR: bool = true;

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.bind_keys([
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("cmd-b", ToggleSidebar, None),
            KeyBinding::new("cmd-c", Copy, Some("Terminal")),
            KeyBinding::new("cmd-v", Paste, Some("Terminal")),
            KeyBinding::new("cmd-a", SelectAll, Some("Terminal")),
            KeyBinding::new("cmd-k", Clear, Some("Terminal")),
        ]);

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

            let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
            let mut env_vars: HashMap<String, String> = env::vars().collect();
            env_vars.insert("TERM".to_string(), "xterm-256color".to_string());
            env_vars.insert("COLORTERM".to_string(), "truecolor".to_string());

            let window_id = window.window_handle().window_id().as_u64();
            let terminal_task = TerminalBuilder::new(
                env::current_dir().ok(),
                Some(shell),
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
    projects: Vec<Project>,
}

impl AgentTermApp {
    fn set_terminal(
        &mut self,
        terminal: Entity<Terminal>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let terminal_view = cx.new(|cx| TerminalView::new(terminal.clone(), window, cx));
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

    fn render_sidebar(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        let alpha = (BACKGROUND_OPACITY * 255.0) as u32;

        div()
            .w(px(260.0))
            .h_full()
            .flex_shrink_0()
            .bg(rgba(SIDEBAR_BG << 8 | alpha))
            .border_r_1()
            .border_color(rgb(BORDER_COLOR))
            .flex()
            .flex_col()
            .child(self.render_sidebar_header())
            .child(self.render_add_project())
            .child(self.render_project_tree())
    }

    fn render_sidebar_header(&self) -> impl IntoElement {
        div()
            .h(px(52.0))
            .px(px(16.0))
            .flex()
            .items_center()
            .justify_between()
            .border_b_1()
            .border_color(rgb(BORDER_COLOR))
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
        div()
            .px(px(16.0))
            .py(px(12.0))
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(TEXT_SECONDARY))
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
                            .text_color(rgb(TEXT_SECONDARY))
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
                            .text_color(rgb(TEXT_MUTED))
                            .child("No terminals"),
                    );
                } else {
                    for session in &project.sessions {
                        sessions_container =
                            sessions_container.child(self.render_session(session));
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
                            .border_color(rgb(TEXT_SECONDARY)),
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
                            .text_color(rgb(TEXT_SECONDARY))
                            .px(px(4.0))
                            .cursor_pointer()
                            .hover(|s| s.text_color(rgb(TEXT_PRIMARY)))
                            .child("···"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(TEXT_SECONDARY))
                            .px(px(4.0))
                            .cursor_pointer()
                            .hover(|s| s.text_color(rgb(TEXT_PRIMARY)))
                            .child("×"),
                    ),
            )
    }

    fn render_main_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let alpha = (BACKGROUND_OPACITY * 255.0) as u32;

        div()
            .flex_1()
            .h_full()
            .bg(rgba(MAIN_BG << 8 | alpha))
            .flex()
            .flex_col()
            .child(self.render_status_bar())
            .child(self.render_terminal_area(cx))
    }

    fn render_status_bar(&self) -> impl IntoElement {
        div()
            .h(px(36.0))
            .px(px(16.0))
            .flex()
            .items_center()
            .gap(px(4.0))
            .border_b_1()
            .border_color(rgb(BORDER_COLOR))
            // User segment
            .child(status_segment(
                " adityasharma",
                ACCENT_CYAN,
                0x134e4a,
                true,
                false,
            ))
            // Path segment
            .child(status_segment(
                ".../terminal-app",
                ACCENT_CYAN,
                0x134e4a,
                false,
                false,
            ))
            // Git branch segment
            .child(status_segment(" 0.1.4 !", ACCENT_ORANGE, 0x78350f, false, false))
            // Version segment
            .child(status_segment(" v24.4.1", ACCENT_GREEN, 0x14532d, false, false))
            // Time segment
            .child(status_segment(" 22:09", ACCENT_PURPLE, 0x581c87, false, true))
            // Chevron
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(TEXT_SECONDARY))
                    .pl(px(4.0))
                    .child("❯"),
            )
    }

    fn render_terminal_area(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_1()
            .w_full()
            .when_some(self.terminal_view.as_ref(), |el, tv| {
                el.child(tv.clone())
            })
            .when(self.terminal_view.is_none(), |el| {
                el.flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_color(rgb(TEXT_MUTED))
                            .child("Loading terminal..."),
                    )
            })
    }
}

fn status_segment(
    text: impl Into<SharedString>,
    fg_color: u32,
    bg_color: u32,
    is_first: bool,
    is_last: bool,
) -> impl IntoElement {
    let text: SharedString = text.into();
    div()
        .flex()
        .items_center()
        .h(px(22.0))
        .px(px(10.0))
        .bg(rgb(bg_color))
        .text_color(rgb(fg_color))
        .text_xs()
        .font_weight(gpui::FontWeight::MEDIUM)
        .when(is_first, |s| s.rounded_l(px(4.0)))
        .when(is_last, |s| s.rounded_r(px(4.0)))
        .when(!is_first && !is_last, |s| s.rounded(px(0.0)))
        .child(text)
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
        .text_color(rgb(TEXT_SECONDARY))
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
            .flex()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::toggle_sidebar))
            .when(self.sidebar_visible, |el| {
                el.child(self.render_sidebar(cx))
            })
            .child(self.render_main_content(cx))
    }
}

impl Focusable for AgentTermApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
