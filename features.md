# Features

## Core Commands

| Command | Aliases | Description |
| --- | --- | --- |
| `save <message>` | | Stage all changes, commit with message, and push to remote in one step |
| `undo [count]` | | Undo the last commit while keeping changes staged |
| `sync` | | Pull latest changes and push in a single command |
| `history` | `log` | View a clean, formatted commit history |
| `status` | `st` | Display current branch, remote status, and colored file changes |
| `stash [subcmd]` | | Quickly stash and manage uncommitted changes |
| `help` | | Show help message with all commands and aliases |

---

## save

Stage all modified and new files, commit with a message, and push to the remote repository.

**Flags:**
- `--skip-push` / `--no-push` — commit locally without pushing
- `--amend` — amend the last commit instead of creating a new one

**Examples:**
```bash
git-helper save "fix: resolve login redirect bug"
git-helper save --skip-push "wip: refactoring database layer"
git-helper save --amend "fix: better commit message"
```

**Behavior:**
- Runs `git add .`, `git commit -m <message>`, then `git push`
- Skips push if no upstream branch is configured (shows setup hint)
- Detects no-op (no changes to commit) and exits gracefully

---

## undo

Revert recent commits while keeping changes in the working directory.

**Flags:**
- `--hard` — discard changes completely (irreversible)
- `--count <N>` / `-n <N>` — undo N commits at once

**Examples:**
```bash
git-helper undo
git-helper undo -n 3
git-helper undo --hard
git-helper undo --count 5 --hard
```

**Behavior:**
- Default: `git reset --soft HEAD~1` (changes remain staged)
- With `--hard`: `git reset --hard HEAD~1` (changes lost)
- Accepts a bare number as positional arg for count

---

## sync

Pull the latest changes from remote and push local commits in one command.

**Flags:**
- `--rebase` — use rebase instead of fast-forward merge when pulling

**Examples:**
```bash
git-helper sync
git-helper sync --rebase
```

**Behavior:**
- Default pull mode: `--ff-only` (fast-forward only)
- With `--rebase`: `git pull --rebase`
- Warns if there are uncommitted changes
- Shows ahead/behind counts after sync
- Errors if no upstream branch is configured

---

## history

Display the commit log in a clean, readable format.

**Aliases:** `log`

**Flags:**
- `--graph` / `-g` — show branch graph visualization with colored output
- `--all` / `-a` — include commits from all branches
- `--oneline` / `-o` — compact one-line-per-commit format (default)
- `--detailed` / `-d` — show full commit details with diffs
- `--count <N>` / `-n <N>` — limit to N commits

**Examples:**
```bash
git-helper history
git-helper history --graph
git-helper log --graph --all
git-helper history -n 5
git-helper history --detailed -n 3
```

**Behavior:**
- Default: shows last 20 commits in oneline format
- Graph mode uses colored pretty format with author and relative time
- Detailed mode shows patch content

---

## status

Show the current state of the repository.

**Aliases:** `st`

**Examples:**
```bash
git-helper status
git-helper st
```

**Behavior:**
- Displays current branch name (colored green)
- Shows ahead/behind counts relative to upstream
- Lists changed files with colored status indicators:
  - `?` yellow — untracked
  - `M` cyan — modified
  - `A` green — added
  - `D` red — deleted
  - `R` magenta — renamed
- Reports "Working tree clean" when applicable

---

## stash

Quickly save and restore unfinished changes.

**Subcommands:**
| Subcommand | Description |
| --- | --- |
| *(none)* | Stash current changes with auto-generated message |
| `pop` | Restore and remove the latest stash |
| `apply` | Apply latest stash without removing it |
| `list` / `ls` | List all stashes |
| `drop` | Drop the latest stash |
| `clear` | Remove all stashes (irreversible) |

**Examples:**
```bash
git-helper stash
git-helper stash pop
git-helper stash list
git-helper stash drop
```

---

## Typo Tolerance

Misspelled commands and stash subcommands get automatic suggestions via Levenshtein distance matching:

```bash
$ git-helper histroy
error: unknown command 'histroy'.
hint: Did you mean 'history'?

$ git-helper statis
error: unknown command 'statis'.
hint: Did you mean 'status'?

$ git-helper stash lis
unknown stash subcommand: 'lis'
hint: Did you mean 'list'?
```

This works for all top-level commands and stash subcommands with up to 2 character errors.

---

## Configuration

Git Helper reads optional settings from `~/.git-helper/config`.

**Supported keys:**
- `default_push` — whether to push after save (default: true)
- `default_rebase` — whether to use rebase in sync (default: false)
- `history_count` — default number of commits to show (default: 20)
- `auto_fetch` — whether to auto-fetch before status (default: false)

**Example config:**
```text
# ~/.git-helper/config
default_push = true
default_rebase = false
history_count = 30
```

Lines starting with `#` or `//` are comments.

---

## Colored Output

The CLI uses colored terminal output for improved readability:
- Green for success messages and branch names
- Cyan for command steps, suggestions, and modified files
- Yellow for warnings, untracked files, and remote info
- Red for errors and deleted files
- Magenta for renamed files

---

## Error Handling

- Graceful error messages with `error:` prefix in red
- Informational warnings with `warning:` prefix
- Typo suggestions with `hint:` prefix in cyan
- Step-by-step progress output with `▶` prefix
- Success confirmation with `✔` prefix
- Non-zero exit code on failure (compatible with CI/CD pipelines)
- Help text displayed for missing or incorrect arguments

---

## Cross-Platform

- Written in Rust, compiles on Windows, macOS, and Linux
- No external runtime dependencies
- Single binary — no installation beyond placing the executable on PATH
- Handles Windows CRLF line endings gracefully
