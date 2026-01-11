# Repository Guidelines

## Project Structure & Module Organization
- `crates/gpui_term/` contains the core terminal library (terminal model, view, element, and input mappings).
- `examples/basic/` is the demo application used for manual testing and UI iteration.
- `Cargo.toml` defines the workspace; `README.md` covers quick-start usage; `target/` holds build artifacts.

## Build, Test, and Development Commands
- `cargo build` compiles the full workspace (library and examples).
- `cargo build --release` produces optimized artifacts for profiling or distribution.
- `cargo run -p basic` launches the basic GPUI terminal example.
- `cargo check` runs a fast type-check without full code generation.
- `cargo test` runs unit tests across workspace crates.

## Coding Style & Naming Conventions
- Rust edition is 2024; ensure your toolchain supports it.
- Use default rustfmt style; run `cargo fmt` before opening a PR.
- Naming: `snake_case` for modules/functions, `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
- Keep rendering logic in `terminal_element.rs`, view/input handling in `terminal_view.rs`, and key/mouse mapping in `mappings/`.

## Testing Guidelines
- Uses Rust's built-in test harness (`#[test]`), colocated in source files.
- Current tests live in `crates/gpui_term/src/mappings/keys.rs` and `crates/gpui_term/src/terminal_element.rs`.
- Prefer descriptive `test_*` names that state expected behavior.
- No explicit coverage target; add tests when changing input mapping or rendering behavior.

## Commit & Pull Request Guidelines
- Commit history favors short, imperative summaries; both English and Chinese messages appear, so keep it concise.
- PRs should describe behavior changes, link issues when relevant, and include screenshots or recordings for UI updates.
- Note the test command(s) you ran, or explicitly state when tests were not run.

## Configuration & Runtime
- Default config path: `~/.config/gpui-term/config.toml`.
- Override with `GPUI_TERM_CONFIG=/path/to/config.toml`; colors use `#RRGGBB` or `#RRGGBBAA` hex values.
