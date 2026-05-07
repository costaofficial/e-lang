<!-- Copyright (c) 2025 Costa -->
# E — formal grammar (EBNF)

> v5.0

---

## 1) Program

```
program    = { statement_unit } ;

statement_unit = time_unit | script_unit | fn_definition | let_statement | use_statement ;
```

Top-level can be `time`, `do`, `fn`, `let`, or `use`.

---

## 2) 3-tier file (.eee)

```
efile = ":sys", { line }, ":core", { line | statement }, ":ui", { line } ;
```

- `:sys` — plugin declarations (built-in modules: json, fs, db, http)
- `:core` — E code (variables, functions, logic)
- `:ui` — HTML + JavaScript (shown in WebView window)

---

## 3) Scheduling

```
time_unit = "time", schedule, "do", actions, "done", [ "or", fallback ] ;

schedule   = "every", interval, [ "at", time ]
           | "at", time
           ;

interval   = "hour" | "day" | "minute" | "week" ;
time       = hour, ":", minute ;
hour       = number ;  (* 0-23 *)
minute     = number ;  (* 0-59 *)
```

With `--watch` flag, scheduled tasks repeat at the specified interval.

---

## 4) Script block

```
script_unit = "do", actions, "done", [ "or", fallback ] ;
```

---

## 5) Actions

```
actions = { statement } ;
```

---

## 6) Statements

```
statement = core_statement, [ "or", fallback ] ;

core_statement =
      with_unit
    | retry_unit
    | wait_statement
    | watch_unit
    | login_statement
    | stop_statement
    | write_action
    | transfer_action
    | ui_action
    | log_action
    | when_unit
    | while_unit
    | for_statement
    | let_statement
    | fn_definition
    | use_statement
    | expression
    ;
```

---

## 7) Variables

```
let_statement = "let", identifier, "=", expression ;
```

Lexical scoping via scope stack (`Vec<HashMap>`). Each function call pushes a new scope layer.

The special variable `args` contains CLI arguments: `args[0]` is the script path, `args[n]` are additional arguments.

---

## 8) Functions

```
fn_definition = "fn", identifier, { identifier }, "do", actions, "done" ;
```

- Parameters space-separated, no parentheses
- Last expression is the return value
- Dynamic types (unified `f64` for all numbers)
- Functions see outer scope variables via scope stack

---

## 9) Loops

```
for_statement = "for", identifier, "in", expression, "do", actions, "done" ;
while_unit    = "while", condition, "do", actions, "done" ;
```

---

## 10) Conditions

```
when_unit = "when", condition, "do", actions, "done" ;

condition = "item", ("visible" | "hidden")
          | ("number" | "count"), ("=" | ">" | "<" | ">=" | "<="), number
          | expression
          ;
```

`and` is supported in expressions: `when x > 5 and x < 10 do`.

---

## 11) Modules

```
use_statement = "use", string ;
```

Loads and executes another `.eee` file. Supports both local paths and built-in plugin names (`json`, `fs`, `db`, `http`).

---

## 12) Plugin call

```
sys_call = "sys_call", expression, expression, [ expression, expression ] ;
```

Calls a built-in plugin function. Arguments: plugin name, function name, optional arg1, optional arg2.

Built-in plugins:

| Plugin | Functions |
|--------|-----------|
| `json` | `e_parse`, `e_stringify` |
| `fs` | `e_exists`, `e_size`, `e_copy`, `e_delete` |
| `db` | `e_open`, `e_query` (file-persistent JSON) |
| `http` | `e_get` (url), `e_post` (url\|body), `e_post_json` (JSON `{url, body}`) |

---

## 13) Expressions

```
expression = comparison ;

comparison = addition, { ("=" | ">" | "<" | ">=" | "<=" | "==" | "!="), addition }
           | addition, { "and", comparison }
           ;

addition = term, { ("+" | "-"), term } ;

term = unary, { ("*" | "/"), unary } ;

unary = [ "-" | "not" ], factor ;

factor = number | float | string
       | list_literal
       | "(", expression, ")"
       | identifier, { expression }          (* function call *)
       | "run", string, [ "with", expression ]
       | "read", string
       | "ls", [ string ]
       | "len", expression
       | factor, "[", expression, "]"
       | factor, "[", expression, "..", expression, "]"
       | factor, ".", identifier, { expression }   (* method call *)
       ;

list_literal = "[", [ expression, { ",", expression } ], "]" ;
```

