# E — Language Specification

> v5.0 — May 2026 (Complete language: Rust runtime, headless browser, WebView UI, built-in plugins)

---

## 1. Philosophy

E is a **general-purpose language** that describes **when** to do something, **on what**, and **what to do**. The runtime handles complexity — you describe intent.

Core principles:
- **Declarative** — you say *what*, not *how*
- **Human-readable** — syntax is minimal, no punctuation noise
- **Event-oriented** — time, conditions, actions are built-in
- **Consistent** — `... do ... done` for everything

---

## 2. Syntax rules

| Rule | Explanation |
|------|-------------|
| Everything is `... do ... done` | No braces, no `:`, no `end` |
| `// comment` | Line comments only |
| `"strings"` | Double-quoted |
| Identifiers | `[a-zA-Z_][a-zA-Z0-9_]*` |
| Sequential by default | Actions run one after another, top to bottom |
| `or` on any statement | Every action can have a fallback |

---

## 3. File formats

### Standard `.eee` files

A program is a sequence of **statement units**:

```
program    = { statement_unit } ;
statement_unit = time_unit | script_unit | fn_definition | let_statement | use_statement ;
```

### 3-tier `.eee` files

An `.eee` file can also contain three sections:

```
efile = ":sys", <plugin declarations>
      | ":core", <E code>
      | ":ui", <HTML/JS>
      ;
```

- **`:sys`** — declares which Rust `.so` plugins to load
- **`:core`** — E code (variables, functions, logic)
- **`:ui`** — HTML + JavaScript interface (shown in a native window)

All three sections in one file. The runtime executes `:core`, loads `:sys` plugins, and opens `:ui` in a WebView.

**Example:**

```eee
:sys
use "db.eso"
use "http.eso"

:core
do
    let result = sys_call "db.eso" "query" "SELECT * FROM users"
    log result
done

:ui
<h1>App</h1>
<script>console.log('ready');</script>
```

---

## 4. Scheduling (`time`)

**Purpose:** Run code at specific times.

```
time_unit = "time", schedule, "do", actions, "done", [ "or", fallback ] ;

schedule   = "every", interval, [ "at", time ]
           | "at", time
           ;
interval   = "hour" | "day" | "minute", number ;
time       = hour, ":", minute ;
```

**Examples:**

```e
// every hour at the 0th minute
time every hour at 00 do
    log "tick"
done

// every day at 2am
time every day at 02:00 do
    backup
done

// once at 6pm today (or tomorrow if past)
time at 18:00 do
    send report
done
```

**Runtime behavior:**
- **Dry-run:** executes immediately once
- **Live (APScheduler):** schedules for real, keeps process alive with `--watch`

---

## 5. Immediate scripts (`do`)

**Purpose:** Run code right now, no scheduling.

```
script_unit = "do", actions, "done", [ "or", fallback ] ;
```

**Example:**

```e
do
    run "echo hello"
    log "done"
done or log error
```

---

## 6. Context units (`with`)

**Purpose:** Set the scope/object for inner actions.

```
with_unit = "with", object, [ "{", config, "}" ], "do", actions, "done" ;

object    = "file", string
          | "browser"
          | "page"
          | "app", string
          ;

config    = "timeout", ":", duration ;
duration  = number, "s" | number, "ms" ;
```

**Examples:**

```e
with file "data.txt" do
    write "hello"     // writes to file "data.txt"
done

with browser do
    open "https://example.com"

    with page { timeout: 10s } do
        find "#login"
        click
    done
done
```

**Context inheritance:**
- `with file` → inner `write` without a file argument uses this file
- `with browser` → starts a Playwright browser; inner `open`, `click`, `find` use it
- `with page` → optional grouping, allows setting `timeout`

---

## 7. Variables (`let`)

**Purpose:** Store values in named variables.

```
let_statement = "let", identifier, "=", expression ;
```

**Examples:**

```e
let x = 5
let name = "Costa"
let result = double 42
let list = [1, 2, 3]
let item = list[0]
```

Variables are lexically scoped. Inner scopes (from `with`, `do`) can see outer variables.

---

## 8. Functions (`fn`)

**Purpose:** Define reusable logic with a name and parameters.

```
fn_definition = "fn", identifier, { identifier }, "do", actions, "done" ;
```

- Parameters are space-separated, no commas or parentheses
- Return value is the last expression evaluated
- Types are dynamic

**Examples:**

```e
fn double n do
    n * 2
done

fn greet name do
    "hello " + name
done

let x = double 21     // 42
let msg = greet "E"   // "hello E"
```

Functions can be defined at the top level of a file or inside a `do` block.

