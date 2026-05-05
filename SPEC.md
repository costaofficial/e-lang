# E — Language Specification

> v0.5 — May 2026

---

## 1. Philosophy

E is a **general-purpose language** that describes **when** to do something, **on what**, and **what to do**. The runtime handles complexity — you describe intent.

Core principles:
- **Declarative** — you say *what*, not *how*
- **Human-readable** — syntax is minimal, no punctuation noise
- **Event-oriented** — time, conditions, retries are built-in, not bolted on
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

## 3. Programs

A program is a sequence of **statement units**:

```
program    = { statement_unit } ;
statement_unit = time_unit | script_unit ;
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

## 7. Actions

All available actions:

| Action | Syntax | What it does |
|--------|--------|-------------|
| `open` | `open "url"` | Opens a URL in browser |
| `click` | `click` or `click "selector"` | Clicks element |
| `find` | `find "selector"` | Sets current element |
| `find all` | `find all "selector"` | Finds all elements, sets `count` |
| `get number` | `get number from "selector"` | Extracts a number, sets `number` |
| `write` | `write file "x" "content"` | Writes to file |
| `run` | `run "command"` | Runs shell command |
| `login` | `login "user" "pass"` | Browser login |
| `email` | `email to "addr" file "x"` | Sends email |
| `upload` | `upload to "url" file "x"` | Uploads file |
| `create` | `create "name"` | Creates file |
| `log` | `log "message"` | Prints message |
| `stop` | `stop` | Halts current unit |
| `wait download` | `wait download` | Waits for download |
| `wait until` | `wait until visible "sel"` | Waits for element state |

---

## 8. Conditions (`when`)

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

## 9. Retry

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

## 10. Wait

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

## 11. Watch

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

## 12. Error handling (`or`)

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

## 13. Runtime architecture

```
E source code
     ↓
Lexer (tokenizer)
     ↓
Parser (recursive descent)
     ↓
AST (typed nodes)
     ↓
Executor (walks AST)
     ↓
Driver (interface)
 ├── DryDriver (logs everything, safe)
 └── RealDriver (actually executes)
      ├── SchedulerDriver (APScheduler)
      ├── FileDriver (filesystem I/O)
      ├── BrowserDriver (Playwright)
      ├── EmailDriver (SMTP — stub)
      └── WatcherDriver (watchdog — stub)
```

---

## 14. Current status

| Feature | Status | Notes |
|---------|--------|-------|
| Lexer + Parser | ✅ Complete | EBNF grammar, all constructs |
| AST + dump | ✅ Complete | Typed nodes, pretty printer |
| Dry-run mode | ✅ Complete | Logs everything, no side-effects |
| Live mode | ✅ Complete | Actually executes |
| `time` scheduler | ✅ Complete | APScheduler |
| `do` script units | ✅ Complete | Immediate execution |
| `with` context | ✅ Complete | File, browser, page |
| `find` / `click` | ✅ Complete | Playwright |
| `find all` / `count` | ✅ Complete | Playwright |
| `get number` | ✅ Complete | Playwright |
| `when` conditions | ✅ Complete | `item`, `number`, `count` |
| `wait visible/hidden` | ✅ Complete | Playwright |
| `retry` | ✅ Complete | With fallback |
| `or` fallback | ✅ Complete | Local + block |
| `write` / `create` | ✅ Complete | Filesystem |
| `run` | ✅ Complete | Subprocess |
| `log`, `stop` | ✅ Complete | Built-in |
| `login` | ⏳ Stub | Playwright auto-detect |
| `email` | ⏳ Stub | SMTP config |
| `wait download` | ⏳ Stub | Browser download handler |
| `watch` | ⏳ Stub | Watchdog library |
| Variables | ❌ Not yet | `let`, `fn`, `for` — future |
| Types | ❌ Not yet | Future |
| Modules | ❌ Not yet | Future |

---

## 15. Complete example

```e
// Weekly report — every day at 2am
time every day at 02:00 do
    // 1. Extract data from server
    with browser do
        open "https://connect.api"
        with page { timeout: 10s } do
            login "user" "password" or stop
            click "#export-all"
            wait download
        done
    done

    // 2. Process downloaded files
    watch "downloads/" do
        with file "*.fit" do
            upload to "https://fitness.db/import"
            log "fit imported"
        done
    done

    // 3. Notify
    email to "admin@example.com" file "report.pdf"
done or log error
```

---

## 16. CLI usage

```bash
# dry-run (default, safe)
python3 runtime/run_e.py script.e

# live execution (writes files, opens browser, etc.)
python3 runtime/run_e.py --live script.e

# keep alive for scheduled tasks
python3 runtime/run_e.py --live --watch script.e
```

### Dependencies

```bash
# Always needed: none (stdlib only)

# For scheduler:
pip install apscheduler

# For browser automation:
pip install playwright
playwright install chromium
```
