已加入可配置的终端主题与字体参数，并在启动时从配置文件加载（不存在则写入默认配置）。

  做了这些改动：

  - 新增配置模块：crates/gpui_term/src/config.rs，提供 TerminalConfig / TerminalThemeConfig /
    TerminalTheme，支持字体/字号/行高/字距/配色方案，并持久化到配置文件。
  - 渲染层接入配置：crates/gpui_term/src/terminal_element.rs 的 TextStyle 新增 letter_spacing
    和 theme，颜色转换与选区颜色使用主题；cell 宽度叠加字距；光标颜色使用主题；提供
    TextStyle::from_config。
  - 视图层支持样式：crates/gpui_term/src/terminal_view.rs 新增 new_with_style，TerminalView
    持有样式并传给 TerminalElement。
  - 示例加载配置：examples/basic/src/main.rs 启动时 TerminalConfig::load_or_create()，并用
    TextStyle::from_config 注入渲染。

  配置文件位置与规则：

  - 默认路径：~/.config/gpui-term/config.toml
  - 可用环境变量覆盖：GPUI_TERM_CONFIG=/path/to/config.toml
  - 启动时若不存在会自动写入默认配置（保持原有透明背景效果，背景色默认 #1e1e1e00）。

  示例配置字段（默认会生成）：

  font_family = "FiraCode Nerd Font"
  font_size = 14.0
  line_height = 1.2
  letter_spacing = 0.0

  [theme]
  foreground = "#d4d4d4ff"
  background = "#1e1e1e00"
  cursor = "#aeafadff"
  selection = "#5b7bb359"
  ansi = ["#1e1e1eff", "#e06c75ff", "#98c379ff", "#e5c07bff", "#61afefff", "#c678ddff",
  "#56b6c2ff", "#abb2bfff"]
  bright = ["#5c6370ff", "#e06c75ff", "#98c379ff", "#e5c07bff", "#61afefff", "#c678ddff",
  "#56b6c2ff", "#dfdfffff"]
  dim = ["#1e1e1eff", "#be5b65ff", "#7a9f60ff", "#d19a66ff", "#4e88b8ff", "#a061b0ff",
  "#44919bff", "#8a8f98ff"]
  bright_foreground = "#dfdfffff"
  dim_foreground = "#8a8f98ff"

  自然的下一步建议：

  1. 运行 cargo run -p basic，确认修改配置后实时生效。
  2. 告诉我是否要加热加载（监听配置文件变更，自动刷新样式）。