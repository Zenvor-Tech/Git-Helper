# Git Helper CLI

A lightweight Rust-based command-line tool that simplifies common Git workflows into short, easy-to-remember commands.

Instead of writing long Git commands, perform everyday operations like:

```text
git-helper save "fix: resolve login bug"   # stage, commit, push in one step
git-helper undo                             # undo last commit, keep changes
git-helper sync --rebase                    # pull (rebase) + push
git-helper history --graph                  # view log with branch graph
git-helper status                           # colored repo status
git-helper stash                            # stash changes quickly
```

## Features

- **save** - Stage all changes, commit, and push with a single command. Supports `--skip-push` and `--amend`.
- **undo** - Undo recent commits while keeping changes staged. Use `--hard` to discard changes or `-n 3` to undo multiple commits.
- **sync** - Pull and push in one command. Use `--rebase` for a rebase-based pull.
- **history** - View a clean commit history with support for `--graph`, `--all`, `--detailed`, and `--count`.
- **status** - Display current branch, ahead/behind remote, and colored file status.
- **stash** - Manage stashes with subcommands: push, pop, list, apply, drop, clear.

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) (1.70 or later)
- [Git](https://git-scm.com/)

### Build from source

```bash
git clone https://github.com/yourusername/git-helper.git
cd git-helper
cargo build --release
```

The binary will be at `target/release/git-helper` (or `git-helper.exe` on Windows).

### Optional: Add to PATH

Copy the binary to a directory in your PATH:

```bash
# Linux / macOS
cp target/release/git-helper ~/.local/bin/

# Windows
copy target\release\git-helper.exe %USERPROFILE%\.cargo\bin\
```

## Usage

```text
git-helper <command> [options] [args]
```

### Commands

| Command | Description |
| ------ | --- |
| `save <message>` | Stage all, commit, and push |
| `undo [count]` | Undo last commit(s), keep changes staged |
| `sync` | Pull then push |
| `history` | Show commit log |
| `status` | Show repo status |
| `stash [subcmd]` | Manage stashes |
| `help` | Show help |

### Examples

```bash
# Quick save and push
git-helper save "feat: add user authentication"

# Save locally without pushing
git-helper save --skip-push "wip: refactoring"

# Undo the last 3 commits (keep changes)
git-helper undo -n 3

# Undo and discard changes (irreversible)
git-helper undo --hard

# Sync with rebase
git-helper sync --rebase

# View history with graph
git-helper history --graph --all

# Stash current work
git-helper stash

# Restore stashed work
git-helper stash pop
```

## Configuration

Git Helper reads from `~/.git-helper/config` if it exists:

```text
default_push = true
default_rebase = false
history_count = 20
auto_fetch = false
```

Lines starting with `#` or `//` are treated as comments.

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Release build
cargo build --release
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git-helper save "feat: amazing feature"`)
4. Push to the branch (`git-helper sync`)
5. Open a Pull Request

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
