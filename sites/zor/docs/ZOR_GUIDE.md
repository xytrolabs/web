# Your First Steps with Zor

> Zor transpiles to Rust. You write clean, readable code — Zor turns it into production Rust that compiles to native binaries. No semicolons, no `::` paths, no ceremony. Just code that works.

---

## 1. Hello, Zor!

Create your first file. Call it `hello.zor`:

```zor
fun main int {
    say "Hello, World!"
    return 0
}
```

Run it:

```bash
zor run hello.zor
# → Hello, World!
```

**What just happened?** Zor read your file, translated it to Rust, compiled it with `rustc`, and ran the resulting binary. You just wrote Rust — without writing Rust.

Every Zor program needs a `main` function that returns `int`. `say` prints to the screen. That's all you need to get started.

---

## 2. Storing Values — Variables

```zor
var name string = "Ada"
var age int = 28
var price float = 9.99
var active bool = true
```

The colon before the type is optional:

```zor
var name: string = "Ada"   // both work
var name string = "Ada"    // both work
```

Zor has five basic types: `int`, `float`, `string`, `bool`, and `char`.

You can change a variable's value anytime:

```zor
name = "Grace"
age = 29
```

---

## 3. Printing — `say`

`say` is your window into your program. It prints anything:

```zor
say "Hello"           // Hello
say 42                // 42
say "Age: " + age     // Age: 29
```

You can print expressions directly:

```zor
say x + y
say len("hello")      // 5
```

---

## 4. Functions — Your Own Commands

Functions let you name a piece of code and reuse it:

```zor
fun greet(name string) {
    say "Hello, " + name
}
```

If a function returns a value, declare the return type:

```zor
fun add(a int, b int) int {
    return a + b
}
```

Call functions the way you'd expect:

```zor
greet("Ada")
var result int = add(10, 20)
say result             // 30
```

### Parameters — Colon Is Optional

Both styles work. Pick what feels natural:

```zor
fun greet(name: string) { ... }   // with colon
fun greet(name string) { ... }    // without colon
```

### Async Functions

Need to work with the network or files without blocking?

```zor
async fun fetch(url string) string {
    return "data from " + url
}
```

---

## 5. Making Decisions — `if`

```zor
var score int = 85

if score >= 90 {
    say "Excellent!"
} else if score >= 70 {
    say "Good job!"
} else {
    say "Keep practicing!"
}
```

The curly braces `{ }` mark where each branch starts and ends. Zor uses `else if` (not `elif` or `or`).

---

## 6. Loops — Doing Things Repeatedly

### The Forever Loop

```zor
loop {
    say "This never stops!"
    if done { stop }     // stop = break out
}
```

### While Something Is True

```zor
var count int = 0
while count < 5 {
    say count
    count = count + 1
}
```

### Repeat N Times

```zor
repeat 3 {
    say "Hello!"         // prints 3 times
}
```

### Loop Controls

| Keyword | What it does |
|---|---|
| `stop` | Exit the loop immediately |
| `next` | Skip to the next iteration |

---

## 7. Grouping Data — Structs

When you need to keep related values together, use a `struct`:

```zor
struct Person {
    name string
    age int
}

fun main int {
    var ada Person = Person { name: "Ada", age: 28 }
    say ada.name          // "Ada"
    say ada.age           // 28
    return 0
}
```

---

## 8. Multiple Choices — Enums & `match`

Enums let you say "this value is one of these options":

```zor
enum Color {
    Red
    Green
    Blue
    Custom(int, int, int)      // RGB values
}
```

Use `match` to handle each possibility:

```zor
fun describe(c Color) string {
    match c {
        Red { return "Stop" }
        Green { return "Go" }
        Blue { return "Sky" }
        Custom(r, g, b) { return "Custom color" }
    }
}
```

> The Zor compiler checks that you handle every variant. Miss one, and it tells you. This catches bugs before your code even runs.

---

## 9. References — Borrowing Without Copying

When you pass a large struct to a function, Zor can *move* it (transfer ownership) or *borrow* it (lend a reference).

### Immutable Borrow — Read-Only

```zor
fun inspect(p &Point) {
    say p.x        // can read, can't change
}
```

### Mutable Borrow — Read & Write

```zor
fun modify(p &mut Point) {
    p.x = 100      // can change the original
}
```

### Using References

```zor
fun main int {
    var pt Point = Point { x: 10, y: 20 }
    inspect(&pt)       // &pt = lend a read-only reference
    modify(&mut pt)    // &mut pt = lend a mutable reference
    return 0
}
```

> **The golden rule**: You can have many `&` references OR one `&mut` reference, but never both at the same time. Zor enforces this at compile time — you'll never have a data race.

---

## 10. Error Handling — Things Will Go Wrong

### The `?` Operator — Propagate Errors

When a function can fail, use `?` to pass the error up the call chain:

```zor
fun read_config() string {
    var data = read_file("config.txt")?
    return data
}
```

If `read_file` fails, the function returns the error immediately. If it succeeds, `data` gets the value.

### `attempt` / `catch` — Handle Errors Locally

```zor
fun main int {
    attempt {
        var f = read_file("config.txt")
        say f
    } catch {
        say "Couldn't read config — using defaults"
    }
    return 0
}
```

---