Operator precedence: `not` > `* /` > `+ -` > comparisons.

### Methods

**String methods:** `split`, `contains`, `replace`, `trim`, `lower`, `upper`, `get`, `len`

```eee
"hello".split " "       → ["hello", "world"]
"hello".contains "ell"  → true
"hello".replace "l" "x" → "hexxo"
"  hi  ".trim           → "hi"
"HELLO".lower           → "hello"
"hello".upper           → "HELLO"
"hello".get 0           → "h"
```

**List methods:** `sort`, `join`, `get`, `len`, `append`

```eee
[3, 1, 2].sort          → [1, 2, 3]
[1, 2, 3].join ","      → "1,2,3"
[1, 2, 3].get 1         → 2
[1, 2, 3].len           → 3
[1, 2, 3].append 4      → [1, 2, 3, 4]
```

---

## 14) Browser actions

```
with_unit = "with", object, [ "{", config, "}" ], "do", actions, "done" ;

object    = "file" | "browser" | "page" | "app" ;

ui_action = "click", [ selector ]
          | "find", selector
          | "find", "all", selector
          ;

get_number_action = "get", "number", [ "from", selector ] ;
login_statement   = "login", string, string ;

wait_statement = "wait", "until", condition
               | "wait", "download"
               ;
```

Browser actions use `headless_chrome` (Chrome DevTools Protocol):

| Action | Description |
|--------|-------------|
| `open url` | Navigate to URL (launches Chrome if not running) |
| `find selector` | Wait for element to appear |
| `click selector` | Click element |
| `login user pass` | Auto-detect form, fill and submit |
| `find all selector` | Count matching elements |
| `get number from selector` | Extract numeric value from element text |
| `wait until visible/hidden selector` | Wait for element state |
| `wait download` | Poll download directory for new file |

---

## 15) Retry

```
retry_unit = "retry", number, "times", "do", actions, "done" ;
```

---

## 16) Watch

```
watch_unit = "watch", string, "do", actions, "done" ;
```

Monitors a directory for new files (polling every 2 seconds).

---

## 17) Transfer actions

```
transfer_action = ("email" | "upload"), "to", target, [ object ] ;
write_action    = "write", ( object, string | string ) ;
```

Email uses SMTP via `lettre` crate. Configure with env vars:

```
E_SMTP_HOST, E_SMTP_PORT, E_SMTP_USER, E_SMTP_PASS, E_SMTP_FROM
```

---

## 18) Error handling

```
fallback = simple_fallback
         | "do", actions, "done"
         ;

simple_fallback = log_action | stop_statement ;
```

Every statement can have an `or` fallback. Errors produce clean messages (no raw backtraces).

---

## 19) CLI

```
e [OPTIONS] <FILE> [ARGS...]

OPTIONS:
  --live     Execute actions (default: dry-run)
  --watch    Keep alive for scheduled tasks
  ARGS       Available in E as args variable
```

---

## 20) Literals

```
string  = '"', { character }, '"' ;
number  = digit, { digit } ;
float   = digit, ".", digit ;
digit   = "0" | "1" | ... | "9" ;
identifier = letter, { letter | digit | "_" } ;
```

Numbers are unified as `f64` internally. Display: `5.0` shows as `5`, `3.14` shows as `3.14`.

---

## 21) Comments

```
comment = "//", { character }, newline ;
```

---

## Runtime variables

| Variable | Set by | Type | Meaning |
|----------|--------|------|---------|
| `item` | `find`, `find all` | Any | Current element or list |
| `number` | `get number` | Numeric | Extracted numeric value |
| `count` | `find all` | Numeric | Number of elements |
| `args` | CLI | List | Script arguments: `[path, arg1, ...]` |
| user variables | `let x = ...` | Any | Any value |
