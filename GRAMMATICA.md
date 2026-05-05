# E — formal grammar (EBNF)

---

## 1) Program

```
program    = { statement_unit } ;
```

---

## 2) Statement blocks

```
statement_unit = time_unit | script_unit ;
```

---

## 3) Time block

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

## 4) Script block (immediate)

```
script_unit = "do", actions, "done", [ "or", fallback ] ;
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
    ;
```

---

## 7) When block

```
when_unit = "when", condition, "do", actions, "done" ;

condition = "item", ("visible" | "hidden")
          | ("number" | "count"), ("=" | ">" | "<" | ">=" | "<="), number
          ;
```

📌 `item visible` / `item hidden` controlla lo stato dell'elemento corrente.
📌 `number > 5` / `count <= 10` confronta il valore corrente con un numero.

---

## 8) Get number

```
get_number_action = "get", "number", [ "from", selector ] ;
```

📌 Estrae un valore numerico dall'elemento corrente o da un selettore.
📌 Il risultato viene salvato in `number`.

---

## 9) Find all

```
find_all_action = "find", "all", selector ;
```

📌 Trova tutti gli elementi che匹配 il selettore.
📌 `count` = numero di elementi trovati.
📌 `item` = la lista completa.

---

## 10) Fallback

```
fallback = simple_fallback
         | "do", actions, "done"
         ;

simple_fallback = log_action | stop_statement ;
```

---

## 8) With block (sets current object)

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

## 11) UI actions (`find` sets current element)

```
ui_action = "click", [ selector ]
          | "find", selector
          ;

selector  = string ;
```

📌 `click` without selector uses the current element set by `find`.
📌 Error if there is neither a selector nor a current element.

---

## 12) Write action

```
write_action = "write", ( object, string | string ) ;
```

📌 If object is missing, uses the current object (from `with`).
📌 Error if there is no current object.

---

## 13) Transfer action

```
transfer_action = ("email" | "upload"), "to", target, [ object ] ;

target = string ;
```

📌 `to` specifies the destination.
📌 object is optional → uses current object if omitted.

---

## 14) Retry block

```
retry_unit = "retry", number, "times", "do", actions, "done" ;
```

---

## 15) Wait statement

```
wait_statement = "wait", "until", condition
               | "wait", "download"
               ;

condition    = "visible", selector
             | "hidden",  selector
             ;
```

---

## 16) Watch block

```
watch_unit = "watch", string, "do", actions, "done" ;
```

(string = path to watch)

---

## 17) Login statement

```
login_statement = "login", string, string ;
```

---

## 18) Stop statement

```
stop_statement = "stop" ;
```

📌 `stop` halts the nearest enclosing `with` or `retry` block. If outside both, it halts the entire program.

---

## 19) Log action

```
log_action = "log", string ;
```

---

## 20) Literals

```
string  = '"', { character }, '"' ;
number  = digit, { digit } ;
digit   = "0" | "1" | ... | "9" ;
```

---

## 21) Comments

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
