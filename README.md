# dotfiles

![Demo of the dotfiles TUI](docs/assets/dotfiles-demo.gif)

## Overview

`dotfiles` is a terminal user interface built with [Ratatui](https://ratatui.rs) for browsing and executing the shell scripts that configure your development environment. It now also ships a non-TUI CLI for packaging and reinstalling your setup across machines.

## Features

- Manage tooling scripts defined in `~/.dotfiles/config.yaml`
- Inspect tool metadata, filesystem paths, and dependency maps
- Preview the underlying shell script directly in the UI
- Execute tools in dependency-aware batches with real-time logging
- Export your configuration plus scripts into a portable archive
- Install an exported archive onto a new machine with integrity checks

## Prerequisites

- Rust toolchain (edition 2024) — install via [rustup](https://rustup.rs)
- `zsh` available on your `PATH` (used to run tool scripts)
- macOS or Linux terminal that supports ANSI escape sequences

## Installation

### Homebrew (recommended)

```sh
brew tap TomoyukiSugiyama/homebrew-tap
brew "tomoyukisugiyama/tap/dotfiles"
```

To upgrade later, run `brew upgrade dotfiles`.

### Build from source

```sh
git clone https://github.com/your-user/dotfiles.git
cd dotfiles
cargo build --release
# Optional: install into ~/.cargo/bin
cargo install --path .
```

Once installed you can launch the TUI with `dotfiles --tui` (or simply `dotfiles`) and run the CLI subcommands directly (e.g. `dotfiles export`).

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

### Run the TUI

```sh
cargo run --release
# or, if installed:
dotfiles --tui
# the binary also launches the TUI by default
dotfiles
```

The UI opens with two tabs:

- `Dotfiles` shows the configured tools, dependency tree, and script preview.
- `Workflow` lets you run the scripts in dependency order and tail structured logs.

### Export an environment archive

Create a portable archive (default `tar.gz`) containing your `config.yaml`, tool graph metadata, and the associated scripts:

```sh
# write dotfiles-export-<timestamp>.tar.gz next to the command
cargo run --release -- export --dest ./backup

# explicit format and path
dotfiles export --dest ~/Desktop/my-dotfiles --format zip
```

Each archive contains a manifest with file hashes and permissions so that installs can verify integrity before writing anything to disk.

### Install from an archive

```sh
# interactively choose the destination root (defaults to value stored in the archive)
cargo run --release -- install --src ~/Desktop/my-dotfiles.tar.gz

# non-interactive install into a custom path
dotfiles install --src ./backup.tar.gz --dest ~/.dotfiles --non-interactive
```

The installer performs the following steps:

- Extracts into a temporary directory and validates every file against the manifest hashes
- Prompts for (or accepts) the destination root directory
- Creates a timestamped backup for any file that would be overwritten
- Restores permissions after copying scripts and config

Once completed you can launch the TUI on the new machine and run workflows immediately.

### Key bindings (TUI)

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

Run `cargo run` without `--release` for faster feedback during iterative work. You can also invoke the CLI directly with `cargo run -- export …` or `cargo run -- install …` to test the packaging flows.

## License

Copyright (c) TomoyukiSugiyama <s_tomoyuki07@yahoo.co.jp>

This project is licensed under the MIT license ([LICENSE] or <http://opensource.org/licenses/MIT>)

[LICENSE]: ./LICENSE
