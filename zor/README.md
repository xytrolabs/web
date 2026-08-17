# Zor Language (.zor)

Zor is a clean, simple language that transpiles to Rust. Write Rust without the noise — build apps, OS kernels, and desktop environments with a zen syntax.

## Quick Install

### 🐧 Any Linux (universal installer)

```bash
curl -fsSL https://raw.githubusercontent.com/xytrolabs/zor/main/scripts/install.sh | bash
```

Installs to `~/.local/` — no root needed.

### 🪟 Windows (PowerShell)

```powershell
powershell -c "irm https://raw.githubusercontent.com/xytrolabs/zor/main/scripts/install.ps1 | iex"
```

## Quickstart

```zor
fun main int {
    say "Hello from Zor!"
    return 0
}
```

```bash
zor run hello.zor
# → Hello from Zor!
```

📖 Full guide: [docs/ZOR_GUIDE.md](docs/ZOR_GUIDE.md)

## Features

- **Transpiles to Rust** — production performance, all crates available
- **Clean syntax** — no semicolons, minimal punctuation
- **Indentation-based** — braces optional for simple bodies
- **Borrow checker** — `&T` / `&mut T` references with safety
- **FFI** — `extern "C"` for calling C libraries
- **OS dev** — `#![no_std]` via `ZOR_NO_STD=1`
- **Dot paths** — `gtk4.Application.new()` instead of `::`
- **Error translation** — Rust errors mapped to Zor source
- **Multi-file projects** — `zor build src/`
- **Auto-update** — `zor --update` keeps you current

## Run Zor

```bash
# Run a script
zor run app.zor

# Compile to native binary
zor build app.zor

# Check syntax only
zor check app.zor

# Update to latest version
zor --update

# Run tests
zor test tests/

# Format code
zor fmt app.zor

# Build multi-file project
zor build src/
```

## Build from Source

```bash
cd zor-native
cargo build --release
./target/release/zor --version
```

## Documentation

| Doc | Description |
|---|---|
| [docs/ZOR_GUIDE.md](docs/ZOR_GUIDE.md) | Full language reference |
| [docs/OS_DEV_GUIDE.md](docs/OS_DEV_GUIDE.md) | Bare-metal kernel development |
| [docs/QUICK_REFERENCE.md](docs/QUICK_REFERENCE.md) | Syntax cheat sheet |

## VS Code Language Support

The `syntaxes/` directory contains VS Code language support:
- Syntax highlighting for `.zor`
- Language configuration (comments/brackets)
- Snippets for common Zor patterns

## Standard Library

Starter modules in `std/`:

- `std/io.zor` — File I/O
- `std/math.zor` — Math functions
- `std/strings.zor` — String operations
- `std/testing.zor` — Test framework
- `std/net.zor` — HTTP/TCP networking
- `std/crypto.zor` — Hashing & crypto
- `std/os.zor` — OS utilities
- `std/async.zor` — Async runtime stubs

## OS Development

```bash
ZOR_NO_STD=1 zor build kernel.zor
# → kernel.o — bare metal ELF64 object
```

See [docs/OS_DEV_GUIDE.md](docs/OS_DEV_GUIDE.md) for the full kernel tutorial.

## Desktop Apps

```zor
use "gtk4"

fun main int {
    var app = gtk4.Application.new("com.myapp")
    var win = gtk4.ApplicationWindow.new(app)
    win.set_title("Zor Desktop App")
    win.show()
    return app.run()
}
```

Zor transpiles to Rust — use any Rust crate (gtk4, iced, egui, tokio, serde...).

## License

MIT

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)
