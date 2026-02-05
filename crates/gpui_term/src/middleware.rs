// 标准库导入
use std::borrow::Cow;

// 内部模块导入
use crate::{Event, TerminalContent};

/// 描述输入字节的来源
/// 用于标识不同类型的数据输入来源
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputOrigin {
    /// 键盘按键输入
    Keystroke,
    /// 文本输入
    Text,
    /// 粘贴操作
    Paste,
    /// 鼠标操作
    Mouse,
    /// 滚轮滚动
    Scroll,
    /// 焦点变化
    Focus,
    /// 剪贴板操作
    Clipboard,
    /// 系统级输入
    System,
    /// 程序化输入
    Programmatic,
}

/// 终端中间件trait
/// 用于观察或转换终端的输入/输出数据
pub trait TerminalMiddleware: Send + Sync {
    /// 在输入数据到达PTY之前检查或转换输入字节
    /// 返回`None`表示丢弃该输入
    fn on_input(
        &self,
        input: Cow<'static, [u8]>,
        _origin: InputOrigin,
    ) -> Option<Cow<'static, [u8]>> {
        Some(input)
    }

    /// 观察发送到视图层的终端事件
    fn on_event(&self, _event: &Event) {}

    /// 观察渲染的输出快照
    fn on_output(&self, _content: &TerminalContent) {}
}
