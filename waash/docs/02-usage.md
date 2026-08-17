# 02 — Using WAASH like Bash

WAASH is a drop-in shell for everyday bash-style workflows. Everything you
type at the prompt behaves like bash, with FISH-style niceties on top.

---

## Running Commands

Type a command name followed by arguments:

```
% ls -la /home
% git status
% cargo build --release
```

WAASH searches your `PATH` for commands, just like bash. If a command isn't
found you'll see:

```
waash: notacommand: command not found
```

### Command lookup order

1. **Builtins** (run inside the shell — see below)
2. **Aliases** (from your config)
3. **External programs** on `PATH`

---

## Built-in Commands

These run inside WAASH (no subprocess needed):

| Builtin | Description | Example |
|---|---|---|
| `cd` | Change directory | `cd ~/Projects`, `cd -` (last dir) |
| `pwd` | Print working directory | `pwd` |
| `echo` | Print text (`-n` no newline, `-e` escapes) | `echo -n "hi"` |
| `export` | Set environment variable | `export FOO=bar` |
| `unset` | Remove environment variable | `unset FOO` |
| `exit` | Leave the shell | `exit 42` |
| `source` / `.` | Run a script in the shell | `source myscript.waash` |
| `type` | Show command type | `type ls` → `ls is /usr/bin/ls` |
| `test` / `[` | Evaluate conditions | `test -f file` |
| `true` / `false` | Always succeed/fail | |
| `:` | No-op (does nothing) | |
| `jobs` | List background jobs | `jobs` |
| `fg` / `bg` | Foreground/background a job | `fg` |
| `disown` | Stop tracking a background job | `disown 1` |
| `wait` | Wait for background job(s) | `wait 1` |
| `history` | Show command history | `history` |
| `alias` | List aliases | `alias` |
| `set` | Show shell variables | `set` |

---

## Variables

### Environment variables

```
% echo $HOME
/home/raf
% export MY_VAR="hello"
% echo $MY_VAR
hello
```

Special variables:

| Var | Meaning |
|---|---|
| `$?` | Exit status of the last command |
| `$$` | Current shell PID |
| `$!` | PID of the last background job |
| `$0` | Shell name (`waash`) |
| `$1`–`$9` | Positional parameters (in scripts) |
| `$HOME` | Home directory |
| `$PWD` | Current directory |
| `$OLDPWD` | Previous directory |

### Command-local variables

`VAR=value command` sets a variable **only for that command** (it does NOT
leak into your shell):

```
% FOO=temp printenv FOO
temp
% printenv FOO          # empty — no leak!
%
```

---

## Pipelines

Connect commands with `|`:

```
% cat file.txt | grep "error" | wc -l
42
```

`|&` pipes **both stdout and stderr**:

```
% cargo build 2>&1 | tail -5
% cargo build |& tail -5    # same thing, shorter
```

---

## Redirections

| Operator | Meaning |
|---|---|
| `>` | Write stdout to file (overwrite) |
| `>>` | Append stdout to file |
| `<` | Read stdin from file |
| `2>` | Write stderr to file |
| `2>&1` | Send stderr to stdout |
| `&>` | Redirect both stdout and stderr |
| `<>` | Open file for reading and writing |

Examples:

```
% ls > listing.txt
% echo "more" >> listing.txt
% grep pattern < input.txt > output.txt
% command 2> errors.log
% command > all.log 2>&1
% command &> everything.log
```

---

## Heredocs — WAASH's Superpower

WAASH supports **all BASH heredoc forms**. A heredoc feeds multi-line text
into a command's stdin.

### `<<EOF` — with variable expansion

```
% cat <<EOF
Hello $USER
Today is a good day
EOF
Hello raf
Today is a good day
```

### `<<'EOF'` — no expansion (literal)

```
% cat <<'EOF'
The variable $HOME is NOT expanded
EOF
The variable $HOME is NOT expanded
```

### `<<-EOF` — strip leading tabs

```
% cat <<-EOF
    indented with a tab
EOF
indented with a tab
```

### `<<<` — herestrings

```
% read <<< "hello world"
```

### Multiple heredocs

```
% cat <<A <<B
body of A
A
body of B
B
```

---

## Logical Operators

```
% make && ./run          # run ./run only if make succeeds
% make || echo "failed"  # echo only if make fails
% cmd1; cmd2             # run cmd1 then cmd2 regardless
```

Exit codes: `0` = success, non-zero = failure. `$?` holds the last status.

---

## Background Jobs

Append `&` to run a command in the background:

```
% sleep 100 &
[1] 12345
% jobs
[1] + Running  pid=12345  sleep 100
% fg                  # bring it to the foreground
% <Ctrl+C>            # stop it
```

Job control builtins:

- `jobs` — list jobs with their index, state, PID and command (the `+`
  marks the most recent job, the default target of `fg`/`bg`).
- `fg [N]` / `bg [N]` — bring a job to the foreground / resume it in the
  background.
- `disown [N]` — stop tracking a job so it is no longer reported; the
  process keeps running detached.
- `wait [N]` — block until a background job (or all of them) finishes and
  return its exit status.

WAASH automatically reaps finished background jobs, so you won't leave
zombie processes around.

---

## Command Substitution & Grouping

### Command substitution `$( )` and backticks

Replace a command with its output, inline:

```
% echo "Today is $(date +%A)"
Today is Monday
% echo The host is `hostname`
The host is raf-cachy
```

### Subshell `( )`

Runs in a child shell — changes don't affect your session:

```
% (cd /tmp && ls)
```

### Group `{ }`

Runs in the current shell:

```
% { cd /tmp && ls; }
```

---

## History

- **`↑` / `↓`** — navigate history
- **`Ctrl+R`** — reverse search history
- **`history`** — list it
- History is saved to `~/.config/waash/history` automatically

---

## Keyboard Shortcuts

| Keys | Action |
|---|---|
| `→` / `Ctrl+F` | Accept autosuggestion |
| `Tab` | Complete |
| `↑` / `↓` | History navigation |
| `Ctrl+R` | Reverse history search |
| `Ctrl+C` | Interrupt |
| `Ctrl+D` | Exit (at empty prompt) |
| `Ctrl+A` / `Ctrl+E` | Start / end of line |
| `Ctrl+W` | Delete word |

---

## Ctrl+C Behavior

WAASH ignores `SIGINT` itself, so pressing **Ctrl+C while a command runs**
kills the **command** but keeps WAASH alive:

```
% sleep 100
^C
%                    # still in WAASH!
```

---

## Next Steps

- [03 — Scripting with Indent](03-scripting.md)
- [04 — Configuration](04-configuration.md)
