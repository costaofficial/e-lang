# Grammatica formale di E (EBNF)

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

## 4) Script block (immediato)

```
script_block = "do", actions, "done", [ "or", fallback ] ;
```

---

## 5) Actions (sequenza)

```
actions   = { statement } ;  (* sequenziale *)
```

---

## 6) Statement (tutti possono avere fallback)

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

## 8) With block (imposta current object)

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

## 9) UI actions (`find` imposta current element)

```
ui_action = "click", [ selector ]
          | "find", selector
          ;

selector  = string ;
```

📌 `click` senza selector usa il current element impostato da `find`.
📌 Errore se non c'è né selector né current element.

---

## 10) Write action

```
write_action = "write", ( object, string | string ) ;
```

📌 Se manca object, usa il current object (da `with`).
📌 Errore se non c'è current object.

---

## 11) Transfer action

```
transfer_action = ("email" | "upload"), "to", target, [ object ] ;

target = string ;
```

📌 `to` specifica la destinazione.
📌 object opzionale → usa current object se omesso.

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

(string = path da osservare)

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

📌 `stop` ferma il blocco `with` o `retry` più vicino. Se fuori da entrambi, ferma l'intero programma.

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

## 19) Commenti

```
comment = "//", { character }, newline ;
```

I commenti sono ignorati dal parser.

---

## Riassunto regole runtime

| Concetto | Impostato da | Usato da | Errore se |
|----------|-------------|----------|-----------|
| Current element | `find` | `click` senza args | `click` senza né args né current element |
| Current object | `with` | `write`, `upload`, `email` senza object | azione richiede object ma non c'è contesto |
