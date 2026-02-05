/// 配置模块 - 处理终端配置和主题设置
mod config;
/// 映射模块 - 键盘和鼠标事件映射
mod mappings;
/// 中间件模块 - 提供输入/输出处理的中间件系统
mod middleware;
/// 核心终端模块 - 终端仿真器的主要实现
mod terminal;
/// 终端元素模块 - GPUI UI元素和渲染逻辑
mod terminal_element;
/// 终端视图模块 - 终端的视图层实现
mod terminal_view;
/// 主题管理模块 - 主题加载和管理系统
mod theme_manager;

/// 导出配置相关类型
pub use config::{TerminalConfig, TerminalTheme, TerminalThemeConfig, hsla_from_rgb};
/// 导出中间件相关类型
pub use middleware::{InputOrigin, TerminalMiddleware};
/// 导出核心终端类型
pub use terminal::{
    Event, IndexedCell, Terminal, TerminalBounds, TerminalBuilder, TerminalContent, ZedListener,
};
/// 导出终端元素相关类型
pub use terminal_element::{TerminalElement, TextStyle, convert_color};
/// 导出终端视图相关类型和命令
pub use terminal_view::{
    ChangeTheme, Clear, Copy, Paste, ScrollLineDown, ScrollLineUp, ScrollPageDown, ScrollPageUp,
    SelectAll, TerminalView,
};
/// 导出主题管理相关类型
pub use theme_manager::{ThemeDefinition, ThemeManager};
