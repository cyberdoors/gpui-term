//! gpui-term的终端视图组件。
//!
//! 此模块提供TerminalView结构体，它是处理输入事件并使用TerminalElement
//! 渲染终端的主要GPUI实体。
//!
//! # 架构设计
//!
//! TerminalView充当GPUI事件系统和Terminal实体之间的粘合剂。
//! 它：
//! - 接收键盘事件并通过try_keystroke转发给终端
//! - 处理鼠标事件（按下、释放、移动、滚动）用于选择和鼠标报告
//! - 提供绑定到键盘快捷键的复制/粘贴/清除/全选操作
//! - 订阅终端事件（唤醒、响铃、标题更改等）以更新UI
//!
//! # 示例
//!
//! ```ignore
//! let terminal = TerminalBuilder::new(...)?.subscribe(cx);
//! let terminal_entity = cx.new(|_| terminal);
//! let view = cx.new(|cx| TerminalView::new(terminal_entity, cx));
//! ```

// GPUI框架导入
use gpui::{
    App, ClipboardItem, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, Render,
    ScrollWheelEvent, Styled, Window, actions, div,
};

// 内部模块导入
use crate::{Event, Terminal, TerminalElement, TextStyle, ThemeManager};

// 定义终端相关的操作动作
actions!(
    terminal,
    [
        Copy,           // 复制
        Paste,          // 粘贴
        Clear,          // 清除
        SelectAll,      // 全选
        ScrollLineUp,   // 向上滚动一行
        ScrollLineDown, // 向下滚动一行
        ScrollPageUp,   // 向上滚动一页
        ScrollPageDown, // 向下滚动一页
        ChangeTheme     // 更改主题
    ]
);

/// 主终端视图组件，处理输入并协调渲染。
///
/// 此结构体包装Terminal实体并提供：
/// - 键盘输入路由的焦点管理
/// - 键盘和鼠标输入的事件处理器
/// - 剪贴板操作和滚动的动作处理器
/// - 订阅终端事件以更新UI
pub struct TerminalView {
    /// 底层终端实体
    terminal: Entity<Terminal>,
    /// 焦点句柄，用于管理输入焦点
    focus_handle: FocusHandle,
    /// 是否有响铃标志
    has_bell: bool,
    /// 文本样式配置
    text_style: crate::TextStyle,
}

impl TerminalView {
    /// Creates a new TerminalView wrapping the given Terminal entity.
    ///
    /// Sets up event subscriptions and focus handling for the terminal.
    pub fn new(terminal: Entity<Terminal>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::new_with_style(terminal, crate::TextStyle::default(), window, cx)
    }

    /// Creates a new TerminalView with a custom text style.
    pub fn new_with_style(
        terminal: Entity<Terminal>,
        text_style: crate::TextStyle,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus_handle = cx.focus_handle();

        cx.subscribe(&terminal, |this, _, event: &Event, cx| {
            this.handle_terminal_event(event, cx);
        })
        .detach();

        cx.on_focus_in(&focus_handle, window, |this: &mut Self, _window, cx| {
            this.terminal.update(cx, |terminal, _| {
                terminal.focus_in();
            });
        })
        .detach();

        cx.on_focus_out(
            &focus_handle,
            window,
            |this: &mut Self, _event, _window, cx| {
                this.terminal.update(cx, |terminal, _| {
                    terminal.focus_out();
                });
            },
        )
        .detach();

        Self {
            terminal,
            focus_handle,
            has_bell: false,
            text_style,
        }
    }

    /// Returns a reference to the underlying Terminal entity.
    pub fn terminal(&self) -> &Entity<Terminal> {
        &self.terminal
    }

    /// Returns whether the terminal bell has sounded since last cleared.
    pub fn has_bell(&self) -> bool {
        self.has_bell
    }

    /// Clears the bell indicator.
    pub fn clear_bell(&mut self, cx: &mut Context<Self>) {
        self.has_bell = false;
        cx.notify();
    }

    /// Updates the theme by applying a new theme from the ThemeManager.
    ///
    /// This updates the text_style with the new theme colors while preserving
    /// font settings.
    pub fn set_theme(&mut self, theme_name: &str, cx: &mut Context<Self>) {
        if let Some(theme_manager) = cx.try_global::<ThemeManager>()
            && let Some(theme) = theme_manager.get_theme(theme_name)
        {
            self.text_style.theme = theme.clone();
            self.text_style.foreground = theme.foreground;
            self.text_style.background = theme.background;
            cx.notify();
        }
    }

    /// Returns a mutable reference to the text style for external updates.
    pub fn text_style_mut(&mut self) -> &mut TextStyle {
        &mut self.text_style
    }

    fn handle_terminal_event(&mut self, event: &Event, cx: &mut Context<Self>) {
        match event {
            Event::Wakeup => cx.notify(),
            Event::Bell => {
                self.has_bell = true;
                cx.notify();
            }
            Event::TitleChanged => cx.notify(),
            Event::BlinkChanged(_) => cx.notify(),
            Event::SelectionsChanged => cx.notify(),
            Event::CloseTerminal => {
                cx.notify();
            }
        }
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.clear_bell(cx);

        let handled = self.terminal.update(cx, |terminal, _| {
            terminal.try_keystroke(&event.keystroke, false)
        });

        if handled {
            cx.stop_propagation();
        }
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus_handle, cx);
        window.prevent_default();

