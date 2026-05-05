# E — formal grammar (EBNF)

---

## 1) Program

```
program    = { statement_unit } ;

statement_unit = time_unit | script_unit | fn_definition | let_statement | import_statement ;
```

Top-level can be `time`, `do`, `fn`, `let`, or `import`.

---

## 2) Scheduling

```
time_unit = "time", schedule, "do", actions, "done", [ "or", fallback ] ;

schedule   = "every", interval, [ "at", time ]
           | "at", time
           ;

interval   = "hour" | "day" | "minute", number ;
time       = hour, ":", minute ;
hour       = number ;  (* 0-23 *)
minute     = number ;  (* 0-59 *)
```

---

## 3) Script block

```
script_unit = "do", actions, "done", [ "or", fallback ] ;
```

---

## 4) Actions

```
actions = { statement } ;
```

All actions run sequentially, top to bottom.

---

## 5) Statements

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
    | get_number_action
    | find_all_action
    | let_statement
    | fn_definition
    | for_statement
    | import_statement
    | expression
    ;
```

---

## 6) Let (variable assignment)

```
let_statement = "let", identifier, "=", expression ;
```

`let` always creates a new variable in the current scope. `=` is assignment, not comparison.

---

## 7) Fn (function definition)

```
fn_definition = "fn", identifier, { identifier }, "do", actions, "done" ;
```

- Parameters are space-separated, no commas or parentheses
- Return value is the last expression evaluated in the body
- Types are dynamic

---

## 8) For loop

```
for_statement = "for", identifier, "in", expression, "do", actions, "done" ;
```

Iterates over a list, a string (line-split), or any iterable.

---

## 9) Import

```
import_statement = "import", string ;
```

Loads and executes another `.e` file. All its `fn` definitions and top-level code become available.

---

## 10) When (condition)

```
when_unit = "when", condition, "do", actions, "done" ;

condition = "item", ("visible" | "hidden")
          | ("number" | "count"), ("=" | ">" | "<" | ">=" | "<="), number
          | expression
          ;
```

Conditions support both semantic keywords (`item visible`, `count > 5`) and general expressions (`result > 200`).

---

## 11) Expressions

```
expression = comparison ;

comparison = addition, { ("=" | ">" | "<" | ">=" | "<=" | "==" | "!="), addition } ;

addition = term, { ("+" | "-" | "and" | "or"), term } ;

term = unary, { ("*" | "/"), unary } ;

unary = [ "-" ], factor ;

factor = number
       | string
       | list_literal
       | "(", expression, ")"
       | identifier, [ expression ]                  (* function call *)
       | "run", string, [ "with", expression ]       (* shell command *)
       | "read", string                              (* read file *)
       | "ls", [ string ]                            (* list files *)
       | factor, "[", expression, "]"                (* indexing *)
       | factor, ".", identifier, [ expression ]     (* method call *)
       ;

list_literal = "[", [ expression, { ",", expression } ], "]" ;
```

Operator precedence (highest to lowest):

| Level | Operators |
|-------|-----------|
| Unary | `-` |
| Multiplicative | `*` `/` |
| Additive | `+` `-` `and` `or` |
| Comparison | `=` `>` `<` `>=` `<=` `==` `!=` |

📌 `=` in expressions is always **comparison** (equals). Assignment is done via `let`.

---

## 12) Context (`with`)

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

---

## 13) UI actions

```
ui_action = "click", [ selector ]
          | "find", selector
          | "find", "all", selector
          ;

get_number_action = "get", "number", [ "from", selector ] ;
```

- `find` sets current element (single)
- `find all` sets `item` to list and `count` to length
- `get number` extracts a numeric value, sets `number` in context

---

## 14) Write

```
write_action = "write", ( object, string | string ) ;
```

If object is omitted, uses current object from `with file`.

---

## 15) Transfer

```
transfer_action = ("email" | "upload"), "to", target, [ object ] ;

target = string ;
```

- `to` specifies the destination
- object is optional → uses current object from context

---

## 16) Retry

```
retry_unit = "retry", number, "times", "do", actions, "done" ;
```

---

## 17) Wait

```
wait_statement = "wait", "until", condition
               | "wait", "download"
               ;

condition    = "visible", selector
             | "hidden",  selector
             ;
```

---

## 18) Watch

```
watch_unit = "watch", string, "do", actions, "done" ;
```

(string = path to watch)

---

## 19) Login

```
login_statement = "login", string, string ;
```

---

## 20) Stop

```
stop_statement = "stop" ;
```

Halts the nearest enclosing `with` or `retry`. If outside both, halts the program.

---

## 21) Log

```
log_action = "log", expression ;
```

Accepts any expression, not just strings.

---

## 22) Fallback

```
fallback = simple_fallback
         | "do", actions, "done"
         ;

simple_fallback = log_action | stop_statement ;
```

Every statement can have an `or` fallback.

---

## 23) Literals

```
string  = '"', { character }, '"' ;
number  = digit, { digit } ;
digit   = "0" | "1" | ... | "9" ;
identifier = letter, { letter | digit | "_" } ;
```

---

## 24) Comments

```
comment = "//", { character }, newline ;
```

---

## Runtime variables

| Variable | Set by | Type | Meaning |
|----------|--------|------|---------|
| `item` | `find`, `find all` | Any | Current thing (element, list) |
| `number` | `get number` | Numeric | Extracted numeric value |
| `count` | `find all` | Numeric | Number of elements |
| user variables | `let x = ...` | Any | Any value assigned via `let` |
