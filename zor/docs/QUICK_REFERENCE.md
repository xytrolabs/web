# Zor Quick Reference

## Functions
```zor
fun name { }              # no params, no return
fun name int { return 0 } # returns int
fun name(x int) { }       # one param
fun name(x int, y int) int { return x + y }
async fun name int { return 42 }
```

## Variables
```zor
var x int = 42
var y: int = 58           # colon optional
var z = 100               # type inferred
```

## Types
```zor
int      float    string    bool     char     void
&int     &mut int *int      vec[int] map[string]int
Str      Point    Option[int]
```

## Control Flow
```zor
if cond { } else { }
if cond { } else if cond2 { } else { }
loop { }                  # forever
while cond { }
repeat N { }              # N times
stop                      # break
next                      # continue
```

## Output
```zor
say "hello"
say 42
say x + y
```

## Structs & Enums
```zor
struct Point { x int, y int }
enum Option { Some(int), None }
var p = Point { x: 10, y: 20 }
var o = Option.Some(42)
```

## Match
```zor
match value {
    Pattern1 { ... }
    Pattern2 { ... }
    _ { ... }              # wildcard
}
```

## References
```zor
&x                         # immutable ref
&mut x                     # mutable ref
*x                         # dereference
```

## Unsafe
```zor
unsafe { ptr[0] = 90 }    # raw pointer access
```

## FFI
```zor
extern "C" {
    fun puts(s int) -> int
    fun malloc(size int) -> int
}
```

## Crates & Modules
```zor
use "gtk4"                 # Rust crate
get "math"                 # Zor module (math.zor)
```

## Closures
```zor
var double = |x| { return x * 2 }
```

## Traits
```zor
trait Animal { fun speak() string }
impl Animal for Dog { fun speak() string { return "Woof!" } }
```

## Error Handling
```zor
var data = read_file("x.txt")?   # ? operator
attempt { ... } catch { ... }    # try/catch
```

## Async
```zor
async fun fetch() string { return "data" }
spawn task()
await result
```

## Operators
```zor
+ - * / %                  # arithmetic
== != < > <= >=            # comparison
&& || !                    # logical
| &                        # bitwise
```

## Keywords
```zor
fun var return say if else loop while repeat
stop next struct enum match unsafe extern
use get trait impl async spawn await
attempt catch defer true false null
self mut give is in or otherwise
```

## CLI
```bash
zor run app.zor            # build + run
zor build app.zor          # compile
zor check app.zor          # parse only
zor build src/             # multi-file project
zor --update               # update to latest version
ZOR_NO_STD=1 zor build k.zor  # OS kernel
```

## Error Codes

Zor translates Rust errors into Zor-friendly messages:

| Rust Code | Zor Explanation |
|---|---|
| E0308 | type mismatch → check function signature |
| E0425 | undefined name → declare or import |
| E0432 | import not found → check crate/dependency |
| E0133 | FFI needs unsafe → wrap in `unsafe {}` |
| E0382 | value moved → borrow with `&ref` |
| E0502 | borrow conflict → mutable + immutable |
| E0596 | can't borrow as mutable → use `&mut` |
| E0507 | can't move from borrowed → `.clone()` |
| E0317 | if without else → add else branch |
| E0369 | can't combine types → cast/convert |
| E0601 | no main function → add `fun main int {}` |
| E0603 | private item → check visibility |
| E0599 | no method → check impl |
| E0277 | trait not satisfied → implement trait |
| E0061 | wrong arg count → check signature |
| E0412 | type not found → import or check spelling |

## Troubleshooting

```bash
# "rustc not found" → install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# "package not found" → check search paths
ls ~/.zor/pkgs/ && ls ./std/

# "ownership error" → use references
# Moved value → pass &value instead
# Need mutation → use &mut value  
# Need copy → call .clone() on struct
```