        if event.button == MouseButton::Right {
            let mouse_mode = self.terminal.read(cx).mouse_mode(event.modifiers.shift);
            if !mouse_mode {
                if let Some(item) = cx.read_from_clipboard()
                    && let Some(text) = item.text()
                {
                    self.terminal.update(cx, |terminal, _| {
                        terminal.paste(&text);
                    });
                }
                cx.notify();
                return;
            }
        }

        self.terminal.update(cx, |terminal, cx| {
            terminal.mouse_down(event, cx);
        });
        cx.notify();
    }

    fn on_mouse_up(&mut self, event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.terminal.update(cx, |terminal, cx| {
            terminal.mouse_up(event, cx);
        });
        cx.notify();
    }

    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.terminal.update(cx, |terminal, cx| {
            terminal.mouse_move(event, cx);
        });
    }

    fn on_mouse_drag(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let bounds = self.terminal.read(cx).last_content.terminal_bounds.bounds;
        self.terminal.update(cx, |terminal, cx| {
            terminal.mouse_drag(event, bounds, cx);
        });
    }

    fn on_scroll(
        &mut self,
        event: &ScrollWheelEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.terminal.update(cx, |terminal, _| {
            terminal.scroll_wheel(event, 1.0);
        });
        cx.notify();
    }

    fn copy(&mut self, _: &Copy, _window: &mut Window, cx: &mut Context<Self>) {
        self.terminal.update(cx, |terminal, _| {
            terminal.copy(Some(true));
        });

        if let Some(text) = self.terminal.read(cx).last_content.selection_text.clone() {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
        cx.notify();
    }

    fn paste(&mut self, _: &Paste, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard()
            && let Some(text) = item.text()
        {
            self.terminal.update(cx, |terminal, _| {
                terminal.paste(&text);
            });
        }
    }

    fn clear(&mut self, _: &Clear, _window: &mut Window, cx: &mut Context<Self>) {
        self.terminal.update(cx, |terminal, _| {
            terminal.clear();
        });
        cx.notify();
    }

    fn select_all(&mut self, _: &SelectAll, _window: &mut Window, cx: &mut Context<Self>) {
        self.terminal.update(cx, |terminal, _| {
            terminal.select_all();
        });
        cx.notify();
    }

    fn scroll_line_up(&mut self, _: &ScrollLineUp, _window: &mut Window, cx: &mut Context<Self>) {
        self.terminal.update(cx, |terminal, _| {
            terminal.scroll_line_up();
        });
        cx.notify();
    }

    fn scroll_line_down(
        &mut self,
        _: &ScrollLineDown,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.terminal.update(cx, |terminal, _| {
            terminal.scroll_line_down();
        });
        cx.notify();
    }

    fn scroll_page_up(&mut self, _: &ScrollPageUp, _window: &mut Window, cx: &mut Context<Self>) {
        self.terminal.update(cx, |terminal, _| {
            terminal.scroll_page_up();
        });
        cx.notify();
    }

    fn scroll_page_down(
        &mut self,
        _: &ScrollPageDown,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.terminal.update(cx, |terminal, _| {
            terminal.scroll_page_down();
        });
        cx.notify();
    }

    /// Synchronizes terminal state with the window for rendering.
    ///
    /// Should be called before rendering to ensure the terminal content is up to date.
    pub fn sync(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.terminal.update(cx, |terminal, cx| {
            terminal.sync(window, cx);
        });
    }
}

impl Focusable for TerminalView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TerminalView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let terminal = self.terminal.clone();
        let focus_handle = self.focus_handle.clone();
        let is_focused = self.focus_handle.is_focused(window);

        div()
            .id("terminal-view")
            .size_full()
            // Transparent background to allow blur effect from parent window
            .bg(gpui::transparent_black())
            .track_focus(&focus_handle)
            .key_context("Terminal")
            .on_key_down(cx.listener(Self::on_key_down))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_down(MouseButton::Right, cx.listener(Self::on_mouse_down))
            .on_mouse_down(MouseButton::Middle, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up(MouseButton::Right, cx.listener(Self::on_mouse_up))
            .on_mouse_up(MouseButton::Middle, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, window, cx| {
                if event.dragging() {
                    this.on_mouse_drag(event, window, cx);
                } else {
                    this.on_mouse_move(event, window, cx);
                }
            }))
            .on_scroll_wheel(cx.listener(Self::on_scroll))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::clear))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::scroll_line_up))
            .on_action(cx.listener(Self::scroll_line_down))
            .on_action(cx.listener(Self::scroll_page_up))
            .on_action(cx.listener(Self::scroll_page_down))
            .child(TerminalElement::new(
                terminal,
                focus_handle.clone(),
                is_focused,
                true,
                self.text_style.clone(),
            ))
    }
}
