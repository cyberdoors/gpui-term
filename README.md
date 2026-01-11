# gpui-term

GPUI-based terminal component built on Alacritty's terminal emulation library.

## Quick start

Run the basic example:

```sh
cargo run -p basic
```

## Configuration

On startup, gpui-term loads a config file and writes a default one if missing.

- Default path: `~/.config/gpui-term/config.toml`
- Override path: set `GPUI_TERM_CONFIG=/path/to/config.toml`

Supported settings:

- `font_family`: font family name (string).
- `font_size`: font size in pixels (float).
- `line_height`: line-height multiplier (float).
- `letter_spacing`: extra horizontal spacing in pixels (float).
- `theme`: colors for ANSI, foreground, background, cursor, and selection.

Color values are hex strings in `#RRGGBB` or `#RRGGBBAA` format. For transparent
backgrounds, set an alpha value like `#1e1e1e00`.

Example:

```toml
font_family = "FiraCode Nerd Font"
font_size = 14.0
line_height = 1.2
letter_spacing = 0.0

[theme]
foreground = "#d4d4d4ff"
background = "#1e1e1e00"
cursor = "#aeafadff"
selection = "#5b7bb359"
ansi = ["#1e1e1eff", "#e06c75ff", "#98c379ff", "#e5c07bff", "#61afefff", "#c678ddff", "#56b6c2ff", "#abb2bfff"]
bright = ["#5c6370ff", "#e06c75ff", "#98c379ff", "#e5c07bff", "#61afefff", "#c678ddff", "#56b6c2ff", "#dfdfffff"]
dim = ["#1e1e1eff", "#be5b65ff", "#7a9f60ff", "#d19a66ff", "#4e88b8ff", "#a061b0ff", "#44919bff", "#8a8f98ff"]
bright_foreground = "#dfdfffff"
dim_foreground = "#8a8f98ff"
```
