# E — formal grammar (EBNF)

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

Three sections: plugins (raw text), core code (E parsed), UI (HTML/JS raw).

---

## 3) Scheduling

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

## 4) Script block

```
script_unit = "do", actions, "done", [ "or", fallback ] ;
```

---

## 5) Actions

```
actions = { statement } ;
```

All actions run sequentially.

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

`let` creates a new variable. Lexical scoping.

---

## 8) Functions

```
fn_definition = "fn", identifier, { identifier }, "do", actions, "done" ;
```

- Parameters space-separated, no commas or parentheses
- Last expression is the return value
- Dynamic types

---

## 9) For loop

```
for_statement = "for", identifier, "in", expression, "do", actions, "done" ;
```

---

## 10) While loop

```
while_unit = "while", condition, "do", actions, "done" ;
```

---

## 11) Conditions

```
when_unit = "when", condition, "do", actions, "done" ;

condition = "item", ("visible" | "hidden")
          | ("number" | "count"), ("=" | ">" | "<" | ">=" | "<="), number
          | expression
          ;
```

---

## 12) Modules

```
use_statement = "use", string ;
```

Loads and executes another `.eee` file. All `fn` and `let` become available.

---

## 13) Plugin call (3-tier)

```
sys_call = "sys_call", expression, expression, [ expression ] ;
```

Calls a function from a loaded `.so` plugin. Arguments: plugin path, function name, optional input string.

---

## 14) Expressions

```
expression = comparison ;

comparison = addition, { ("=" | ">" | "<" | ">=" | "<=" | "==" | "!="), addition } ;

addition = term, { ("+" | "-"), term } ;

term = unary, { ("*" | "/"), unary } ;

unary = [ "-" | "not" ], factor ;

factor = number
       | string
       | list_literal
       | "(", expression, ")"
       | identifier, { expression }          (* function call with args *)
       | "run", string, [ "with", expression ]
       | "read", string
       | "ls", [ string ]
       | "len", expression
       | factor, "[", expression, "]"
       | factor, "[", expression, "..", expression, "]"
       | factor, ".", identifier, [ expression ]
       ;

list_literal = "[", [ expression, { ",", expression } ], "]" ;
```

Operator precedence (high to low): `* /` → `+ -` → comparisons.

---

## 15) Browser actions

```
with_unit = "with", object, [ "{", config, "}" ], "do", actions, "done" ;

ui_action = "click", [ selector ]
          | "find", selector
          | "find", "all", selector
          ;

get_number_action = "get", "number", [ "from", selector ] ;
login_statement = "login", string, string ;

wait_statement = "wait", "until", condition
               | "wait", "download"
               ;
```

`with browser` starts a browser session. `with page` sets context.

---

## 16) Retry

```
retry_unit = "retry", number, "times", "do", actions, "done" ;
```

---

## 17) Watch

```
watch_unit = "watch", string, "do", actions, "done" ;
```

Monitors a directory for new files.

---

## 18) Transfer actions

```
transfer_action = ("email" | "upload"), "to", target, [ object ] ;

write_action = "write", ( object, string | string ) ;
```

---

## 19) Error handling

```
fallback = simple_fallback
         | "do", actions, "done"
         ;

simple_fallback = log_action | stop_statement ;
```

Every statement can have an `or` fallback.

---

## 20) Literals

```
string  = '"', { character }, '"' ;
number  = digit, { digit } ;
digit   = "0" | "1" | ... | "9" ;
identifier = letter, { letter | digit | "_" } ;
```

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
| user variables | `let x = ...` | Any | Any value |