---

## 9. Lists

**Purpose:** Ordered collections of values.

```
list_literal = "[", [ expression, { ",", expression } ], "]" ;
```

**Operations:**

| Operation | Example | Result |
|-----------|---------|--------|
| Create | `[1, 2, 3]` | New list |
| Index | `nums[0]` | First element |
| Append | `nums.append 4` | `[1, 2, 3, 4]` |

**Examples:**

```e
let nums = [1, 2, 3]
log nums[0]          // 1
nums.append 4
log nums             // [1, 2, 3, 4]

for n in nums do
    log n
done
```

---

## 10. Loops (`for`)

**Purpose:** Iterate over a list or collection.

```
for_statement = "for", identifier, "in", expression, "do", actions, "done" ;
```

**Examples:**

```e
// iterate over a list
for n in [1, 2, 3] do
    log n
done

// iterate over files
for f in ls "*.md" do
    log f
done

// iterate over command output lines
for line in run "cat file.txt" do
    log line
done
```

Strings are split by newlines when iterated. Lists iterate element by element.

---

## 11. Modules (`use`)

**Purpose:** Load code from another file.

```
use_statement = "use", string ;
```

**Example:**

```e
// lib.eee
fn double n do n * 2 done

// main.eee
do
    use "lib"
    log double 5
done
```

Modules can define functions and variables at the top level. Functions defined in imported modules are available in the importing scope.

---

## 12. Expressions

**Purpose:** Compute values.

```
expression = comparison ;

comparison  = addition, { ("=" | ">" | "<" | ">=" | "<=" | "==" | "!="), addition } ;
addition    = term, { ("+" | "-" | "and" | "or"), term } ;
term        = unary, { ("*" | "/"), unary } ;
unary       = [ "-" ], factor ;
factor      = number | string | list_literal | "(", expression, ")"
            | identifier, [ expression ]           (* function call *)
            | "run", string, [ "with", expression ] (* shell command *)
            | "read", string                        (* read file *)
            | "ls", [ string ]                      (* list files *)
            | factor, "[", expression, "]"           (* indexing *)
            | factor, ".", identifier, [ expression ] (* method call *)
            ;
```

**Operator precedence (high to low):**

| Level | Operators |
|-------|-----------|
| Unary | `-` |
| Multiplicative | `*` `/` |
| Additive | `+` `-` `and` `or` |
| Comparison | `=` `>` `<` `>=` `<=` `==` `!=` |

**Note:** `=` in expressions is **comparison** (equals). Assignment is `let`.

---

## 13. Actions

All available actions:

| Action | Syntax | What it does |
|--------|--------|-------------|
| `open` | `open "url"` | Opens a URL in browser |
| `click` | `click` or `click "selector"` | Clicks element |
| `find` | `find "selector"` | Sets current element |
| `find all` | `find all "selector"` | Finds all elements, sets `count` |
| `get number` | `get number from "selector"` | Extracts a number, sets `number` |
| `write` | `write file "x" "content"` | Writes to file |
| `login` | `login "user" "pass"` | Browser login |
| `email` | `email to "addr" file "x"` | Sends email |
| `upload` | `upload to "url" file "x"` | Uploads file |
| `create` | `create "name"` | Creates file |
| `log` | `log expr` | Prints value |
| `stop` | `stop` | Halts current unit |
| `wait download` | `wait download` | Waits for download |
| `wait until` | `wait until visible "sel"` | Waits for element state |

**Built-in expressions (usable anywhere, not just as actions):**

| Expression | What it does |
|------------|-------------|
| `run "cmd"` | Runs shell command, returns stdout |
| `run "cmd" with data` | Runs shell command with stdin data |
| `read "path"` | Reads file, returns content as string |
| `ls "*.md"` | Lists files matching glob, newline-separated |
| `[1, 2, 3]` | Creates a list |
| `list[i]` | Indexes into a list |
| `obj.method arg` | Calls a method on an object |

---

## 14. Conditions (`when`)

**Purpose:** Execute actions only when a condition is true.

```
when_unit = "when", condition, "do", actions, "done" ;

condition = "item", ("visible" | "hidden")
          | ("number" | "count"), ("=" | ">" | "<" | ">=" | "<="), number
          ;
```

**Three semantic variables:**

| Variable | Set by | Type | Meaning |
|----------|--------|------|---------|
| `item` | `find`, `find all` | Any | Current thing (element, list, value) |
| `number` | `get number` | Numeric | Numeric value extracted from something |
| `count` | `find all` | Numeric | Number of elements found |

**Examples:**