## 11. Talking to C — The `extern` Block

Need to call a C library? Use `extern "C"`:

```zor
extern "C" {
    fun puts(s int) -> int
    fun printf(fmt int, ...) -> int
}

fun main int {
    puts("Hello from C!")
    return 0
}
```

> **Important**: FFI calls must be wrapped in `unsafe { }` blocks. Zor will remind you if you forget.

---

## 12. Unsafe — When You Need Raw Power

For OS kernels, hardware access, and FFI, Zor gives you an escape hatch:

```zor
fun kernel_main int {
    var vga int = 0xB8000       // VGA text buffer address
    unsafe {
        vga[0] = 'Z' | 0x0700   // write 'Z' in white-on-black
    }
    loop {}
    return 0
}
```

`unsafe` tells Zor: "I know what I'm doing. Trust me." Use it sparingly.

---

## 13. Using Rust Crates

Zor transpiles to Rust, so you can use any Rust crate:

```zor
use "gtk4"
use "serde"
use "tokio"

fun main int {
    var app = gtk4.Application.new("com.myapp")
    return app.run()
}
```

Notice the dot notation: `gtk4.Application.new()` instead of `gtk4::Application::new()`. Cleaner.

---

## 14. Closures — Functions Without Names

Sometimes you need a quick throwaway function:

```zor
fun main int {
    var double = |x| { return x * 2 }
    say double(21)       // 42
    return 0
}
```

Closures capture the variables around them automatically.

---

## 15. Organizing Code — Modules & Imports

Split your code across files with `get`:

```zor
get "math"         // imports math.zor from the same directory
get "strings"      // imports strings.zor
```

Use external crates with `use`:

```zor
use "serde"
use "std.io"
```

---

## 16. Traits — Shared Behavior

Define what something *can do* with a trait:

```zor
trait Animal {
    fun speak() string
}
```

Then implement it for your types:

```zor
struct Dog { name string }

impl Animal for Dog {
    fun speak() string {
        return "Woof! I'm " + name
    }
}
```

---

## 17. Building Real Projects

### Staying Current

```bash
zor --update           # Auto-update to the latest version
```

This pulls the latest code from GitHub, builds it, and replaces your current binary.

For single files:

```bash
zor run app.zor        # compile + run
zor build app.zor      # compile to binary
zor check app.zor      # syntax check only
```

For multi-file projects:

```bash
zor build src/         # compiles all .zor files in src/
```

For OS kernels (no standard library):

```bash
ZOR_NO_STD=1 zor build kernel.zor
```

---

## 18. Understanding Errors

Zor catches problems early and explains them clearly:

| Rust Code | What it means | What to do |
|---|---|---|
| `E0308` | Type mismatch | Check what you're passing matches the function signature |
| `E0425` | Name not found | Declare with `var` or import with `use`/`get` |
| `E0382` | Value was moved | Pass `&value` to borrow instead of moving |
| `E0596` | Can't borrow mutably | Use `&mut` reference |
| `E0502` | Borrow conflict | Can't have mutable + immutable borrows at once |
| `E0133` | FFI needs unsafe | Wrap in `unsafe { ... }` |
| `E0601` | No main function | Add `fun main int { ... }` |
| `E0277` | Trait not implemented | Your type needs to implement the required trait |
| `E0061` | Wrong argument count | Check the function's parameter list |

### Common Fixes

**"rustc not found"** → Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Ownership error** → If a value was moved, pass `&value`. If you need mutation, use `&mut value`. If you need a copy, call `.clone()`.

**"ZOR_NO_STD=1 linker error"** → OS kernels need a linker script and boot assembly. See `examples/kernel/` for a template.

---

## Quick Reference

| Zor | What it does |
|---|---|
| `fun name() type { }` | Define a function |
| `var x type = value` | Declare a variable |
| `say expr` | Print to screen |
| `return expr` | Return from function |
| `if cond { } else { }` | Conditional |
| `loop { }` | Infinite loop |
| `while cond { }` | While loop |
| `repeat N { }` | Repeat N times |
| `stop` / `next` | Break / continue |
| `struct S { }` | Define a struct |
| `enum E { }` | Define an enum |
| `match x { }` | Pattern match |
| `&x` / `&mut x` | Immutable / mutable reference |
| `unsafe { }` | Raw pointer / FFI access |
| `extern "C" { }` | C function declarations |
| `use "crate"` | Import Rust crate |
| `get "module"` | Import Zor module |
| `trait T { }` | Define a trait |
| `impl T for S { }` | Implement a trait |
| `async fun f()` | Async function |
| `\|x\| { x*2 }` | Closure |
| `expr?` | Propagate error |

---

Next: [OS Development Guide](OS_DEV_GUIDE.md) · [Quick Reference](QUICK_REFERENCE.md)

```bash
# Single file
zor run app.zor

# Multi-file project
zor build src/

# OS kernel (no standard library)
ZOR_NO_STD=1 zor build kernel.zor
```

## Syntax

### Functions

```zor
// No parentheses needed for empty params
fun main int {
    return 0
}

// Parameters: colon between name and type is optional
fun add(a int, b int) int {
    return a + b
}
fun greet(name: string) string {
    return "Hello, " + name
}

// Async functions
async fun fetch(url string) string {
    return "data"
}
```
