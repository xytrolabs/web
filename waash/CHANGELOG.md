# Changelog

## [0.2.0] — 2026-08-06 — Battle-tested for programming

This release fixes several bugs that bit real programming workflows, adds
FISH-style tab completion and job-control upgrades, and is the first
"battle-tested" release intended for everyday development use.

### Fixed — things getting swallowed
- **Glob expansion** (`*`, `?`, `[...]`, `**`) now works: `ls *.rs`, `rm *.tmp`,
  `git add *.py`. Previously `*` was passed literally to commands, so globbing
  silently failed. No-match globs stay literal (POSIX). (`src/wordexp.rs`)
- **Builtins now honor redirections.** `echo "x" > file` previously wrote to
  the terminal and never created the file (builtins ran in-process before the
  redirection code). Now `>`, `>>`, `2>`, `2>&1` work for builtins exactly like
  external commands. (`src/executor/mod.rs` — `run_builtin_with_redirections`)
- **Numeric arguments are no longer swallowed as fd numbers.** `seq 1 3 > out.txt`
  was parsed as `seq 1` + `3>out.txt` (fd-3 redirect), silently eating the `3`.
  Now, matching bash, only an *adjacent* `3>file` is an fd redirect; `3 > file`
  keeps `3` as an argument. (`src/parser/mod.rs` — `is_fd_prefixed_redirection`)
- **`jobs` shows readable commands**, not debug structs (`Simple(SimpleCommand{...})`
  → `sleep 5`). Added `render()` to the AST. (`src/parser/ast.rs`)

### Added
- **FISH-style tab completion**: the candidate list now appears on the **first**
  Tab (was bash-style double-Tab). (`vendor/rustyline` patch)
- **`disown [N]`** — stop tracking a background job (no zombies).
- **`wait [N]`** — block until a background job (or all) finishes; returns its
  exit status.
- **`jobs`** now shows index, `+`/`-` marker, state, PID and command.
- **git completion**: `git che<Tab>` → subcommands; `git checkout <Tab>` →
  branch names (local + remote).

### Fixed (previous)
- Live prompt flicker (vendored rustyline renders atomically).
- Live prompt auto-disabled in VS Code's integrated terminal.
- Login-shell support: sources `/etc/profile` + `~/.profile`, `startup_commands`.
- Safe login (5s profile timeout, stdin from /dev/null).

[0.2.0]: https://xytro.site/waash
