# dotfiles

![Demo of the dotfiles TUI](docs/assets/dotfiles-demo.gif)

## Overview

`dotfiles` is a terminal user interface built with [Ratatui](https://ratatui.rs) for browsing and executing the shell scripts that configure your development environment. It reads a declarative YAML configuration, shows the resulting tool graph, and runs the scripts in dependency order.

## Features

- Manage tooling scripts defined in `~/.dotfiles/config.yaml`
- Inspect tool metadata, filesystem paths, and dependency maps
- Preview the underlying shell script directly in the UI
- Execute tools in dependency-aware batches with real-time logging
- Dual-pane layout for dotfile definitions and workflow execution

## Prerequisites

- Rust toolchain (edition 2024) — install via [rustup](https://rustup.rs)
- `zsh` available on your `PATH` (used to run tool scripts)
- macOS or Linux terminal that supports ANSI escape sequences

## Installation

```sh
git clone https://github.com/your-user/dotfiles.git
cd dotfiles
cargo build --release
# Optional: install into ~/.cargo/bin
cargo install --path .
```

## Configuration

`dotfiles` looks for `~/.dotfiles/config.yaml`. The first run creates the directory and seeds a commented template if it does not exist. Each entry under `Preferences.ToolsSettings` represents a tool:

```yaml
SystemPreferences:
  Root: ~/.dotfiles         # Base directory for managed tool folders
Preferences:
  ToolsSettings:
    - Id: shell             # Optional explicit identifier
      Name: Shell Setup     # Friendly name shown in the UI
      File: shell-setup.zsh # Script invoked when the tool runs
    - Id: brew
      Name: Homebrew
      Root: brew            # Subdirectory at SystemPreferences.Root
      File: brew.zsh
      Dependencies:         # Other tool Ids that must run first
        - shell             # Must match another tool's Id
```

Dependencies must reference the `Id` (explicit or generated) of another tool entry. If `Id`, `Root`, or `File` are omitted, the application derives sensible defaults from `Name`. Missing directories or script files are created automatically with placeholders.

## Usage

### Running the app

```sh
cargo run --release
```

The UI opens with two tabs:

- `Dotfiles` shows the configured tools, dependency tree, and script preview.
- `Workflow` lets you run the scripts in dependency order and tail structured logs.

### Key bindings

- `Tab` — toggle between panes (menu vs. script/log view)
- Arrow keys — move selection in menus or scroll text areas
- `Home` / `End` — jump to start or end of lists/logs/scripts
- `Enter` (Workflow menu) — start running tools
- `q`, `Esc`, or `Ctrl+C` — quit the application

While a workflow run is active, the application streams log output and summarises successes and failures after each dependency stage.

## Development

Useful commands during development:

```sh
cargo fmt
cargo clippy --all-targets --all-features
cargo test
```

Run `cargo run` without `--release` for faster feedback during iterative work.

## License

Copyright (c) TomoyukiSugiyama <s_tomoyuki07@yahoo.co.jp>

This project is licensed under the MIT license ([LICENSE] or <http://opensource.org/licenses/MIT>)

[LICENSE]: ./LICENSE
