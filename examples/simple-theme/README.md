# Simple Theme Example

这个示例演示了如何使用 gpui-term 内置主题系统实现明暗主题切换。

## 功能展示

- 如何初始化 `ThemeManager` 并设置预设主题
- 使用简洁的 UI 在明暗两种主题之间切换
- 主题实时切换，无需重启终端

## 运行示例

```bash
cargo run --example simple-theme --release
```

## 界面说明

示例提供了一个简洁的顶部工具栏，包含：
- **🌙 暗色** - 切换到 One Dark 主题（深色背景）
- **☀️ 亮色** - 切换到 One Light 主题（浅色背景）
- 当前主题显示和快捷键提示

点击按钮即可在明暗主题之间切换，终端内容会立即更新颜色。

## 主题系统概览

gpui-term 包含一个 `ThemeManager`，提供了多个预设主题：
- **One Dark** (默认，暗色)
- **One Light** (亮色)
- Solarized Dark
- Solarized Light
- Dracula
- Nord
- Gruvbox Dark
- Gruvbox Light
- GitHub Light

### 基本用法

```rust
use gpui_term::ThemeManager;

// 使用默认主题初始化
let theme_manager = ThemeManager::new(None);
cx.set_global(theme_manager);

// 终端视图会自动使用当前主题
let terminal_view = cx.new(|cx| TerminalView::new(terminal, window, cx));
```

### 切换主题

运行时切换主题：

```rust
// 获取可用主题列表
let themes = ThemeManager::global(cx).available_themes();

// 切换到不同主题
ThemeManager::global_mut(cx).set_theme("One Light");

// 应用到终端视图
terminal_view.update(cx, |view, cx| {
    view.set_theme("One Light", cx);
});
```

## 代码结构

示例使用了一个简洁的 `ThemeDemo` 结构：
- 包含可选的 `TerminalView`（异步加载）
- 提供 `switch_to_dark()` 和 `switch_to_light()` 方法
- 使用鼠标事件监听器响应按钮点击
- 根据加载状态显示不同UI

## 下一步

要查看更复杂的示例：
- `examples/component-integration` - 与 gpui-component UI 框架集成
- `examples/basic` - 完整的终端应用，带主题菜单和侧边栏

## 快捷键

- **Cmd+Q / Ctrl+Q**: 退出
- **Cmd+C**: 复制
- **Cmd+V**: 粘贴
- **Cmd+A**: 全选
- **Cmd+K**: 清屏

