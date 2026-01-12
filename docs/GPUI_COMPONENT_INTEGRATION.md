# gpui-term 与 gpui-component 主题集成指南

本文档说明如何在使用 gpui-component 时集成 gpui-term，并共用 gpui-component 的主题系统。

## 架构概述

```
┌──────────────────────────────────────────────┐
│     gpui-component Theme (Global)            │
│  - ThemeColor (80+ colors)                   │
│  - light_theme / dark_theme                  │
│  - base colors (red, green, blue, ...)       │
└───────────────────┬──────────────────────────┘
                    │
         ┌──────────▼──────────┐
         │   ThemeAdapter      │
         │  (theme_adapter.rs) │
         └──────────┬──────────┘
                    │
         ┌──────────▼──────────┐
         │   TerminalTheme     │
         │  - ANSI colors      │
         │  - Bright/Dim       │
         └──────────┬──────────┘
                    │
         ┌──────────▼──────────┐
         │   TerminalView      │
         │  - Renders terminal │
         └─────────────────────┘
```

## 启用 gpui-component 集成

### 1. 在 Cargo.toml 中启用 feature

```toml
[dependencies]
gpui_term = { path = "path/to/gpui-term/crates/gpui_term", features = ["gpui-component"] }
gpui-component = { path = "path/to/gpui-component/crates/ui" }
```

### 2. 初始化 gpui-component 主题系统

```rust
use gpui::App;
use gpui_component;

fn main() {
    Application::new().run(|cx: &mut App| {
        // 初始化 gpui-component 主题系统
        gpui_component::init(cx);

        // 你的应用代码...
    });
}
```

## 使用方式

### 方式 1: 自动主题同步（推荐）

使用 `ComponentThemeExt` trait 自动同步 gpui-component 的主题变化：

```rust
use gpui_term::{TerminalView, ComponentThemeExt};

// 创建终端视图
let terminal_view = cx.new(|cx| {
    let mut view = TerminalView::new_with_style(terminal, text_style, window, cx);

    // 应用当前 gpui-component 主题
    view.apply_component_theme(cx);

    // 设置主题观察器，自动响应主题变化
    view.observe_component_theme(cx);

    view
});
```

**优点**：
- 自动响应主题切换
- 无需手动管理主题同步
- 与 gpui-component 的主题切换完全集成

### 方式 2: 手动转换主题

直接使用 `ThemeAdapter` 转换主题：

```rust
use gpui_term::ThemeAdapter;

// 从全局 gpui-component 主题创建终端主题
let terminal_theme = ThemeAdapter::from_global_theme(cx);

// 或者从 ThemeColor 转换
let component_theme = gpui_component::ActiveTheme::theme(cx);
let terminal_theme = ThemeAdapter::to_terminal_theme(&component_theme.colors);

// 应用到 TextStyle
let text_style = TextStyle {
    theme: terminal_theme,
    foreground: terminal_theme.foreground,
    background: terminal_theme.background,
    // ... 其他配置
};
```

### 方式 3: 响应主题切换事件

如果需要在主题切换时执行额外的逻辑：

```rust
use gpui_term::ComponentThemeExt;

// 在终端视图中设置主题观察器
terminal_view.update(cx, |view, cx| {
    cx.observe_global::<gpui_component::Theme>(|this, cx| {
        // 应用新主题
        this.apply_component_theme(cx);

        // 执行额外的自定义逻辑
        log::info!("Terminal theme updated");

        // 可以在这里保存主题设置等
    })
    .detach();
});
```

## 颜色映射详情

### 基础颜色映射

| gpui-component | gpui-term | 说明 |
|----------------|-----------|------|
| `foreground` | `foreground` | 终端前景色/文本色 |
| `background` | `background` | 终端背景色 |
| `caret` | `cursor` | 光标颜色 |
| `selection` | `selection` | 选中区域背景色 |

### ANSI 颜色映射

| Index | gpui-component | gpui-term | 颜色 |
|-------|----------------|-----------|------|
| 0 | `background.lighten(0.1)` | `ansi[0]` | Black |
| 1 | `red` | `ansi[1]` | Red |
| 2 | `green` | `ansi[2]` | Green |
| 3 | `yellow` | `ansi[3]` | Yellow |
| 4 | `blue` | `ansi[4]` | Blue |
| 5 | `magenta` | `ansi[5]` | Magenta |
| 6 | `cyan` | `ansi[6]` | Cyan |
| 7 | `foreground` | `ansi[7]` | White |

### Bright/Dim 颜色

- **Bright 颜色**: 基础 ANSI 颜色亮化 20% (`color.lighten(0.2)`)
- **Dim 颜色**: 基础 ANSI 颜色暗化 20% (`color.darken(0.2)`)

gpui-component 提供了 `*_light` 变体（如 `red_light`, `green_light`），优先使用这些变体作为 bright 颜色。

## 完整示例