```e
find all ".product"
when count > 10 do
    log "more than 10 products"
done

get number from "#price"
when number > 100 do
    log "expensive"
done

find "#loading"
when item visible do
    log "still loading..."
done
when item hidden do
    log "loaded"
done
```

---

## 15. Retry

**Purpose:** Retry a unit of actions on failure.

```
retry_unit = "retry", number, "times", "do", actions, "done" ;
```

**Example:**

```e
retry 3 times do
    click "#export-btn"
    wait until visible ".dashboard"
done or do
    log "export failed after 3 retries"
    stop
done
```

---

## 16. Wait

**Purpose:** Wait for something to happen.

```
wait_statement = "wait", "until", condition
               | "wait", "download"
               ;

condition    = "visible", selector
             | "hidden",  selector
             ;
```

**Examples:**

```e
wait until visible "#chart"
wait until hidden ".loading"
wait download
```

---

## 17. Watch

**Purpose:** React to filesystem changes.

```
watch_unit = "watch", string, "do", actions, "done" ;
```

**Example:**

```e
watch "downloads/" do
    with file "*.csv" do
        upload to "https://api.import.com/csv"
    done
done
```

⚠️ **Currently simulated** — runs once, doesn't actually watch.

---

## 18. Error handling (`or`)

**Purpose:** Fallback when something fails. Available on **every statement**.

```
statement = core_statement, [ "or", fallback ] ;

fallback = simple_fallback
         | "do", actions, "done"
         ;
```

**Examples:**

```e
// simple fallback
click "#btn" or log "no button found"

// block fallback (local)
login "user" "pass" or do
    log "login failed"
    stop
done

// block fallback (global on time unit)
time every day at 02:00 do
    ...
done or log error
```

**Fallback chain:**
1. Local fallback on the action itself
2. Unit-level fallback (e.g., on `time` / `do`)
3. Error propagates up if neither exists

---

## 19. Runtime architecture

```
.eee file
     ↓
Section parser (:sys / :core / :ui)
     ↓
:sys → PluginManager (libloading) → loads .so modules
:core → Lexer → Parser → AST → Executor → Driver
:ui   → WebView window (HTML/JS)
```

```
Driver (interface)
 ├── DryDriver (logs everything, safe)
 └── RealDriver (actually executes)
      ├── Browser (chromium via webview)
      ├── File I/O (std::fs)
      ├── Shell (subprocess)
      ├── Email (lettre SMTP)
      └── Watchdog (fs polling)
```

---

## 20. Current status (v5.0)

| Feature | Status | Engine |
|---------|--------|--------|
| Lexer + Parser | ✅ Complete | Hand-written recursive descent |
| Variables, functions, conditions | ✅ Complete | Scope stack (Vec<HashMap>) |
| Expressions + operators | ✅ Complete | Precedence, and/not |
| Lists | ✅ Complete | sort, join, get, len, append |
| String methods | ✅ Complete | split, contains, replace, trim, lower, upper |
| Numbers | ✅ Complete | Unified f64 |
| Loops (for, while) | ✅ Complete | Any iterable |
| Modules (use) | ✅ Complete | Multi-file, plugin names |
| 3-tier files (:sys :core :ui) | ✅ Complete | Plugin + E + WebView |
| Browser automation | ✅ Complete | headless_chrome (Chrome DevTools) |
| WebView (:ui) | ✅ Complete | wry + tao (native window) |
| `time` scheduling + `--watch` | ✅ Complete | Threaded loop |
| `retry` + `or` fallback | ✅ Complete | On any statement |
| `with` context | ✅ Complete | File, browser, page |
| write / read / ls / run | ✅ Complete | std::fs, subprocess |
| `email` | ✅ Complete | SMTP via lettre |
| `watch` filesystem | ✅ Complete | Directory polling |
| CLI args (`args` variable) | ✅ Complete | Trailing var arg |
| Built-in plugins | ✅ Complete | json, fs, db, http |

---

## 21. Complete example

```eee
:sys
use "db.eso"

:core
fn salva_utente nome do
    sys_call "db.eso" "insert" nome
    log "salvato: " + nome
done

do
    for nome in ["Alice", "Bob"] do
        salva_utente nome
    done
done

:ui
<h1>Users</h1>
<script>
fetch('/api/users').then(r => r.json()).then(d => console.log(d));
</script>
```

---

## 22. CLI usage

```bash
# Build from source
cd e/
cargo build --release
./target/release/e examples/hello.eee

# Or install globally
sudo cp target/release/e /usr/local/bin/
e examples/hello.eee
```

No dependencies required. Single binary.
