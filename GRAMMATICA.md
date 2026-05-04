# E formal grammar (EBNF)

---

## 1) Program

```
program    = { statement_block } ;
```

---

## 2) Statement blocks

```
statement_block = time_block | script_block ;
```

---

## 3) Time block

```
time_block = "time", schedule, "do", actions, "done", [ "or", fallback ] ;

schedule   = "every", interval, [ "at", time ]
           | "at", time
           ;

interval   = "hour" | "day" | "minute", number ;
time       = hour, ":", minute ;
hour       = number ;  (* 0-23 *)
minute     = number ;  (* 0-59 *)
```

---

## 4) Script block (immediate)

```
script_block = "do", actions, "done", [ "or", fallback ] ;
```

---

## 5) Actions (sequential)

```
actions   = { statement } ;  (* sequential *)
```

---

## 6) Statement (all can have fallback)

```
statement = core_statement, [ "or", fallback ] ;

core_statement =
      with_block
    | retry_block
    | wait_statement
    | watch_block
    | login_statement
    | stop_statement
    | write_action
    | transfer_action
    | ui_action
    | log_action
    ;
```

---

## 7) Fallback

```
fallback = simple_fallback
         | "do", actions, "done"
         ;

simple_fallback = log_action | stop_statement ;
```

---

## 8) With block (sets current object)

```
with_block = "with", object, [ "{", config, "}" ], "do", actions, "done" ;

object    = "file", string
          | "browser"
          | "page"
          | "app", string
          ;

config    = "timeout", ":", duration ;
duration  = number, "s" | number, "ms" ;
```

---

## 9) UI actions (`find` sets current element)

```
ui_action = "click", [ selector ]
          | "find", selector
          ;

selector  = string ;
```

📌 `click` without selector uses the current element set by `find`.
📌 Error if there is neither a selector nor a current element.

---

## 10) Write action

```
write_action = "write", ( object, string | string ) ;
```

📌 If object is missing, uses the current object (from `with`).
📌 Error if there is no current object.

---

## 11) Transfer action

```
transfer_action = ("email" | "upload"), "to", target, [ object ] ;

target = string ;
```

📌 `to` specifies the destination.
📌 object is optional → uses current object if omitted.

---

## 12) Retry block

```
retry_block = "retry", number, "times", "do", actions, "done" ;
```

---

## 13) Wait statement

```
wait_statement = "wait", "until", condition
               | "wait", "download"
               ;

condition    = "visible", selector
             | "hidden",  selector
             ;
```

---

## 14) Watch block

```
watch_block = "watch", string, "do", actions, "done" ;
```

(string = path to watch)

---

## 15) Login statement

```
login_statement = "login", string, string ;
```

---

## 16) Stop statement

```
stop_statement = "stop" ;
```

📌 `stop` halts the nearest enclosing `with` or `retry` block. If outside both, it halts the entire program.

---

## 17) Log action

```
log_action = "log", string ;
```

---

## 18) Literals

```
string  = '"', { character }, '"' ;
number  = digit, { digit } ;
digit   = "0" | "1" | ... | "9" ;
```

---

## 19) Comments

```
comment = "//", { character }, newline ;
```

Comments are ignored by the parser.

---

## Runtime rules summary

| Concept | Set by | Used by | Error if |
|---------|--------|---------|----------|
| Current element | `find` | `click` without args | `click` without args and no current element |
| Current object | `with` | `write`, `upload`, `email` without object | action needs object but no context |