```rust
use gpui::{App, Application, Window};
use gpui_component;
use gpui_term::{TerminalBuilder, TerminalView, ComponentThemeExt, TextStyle};

fn main() {
    Application::new().run(|cx: &mut App| {
        // 1. 初始化 gpui-component 主题系统
        gpui_component::init(cx);

        // 2. 打开窗口
        cx.open_window(window_options, |window, cx| {
            // 3. 创建终端
            let terminal_task = TerminalBuilder::new(
                Some(env::current_dir().unwrap()),
                Some("/bin/bash".to_string()),
                env::vars().collect(),
                None,
                window.window_handle().window_id().as_u64(),
                cx,
            );

            // 4. 创建视图
            let view = cx.new(|cx| {
                AppView {
                    terminal: None,
                    terminal_view: None,
                }
            });

            // 5. 异步创建终端
            let view_clone = view.downgrade();
            cx.spawn(async move |cx| {
                let builder = terminal_task.await?;

                cx.update_window(window_handle, |_, window, cx| {
                    view_clone.update(cx, |app, cx| {
                        let terminal = cx.new(|cx| builder.subscribe(cx));

                        // 从 gpui-component 主题创建 TextStyle
                        let text_style = TextStyle::from_component_theme(cx);

                        let terminal_view = cx.new(|cx| {
                            let mut view = TerminalView::new_with_style(
                                terminal.clone(),
                                text_style,
                                window,
                                cx
                            );

                            // ⭐ 关键：应用并观察 gpui-component 主题
                            view.apply_component_theme(cx);
                            view.observe_component_theme(cx);

                            view
                        });

                        app.terminal = Some(terminal);
                        app.terminal_view = Some(terminal_view);
                    });
                })?;

                Ok::<_, anyhow::Error>(())
            })
            .detach();

            view
        })
        .unwrap();
    });
}

// 便利方法：从 gpui-component 主题创建 TextStyle
impl TextStyle {
    #[cfg(feature = "gpui-component")]
    pub fn from_component_theme(cx: &App) -> Self {
        use gpui_term::ThemeAdapter;
        use gpui_component::ActiveTheme;

        let component_theme = cx.theme();
        let terminal_theme = ThemeAdapter::to_terminal_theme(&component_theme.colors);

        Self {
            font: Font {
                family: component_theme.mono_font_family.clone(),
                weight: FontWeight::NORMAL,
                style: FontStyle::Normal,
            },
            font_size: component_theme.mono_font_size,
            font_weight: FontWeight::NORMAL,
            foreground: terminal_theme.foreground,
            background: terminal_theme.background,
            line_height_multiplier: 1.2,
            letter_spacing: 0.0,
            theme: terminal_theme,
        }
    }
}
```

## 主题切换

当使用 gpui-component 的主题切换功能时，终端会自动更新：

```rust
// gpui-component 的主题切换
cx.dispatch_action(gpui_component::SwitchTheme("Catppuccin Latte".into()));

// 或切换 Light/Dark 模式
cx.dispatch_action(gpui_component::SwitchThemeMode(gpui_component::ThemeMode::Dark));

// 终端视图会自动响应（如果设置了 observe_component_theme）
```

## 自定义主题

如果你想自定义 gpui-component 主题中的终端颜色，可以在主题 JSON 文件中定义 base colors：

```json
{
  "name": "My Custom Theme",
  "mode": "dark",
  "colors": {
    "background": "#1e1e1e",
    "foreground": "#d4d4d4",
    "caret": "#aeafad",
    "selection": "#5b7bb359",

    "base.red": "#e06c75",
    "base.green": "#98c379",
    "base.blue": "#61afef",
    "base.yellow": "#e5c07b",
    "base.cyan": "#56b6c2",
    "base.magenta": "#c678dd",

    "base.red.light": "#ff6c75",
    "base.green.light": "#a8d389",
    "base.blue.light": "#71bfff"
  }
}
```

这些 base colors 会被映射到终端的 ANSI 颜色。

## 性能优化

### 避免重复转换

如果你有多个终端实例，可以缓存转换后的主题：

```rust
// 在应用状态中缓存
struct AppState {
    cached_terminal_theme: Option<TerminalTheme>,
}

// 只在主题真正变化时更新
cx.observe_global::<gpui_component::Theme>(|app, cx| {
    let new_theme = ThemeAdapter::from_global_theme(cx);

    // 只在主题真正改变时更新所有终端
    if app.cached_terminal_theme.as_ref() != Some(&new_theme) {
        app.cached_terminal_theme = Some(new_theme.clone());

        // 更新所有终端视图
        for terminal_view in &app.terminal_views {
            terminal_view.update(cx, |view, cx| {
                view.apply_theme(&new_theme, cx);
            });
        }
    }
});
```

## 故障排除

### 编译错误：找不到 gpui-component

确保在 Cargo.toml 中启用了 `gpui-component` feature：

```toml
gpui_term = { features = ["gpui-component"] }
```

### 主题未自动更新

确保调用了 `observe_component_theme`：

```rust
terminal_view.update(cx, |view, cx| {
    view.observe_component_theme(cx);
});
```

### 颜色看起来不对

检查 gpui-component 主题是否定义了 base colors。如果没有，ThemeAdapter 会使用默认的回退颜色。

## API 参考

### `ThemeAdapter`

```rust
pub struct ThemeAdapter;

impl ThemeAdapter {
    /// 将 gpui-component ThemeColor 转换为 TerminalTheme
    pub fn to_terminal_theme(component_theme: &ThemeColor) -> TerminalTheme;

    /// 从全局 gpui-component 主题创建 TerminalTheme
    pub fn from_global_theme(cx: &App) -> TerminalTheme;
}
```

### `ComponentThemeExt`

```rust
pub trait ComponentThemeExt {
    /// 应用当前 gpui-component 主题到终端
    fn apply_component_theme(&mut self, cx: &mut Context<Self>);

    /// 设置主题观察器，自动响应主题变化
    fn observe_component_theme(&mut self, cx: &mut Context<Self>);
}

impl ComponentThemeExt for TerminalView { /* ... */ }
```

## 相关链接

- [gpui-component 文档](https://github.com/longbridge/gpui-component)
- [gpui-term 文档](../README.md)
- [GPUI 文档](https://www.gpui.rs/)
